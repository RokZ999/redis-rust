use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::server::arg_handler::ArgsCli;
use crate::server::command::Command;
use crate::server::common_variables::{CONFIG_COMMAND, Db, ECHO_COMMAND, GET_COMMAND, KEYS_COMMAND, PING_COMMAND, SET_COMMAND};
use crate::server::resp_response::{parse_message, RespResponse};

/// `CommandHandler` is responsible for processing client commands received over a TCP connection.
pub struct CommandHandler {
    reader: BufReader<ReadHalf<TcpStream>>,  // Buffered reader for reading from the TCP stream.
    writer: WriteHalf<TcpStream>,            // Writer for sending responses back to the client.
    db: Db,                                  // Reference to the shared database.
    args_cli: ArgsCli,                       // Command-line arguments passed to the server.
}

impl CommandHandler {
    /// Creates a new `CommandHandler`.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reading half of the TCP stream.
    /// * `writer` - The writing half of the TCP stream.
    /// * `db` - Shared database instance.
    /// * `args_cli` - Command-line arguments for the server.
    pub fn new(reader: ReadHalf<TcpStream>, writer: WriteHalf<TcpStream>, db: Db, args_cli: ArgsCli) -> Self {
        CommandHandler {
            reader: BufReader::new(reader),  // Wrap the reader in a `BufReader` for efficient reading.
            writer,
            db,
            args_cli,
        }
    }

    /// Runs the command handler, continuously reading commands from the client and processing them.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the client disconnects or an error occurs.
    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut buffer = [0; 1024];  // Buffer for storing incoming data.

        loop {
            // Read data from the client into the buffer.
            let bytes_read = self.reader.read(&mut buffer).await?;

            // If no data was read, the client has disconnected.
            if bytes_read == 0 {
                return Ok(());
            }

            // Convert the received data to a UTF-8 string, handling any errors.
            let client_command = std::str::from_utf8(&buffer[..bytes_read])
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in command"))?;

            // Process the client's command.
            self.process_client_command(client_command).await?;

            // Flush the writer to ensure the response is sent to the client.
            self.writer.flush().await?;
        }
    }

    /// Processes a single client command.
    ///
    /// # Arguments
    ///
    /// * `client_command` - The command received from the client as a string slice.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the command was successfully processed, or an error if it failed.
    async fn process_client_command(&mut self, client_command: &str) -> Result<(), anyhow::Error> {
        // Parse the command and its arguments from the client's input.
        let (command, args) = CommandHandler::get_command_with_args(client_command).unwrap();

        // Handle the command and generate a response.
        let response = self.handle_command(&command, &args).unwrap();

        // Send the response back to the client.
        self.print_to_client(response).await
    }

    /// Parses a command and its arguments from the client's input.
    ///
    /// # Arguments
    ///
    /// * `client_command` - The command string received from the client.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the command as a `String` and the arguments as an `Arc<Vec<RespResponse>>`.
    fn get_command_with_args(client_command: &str) -> Result<(String, Arc<Vec<RespResponse>>)> {
        // Parse the RESP message from the client command.
        let (resp, _) = parse_message(client_command)?;

        // Extract the command and arguments from the parsed message.
        resp.get_command_and_args()
    }

    /// Sends a response back to the client.
    ///
    /// # Arguments
    ///
    /// * `value` - The response to be sent to the client as a `RespResponse`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the response was successfully sent, or an error if it failed.
    async fn print_to_client(&mut self, value: RespResponse) -> Result<(), anyhow::Error> {
        // Serialize the response and write it to the client.
        Ok(self.writer.write_all(value.serialize().as_bytes()).await?)
    }

    /// Handles the client's command by mapping it to a known command and executing it.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string received from the client.
    /// * `args` - The arguments associated with the command as a slice of `RespResponse`.
    ///
    /// # Returns
    ///
    /// Returns the response to the command as a `RespResponse`, or an error if the command failed.
    fn handle_command(
        &self,
        command: &str,
        args: &[RespResponse],
    ) -> Result<RespResponse, anyhow::Error> {
        // Convert the command to uppercase for case-insensitive matching.
        let command_name = command.to_ascii_uppercase();

        // Match the command name to a known command, creating a `Command` object.
        let prepared_command: Command = match command_name.as_str() {
            PING_COMMAND => Command::Ping,
            ECHO_COMMAND => Command::Echo(args),
            SET_COMMAND => Command::Set(args, &self.db),
            GET_COMMAND => Command::Get(args, &self.db),
            CONFIG_COMMAND => Command::ConfigGet(args, &self.args_cli),
            KEYS_COMMAND => Command::Keys(args, &self.db),
            _ => Command::Unknown,
        };

        // Execute the matched command and return the result.
        prepared_command.execute()
    }
}
