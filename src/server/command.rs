use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;

use crate::server::arg_handler::ArgsCli;
use crate::server::common_variables::{Db, DIR_ARG_COMMAND, DB_FILENAME_ARG_COMMAND, GET_COMMAND, OK_STR, PONG_STR, PX_ARG_COMMAND};
use crate::server::redis_item::RedisItem;
use crate::server::resp_response::RespResponse;

/// Enum representing different types of commands that can be executed by the server.
pub enum Command<'a> {
    Ping,                                        // Handles the "PING" command.
    Echo(&'a [RespResponse]),                    // Handles the "ECHO" command with arguments.
    Set(&'a [RespResponse], &'a Db),             // Handles the "SET" command with arguments and a reference to the database.
    Get(&'a [RespResponse], &'a Db),             // Handles the "GET" command with arguments and a reference to the database.
    ConfigGet(&'a [RespResponse], &'a ArgsCli),  // Handles the "CONFIG GET" command with arguments and a reference to the CLI arguments.
    Keys(&'a [RespResponse], &'a Db),            // Handles the "KEYS" command with arguments and a reference to the database.
    Unknown,                                     // Represents an unknown command.
}

impl<'a> Command<'a> {
    /// Executes the command based on the enum variant.
    ///
    /// # Returns
    ///
    /// Returns a `RespResponse` wrapped in a `Result`, which represents the response to the command.
    pub fn execute(&self) -> Result<RespResponse, anyhow::Error> {
        match self {
            Command::Ping => handle_ping_command(),                       // Execute the PING command.
            Command::Echo(args) => handle_echo_command(args),             // Execute the ECHO command.
            Command::Set(args, db) => handle_set_command(args, db),       // Execute the SET command.
            Command::Get(args, db) => handle_get_command(args, db),       // Execute the GET command.
            Command::ConfigGet(args, args_cli) => handle_config(args, args_cli), // Execute the CONFIG GET command.
            Command::Keys(args, db) => handle_keys(args, db),             // Execute the KEYS command.
            _ => Ok(RespResponse::SimpleString("-ERR unknown command".to_string())), // Handle unknown commands.
        }
    }
}

/// Handles the "PING" command.
///
/// # Returns
///
/// Returns a `RespResponse` with a "PONG" message.
fn handle_ping_command() -> Result<RespResponse, anyhow::Error> {
    Ok(RespResponse::SimpleString(PONG_STR.to_string()))
}

/// Handles the "ECHO" command.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
///
/// # Returns
///
/// Returns a `RespResponse` containing the second argument, or an empty string if none is provided.
fn handle_echo_command(args: &[RespResponse]) -> Result<RespResponse, anyhow::Error> {
    Ok(args.get(1).cloned().unwrap_or(RespResponse::SimpleString("".to_string())))
}

/// Parses the expiration time from the command arguments, if provided.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
///
/// # Returns
///
/// Returns an `Option<SystemTime>` representing the expiration time, or `None` if no expiration is provided.
fn parse_expiration(args: &[RespResponse]) -> Option<SystemTime> {
    if args.len() >= 4 {
        let expiration_type = args.get(3).unwrap().get_value().to_ascii_uppercase();
        if expiration_type.eq(PX_ARG_COMMAND) {
            if let Ok(expire_millis) = args.get(4).unwrap().get_value().parse::<u64>() {
                return Some(SystemTime::now() + Duration::from_millis(expire_millis));
            }
        }
    }
    None
}

/// Handles the "SET" command, which sets a key-value pair in the database.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
/// * `db` - A reference to the shared database.
///
/// # Returns
///
/// Returns a `RespResponse` indicating success.
fn handle_set_command(args: &[RespResponse], db: &Db) -> Result<RespResponse, anyhow::Error> {
    let set_key: String = args.get(1).unwrap().get_value();   // Retrieve the key to set.
    let set_value: String = args.get(2).unwrap().get_value(); // Retrieve the value to set.

    let expiration = parse_expiration(args);  // Parse any expiration time provided.

    // Create a `RedisItem` with or without expiration.
    let redis_item = if let Some(exp) = expiration {
        RedisItem::new_with_expiration(set_value, exp)
    } else {
        RedisItem::new(set_value)
    };

    // Insert the key-value pair into the database.
    let mut db = db.lock().unwrap();
    db.insert(set_key, redis_item);

    // Return a success response.
    Ok(RespResponse::SimpleString(OK_STR.to_string()))
}

