mod bytes;
mod client;
mod config;
mod directory;
mod packet;
mod prometheus;
mod server;
mod user;

use crate::client::Client;
use crate::server::Server;
use crate::user::User;
use config::load_config;
use directory::Directory;
use tokio::signal;
use tokio::task::JoinSet;

const DEFAULT_SERVER_BUFFER_SIZE: usize = 32;
const DEFAULT_DIRECTORY_BUFFER_SIZE: usize = 32;
const DEFAULT_CLIENT_BUFFER_SIZE: usize = 32;

#[tokio::main]
async fn main() {
    // Get app config
    let config_path = "./config";
    let config_env_prefix = "APPCFG";
    let config = load_config(config_path, config_env_prefix).unwrap();

    // Turn on metrics if enabled
    let (mf, metrics_handle) = config
        .metrics
        .and_then(|metrics_config| metrics_config.enable)
        .and_then(|enable| {
            if enable {
                Some(prometheus::setup())
            } else {
                None
            }
        })
        .unzip();
    let metrics_abort_handle = metrics_handle.map(|handle| handle.abort_handle());

    // Create server
    let server_buffer_size = if let Some(server) = config.server {
        server.buffer_size.unwrap_or(DEFAULT_SERVER_BUFFER_SIZE)
    } else {
        DEFAULT_SERVER_BUFFER_SIZE
    };
    let mut s = Server::new(server_buffer_size);
    let server_tx = s.get_tx();
    let server = tokio::spawn(async move { s.listen().await });
    let server_abort_handle = server.abort_handle();

    // Create directory
    let directory_buffer_size = if let Some(directory) = config.directory {
        directory
            .buffer_size
            .unwrap_or(DEFAULT_DIRECTORY_BUFFER_SIZE)
    } else {
        DEFAULT_DIRECTORY_BUFFER_SIZE
    };
    let mut d = Directory::new(directory_buffer_size);
    let directory_tx = d.get_tx();
    let directory = tokio::spawn(async move { d.listen().await });
    let directory_abort_handle = directory.abort_handle();

    // Create clients
    let mut client_set = JoinSet::new();
    let mut user_set = JoinSet::new();
    let mut client_abort_handles = vec![];
    let mut user_abort_handles = vec![];
    if let Some(client_configs) = config.clients {
        for (client_config, next_client_config) in client_configs
            .iter()
            .zip(client_configs.iter().cycle().skip(1))
        {
            let buffer_size = client_config
                .buffer_size
                .unwrap_or(DEFAULT_CLIENT_BUFFER_SIZE);
            let directory_tx = directory_tx.clone();
            let mut client = Client::new(&client_config.id, directory_tx, buffer_size, &mf);
            let client_tx = client.get_tx();
            let server_tx = server_tx.clone();
            client_abort_handles
                .push(client_set.spawn(async move { client.listen(server_tx).await }));

            let mut user = User::new(&client_config.id, client_tx);

            if client_config.id == next_client_config.id {
                let user_id = client_config.id.clone();
                user_abort_handles.push(user_set.spawn(async move {
                    user.send_loop(
                        &user_id,
                        "I'm sending a message to myself because I'm all alone :(",
                        5000,
                    )
                    .await
                }));
            } else {
                let next_user_id = next_client_config.id.clone();
                user_abort_handles.push(user_set.spawn(async move {
                    user.send_loop(&next_user_id, &format!("Hello, {}!", &next_user_id), 5000)
                        .await
                }));
            }
        }
    }

    // Handle ctrl-c and errors
    // signal::ctrl_c().await.unwrap();
    // println!("Terminating tasks");
    // for handle in client_abort_handles {
    //     handle.abort();
    // }
    // for handle in user_abort_handles {
    //     handle.abort();
    // }
    // server_abort_handle.abort();
    // directory_abort_handle.abort();
    // if let Some(handle) = metrics_abort_handle {
    //     handle.abort();
    // }
    // println!("Termination completed");
    match server.await {
        Ok(_) => println!("Server exited successfully"),
        Err(e) => eprintln!("Server exited: {e}"),
    };
    match directory.await {
        Ok(_) => println!("Directory exited successfully"),
        Err(e) => eprintln!("Directory exited: {e}"),
    };
    while let Some(res) = client_set.join_next().await {
        match res {
            Ok(_) => println!("Client exited successfully"),
            Err(e) => eprintln!("Client exited: {e}"),
        }
    }
    while let Some(res) = user_set.join_next().await {
        match res {
            Ok(_) => println!("User exited successfully"),
            Err(e) => eprintln!("User exited: {e}"),
        }
    }
    println!("done");
}
