use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use crate::server::arg_handler::{ArgHandler, ArgsCli};
use crate::server::command_handler::CommandHandler;
use crate::server::common_variables::Db;
use crate::server::rdb_parser::RdbParser;

/// Handles incoming client connections on the provided `TcpListener`.
///
/// This function listens for incoming connections and spawns a new task to handle each client.
/// It also initializes the database, either by loading data from an RDB file or creating a new, empty database.
///
/// # Arguments
///
/// * `listener` - A `TcpListener` that listens for incoming client connections.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if it fails.
///
/// # Examples
///
/// ```
/// let listener = TcpListener::bind("127.0.0.1:6379").await?;
/// handle_clients(listener).await?;
/// ```
pub async fn handle_clients(listener: TcpListener) -> Result<(), Box<dyn Error>> {
    // Retrieve command-line arguments.
    let retrieved_args = ArgHandler::retrieve_args();
    let db: Db;

    // Check if the necessary arguments are provided and populate the database if possible.
    if retrieved_args.can_be_parsed() {
        let rdb = RdbParser::new(retrieved_args.clone());
        db = rdb.populate_database()?
    } else {
        // If arguments are not provided, initialize an empty in-memory database.
        db = Arc::new(Mutex::new(HashMap::new()))
    }

    loop {
        // Accept a new client connection.
        let (socket, addr) = listener.accept().await?;
        println!("New client: {addr:?}");

        // Clone the database and command-line arguments to be used in the client handler.
        let db = db.clone();
        let cli_args = retrieved_args.clone();

        // Spawn a new task to handle the client asynchronously.
        tokio::spawn(async move {
            if let Err(e) = process_client(socket, db, cli_args).await {
                eprintln!("Error processing client: {e}");
            }
        });
    }
}

/// Processes an individual client's commands.
///
/// This function splits the TCP stream into a reader and a writer, then creates a `CommandHandler`
/// to process the client's commands asynchronously.
///
/// # Arguments
///
/// * `stream` - The `TcpStream` representing the client's connection.
/// * `db` - The shared database instance.
/// * `cli_args` - The command-line arguments.
///
/// # Returns
///
/// Returns `Ok(())` if the client was successfully processed, or an error if something went wrong.
///
/// # Examples
///
/// ```
/// let stream = TcpStream::connect("127.0.0.1:6379").await?;
/// process_client(stream, db, cli_args).await?;
/// ```
pub async fn process_client(stream: TcpStream, db: Db, cli_args: ArgsCli) -> Result<(), anyhow::Error> {
    // Split the TCP stream into a reader and writer for asynchronous I/O.
    let (reader, writer) = io::split(stream);

    // Create a new CommandHandler to manage the client's commands.
    let mut handler = CommandHandler::new(reader, writer, db, cli_args);

    // Run the CommandHandler to process the client's commands.
    handler.run().await
}
