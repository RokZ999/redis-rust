use std::error::Error;
use anyhow::{Result};
use tokio::net::TcpListener;
use crate::server::client_handler::handle_clients;
use crate::server::common_variables::SERVER_IP_AND_PORT;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_IP_AND_PORT).await?;
    println!("Server listening on {}", SERVER_IP_AND_PORT);
    handle_clients(listener).await?;
    Ok(())
}