/// Handles the "GET" command, which retrieves a value from the database.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
/// * `db` - A reference to the shared database.
///
/// # Returns
///
/// Returns a `RespResponse` containing the value or indicating that the key does not exist or is expired.
fn handle_get_command(args: &[RespResponse], db: &Db) -> Result<RespResponse, anyhow::Error> {
    let get_key: String = args.get(1).unwrap().get_value();  // Retrieve the key to get.
    let db = db.lock().unwrap();

    // Check if the key exists in the database and is not expired.
    match db.get(&get_key) {
        Some(redis_item) => {
            if redis_item.is_expired() {
                Ok(RespResponse::NullBulkString)  // Return null if the item is expired.
            } else {
                Ok(RespResponse::BulkString(redis_item.get_data().clone()))  // Return the value if not expired.
            }
        }
        None => Ok(RespResponse::NullBulkString),  // Return null if the key does not exist.
    }
}

/// Handles the "CONFIG GET" command, which retrieves configuration values.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
/// * `args_cli` - A reference to the command-line arguments.
///
/// # Returns
///
/// Returns a `RespResponse` containing the configuration value or null if the command is not recognized.
fn handle_config(args: &[RespResponse], args_cli: &ArgsCli) -> Result<RespResponse, anyhow::Error> {
    let subcommand: String = args.get(1).unwrap().get_value();  // Retrieve the subcommand (e.g., "GET").
    let get_key: String = args.get(2).unwrap().get_value();     // Retrieve the key for the configuration value.

    match subcommand.as_str() {
        GET_COMMAND => handle_config_get(get_key, args_cli),  // Handle the "GET" subcommand.
        _ => Ok(RespResponse::NullBulkString)  // Return null if the subcommand is not recognized.
    }
}

/// Retrieves specific configuration values based on the provided key.
///
/// # Arguments
///
/// * `get_key` - The key for the configuration value to retrieve.
/// * `args_cli` - A reference to the command-line arguments.
///
/// # Returns
///
/// Returns a `RespResponse` containing the configuration value or null if the key is not recognized.
fn handle_config_get(get_key: String, args_cli: &ArgsCli) -> Result<RespResponse, anyhow::Error> {
    let result = match get_key.as_str() {
        DIR_ARG_COMMAND => {
            let arg_name = RespResponse::BulkString(DIR_ARG_COMMAND.to_string());
            let arg_value = RespResponse::BulkString(args_cli.dir.clone().unwrap());
            vec![arg_name, arg_value]
        }
        DB_FILENAME_ARG_COMMAND => {
            let arg_name = RespResponse::BulkString(DB_FILENAME_ARG_COMMAND.to_string());
            let arg_value = RespResponse::BulkString(args_cli.dbfilename.clone().unwrap());
            vec![arg_name, arg_value]
        }
        _ => vec![]  // Return an empty vector if the key is not recognized.
    };

    // Return the configuration values as an array or null if not found.
    if result.is_empty() {
        Ok(RespResponse::NullBulkString)
    } else {
        Ok(RespResponse::RespArray(Arc::new(result)))
    }
}

/// Handles the "KEYS" command, which retrieves keys matching a pattern.
///
/// # Arguments
///
/// * `args` - A slice of `RespResponse` arguments.
/// * `db` - A reference to the shared database.
///
/// # Returns
///
/// Returns a `RespResponse` containing an array of matching keys or null if no matches are found.
fn handle_keys(args: &[RespResponse], db: &Db) -> Result<RespResponse, anyhow::Error> {
    let get_key_pattern: String = args.get(1).unwrap().get_value();
    let db = db.lock().unwrap();
    let mut response_array = vec![];

    match get_key_pattern.as_str() {
        "*" => {
            for (key, _) in db.iter() {
                response_array.push(
                        RespResponse::BulkString(key.clone()),
               )
            }
            Ok(RespResponse::RespArray(Arc::new(response_array)))
        }
        _ => Ok(RespResponse::NullBulkString)
    }
}