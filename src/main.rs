mod client;
mod client_command;
mod client_send_error;
mod config;
mod directory;
mod directory_registration;
mod get_registration_error;
mod message;
mod registration_error;
mod server;
mod server_command;
mod server_registration;
mod user;

use client::Client;
use config::load_config;
use server::Server;
use tokio::task::JoinSet;
use user::User;

const DEFAULT_SERVER_BUFFER_SIZE: usize = 32;
const DEFAULT_CLIENT_BUFFER_SIZE: usize = 32;

#[tokio::main]
async fn main() {
    // Get app config
    let config_path = "./config";
    let config_env_prefix = "APPCFG";
    let config = load_config(config_path, config_env_prefix).unwrap();

    // Create server
    let server_buffer_size = if let Some(server) = config.server {
        server.buffer_size.unwrap_or(DEFAULT_SERVER_BUFFER_SIZE)
    } else {
        DEFAULT_SERVER_BUFFER_SIZE
    };
    let mut s = Server::new(server_buffer_size);
    let server_tx = s.get_tx();
    let server = tokio::spawn(async move { s.listen().await });

    // Create clients
    let mut client_set = JoinSet::new();
    let mut user_set = JoinSet::new();
    if let Some(client_configs) = config.clients {
        for (client_config, next_client_config) in client_configs
            .iter()
            .zip(client_configs.iter().cycle().skip(1))
        {
            let buffer_size = client_config
                .buffer_size
                .unwrap_or(DEFAULT_CLIENT_BUFFER_SIZE);
            let mut client = Client::new(&client_config.id, buffer_size);
            let client_tx = client.get_tx();
            let server_tx = server_tx.clone();
            client_set.spawn(async move { client.listen(server_tx).await });

            let user = User::new(client_tx);

            if client_config.id == next_client_config.id {
                let user_id = client_config.id.clone();
                user_set.spawn(async move {
                    user.send_loop(
                        &user_id,
                        "I'm sending a message to myself because I'm all alone :(",
                        1000,
                    )
                    .await
                });
            } else {
                let next_user_id = next_client_config.id.clone();
                user_set.spawn(async move {
                    user.send_loop(&next_user_id, &format!("Hello, {}!", &next_user_id), 1000)
                        .await
                });
            }
        }
    }

    // Handle errors
    match server.await {
        Ok(_) => println!("Server exited successfully"),
        Err(e) => eprintln!("{e}"),
    };
    while let Some(res) = client_set.join_next().await {
        match res {
            Ok(_) => println!("Client exited successfully"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
