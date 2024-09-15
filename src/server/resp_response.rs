use std::sync::Arc;
use anyhow::Result;
use crate::server::common_variables::{ASTERISK_, CRLF, DOLLAR_SIGN_CHAR, PLUS_CHAR};
use crate::server::resp_response::RespResponse::SimpleString;

/// `RespResponse` represents different types of Redis Serialization Protocol (RESP) responses.
#[derive(Debug, Clone)]
pub enum RespResponse {
    SimpleString(String),                   // A simple string response (e.g., "+OK\r\n").
    BulkString(String),                     // A bulk string response (e.g., "$6\r\nfoobar\r\n").
    RespArray(Arc<Vec<RespResponse>>),      // An array of RESP responses.
    NullBulkString,                         // A null bulk string (e.g., "$-1\r\n").
}

impl RespResponse {
    /// Serializes the `RespResponse` into a string format according to the RESP specification.
    ///
    /// # Returns
    ///
    /// Returns the serialized string representing the `RespResponse`.
    pub fn serialize(&self) -> String {
        match self {
            SimpleString(s) => format!("+{}\r\n", s),  // Serialize a simple string.
            RespResponse::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),  // Serialize a bulk string.
            RespResponse::RespArray(arr) => {
                let mut array_join = String::new();
                array_join.push_str(&format!("*{}\r\n", arr.len()));  // Start with the array length.
                for resp in arr.iter() {
                    array_join.push_str(&resp.serialize());  // Serialize each element in the array.
                }
                array_join
            }
            RespResponse::NullBulkString => format!("${}\r\n", "-1".to_string())  // Serialize a null bulk string.
        }
    }

    /// Extracts the command and its arguments from a `RespResponse`.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the command as a `String` and the arguments as an `Arc<Vec<RespResponse>>`.
    pub fn get_command_and_args(self) -> Result<(String, Arc<Vec<RespResponse>>)> {
        match self {
            SimpleString(s) => {
                Ok((s, Arc::new(Vec::new())))  // If it's a simple string, treat it as a command with no arguments.
            },
            RespResponse::RespArray(arr) if !arr.is_empty() => {
                if let RespResponse::BulkString(cmd) = &arr[0] {
                    Ok((cmd.clone(), Arc::clone(&arr)))  // The first element is the command, and the rest are arguments.
                } else {
                    Err(anyhow::anyhow!("First element in array is not a command string"))
                }
            },
            _ => Err(anyhow::anyhow!("Not a valid command or array")),  // Handle invalid cases.
        }
    }

    /// Retrieves the value from a `RespResponse` as a `String`.
    ///
    /// # Returns
    ///
    /// Returns the value as a `String`. Panics if the response type is not a string.
    pub fn get_value(&self) -> String {
        match self {
            SimpleString(s) => s.to_string(),  // Return the value if it's a simple string.
            RespResponse::BulkString(s) => s.to_string(),  // Return the value if it's a bulk string.
            _ => panic!("Not implemented")  // Panic for unimplemented cases.
        }
    }
}

/// Parses a RESP message from a string.
///
/// # Arguments
///
/// * `command` - The command string to parse.
///
/// # Returns
///
/// Returns a tuple containing the parsed `RespResponse` and the length of the command.
pub fn parse_message(command: &str) -> Result<(RespResponse, i32)> {
    match command.as_bytes()[0] as char {
        PLUS_CHAR => parse_simple_string(command),  // Handle simple strings.
        DOLLAR_SIGN_CHAR => parse_bulk_string(command),  // Handle bulk strings.
        ASTERISK_ => parse_array(command),  // Handle arrays.
        _ => Ok((SimpleString("-ERR unknown command".to_string()), 0)),  // Return an error for unknown commands.
    }
}

/// Parses a simple string from a RESP command.
///
/// # Arguments
///
/// * `command` - The command string to parse.
///
/// # Returns
///
/// Returns a tuple containing the parsed `RespResponse` and the length of the string.
fn parse_simple_string(command: &str) -> Result<(RespResponse, i32)> {
    let data: String = command[1..].to_string();  // Extract the data from the command.
    Ok((SimpleString(data.clone()), data.len() as i32))  // Return the data as a `SimpleString`.
}

/// Parses a bulk string from a RESP command.
///
/// # Arguments
///
/// * `command` - The command string to parse.
///
/// # Returns
///
/// Returns a tuple containing the parsed `RespResponse` and the length of the string.
pub fn parse_bulk_string(command: &str) -> Result<(RespResponse, i32), anyhow::Error> {
    let parts: Vec<&str> = command[1..].split(CRLF).collect();  // Split the command by CRLF.
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid RESP bulk string format"));  // Return an error if the format is invalid.
    }

    let length: i32 = parts[0].parse().map_err(|e| anyhow::anyhow!("Failed to parse length: {}", e))?;  // Parse the length of the bulk string.
    let data: String = parts[1].to_string();  // Extract the data.

    Ok((RespResponse::BulkString(data), length))  // Return the data as a `BulkString`.
}

/// Parses an array from a RESP command.
///
/// # Arguments
///
/// * `command` - The command string to parse.
///
/// # Returns
///
/// Returns a tuple containing the parsed `RespResponse` and the size of the array.
pub fn parse_array(command: &str) -> Result<(RespResponse, i32)> {
    let parts: Vec<&str> = command[1..].split(CRLF).collect();  // Split the command by CRLF.
    let arr_size: i32 = parts[0].parse().map_err(|e| anyhow::anyhow!("Failed to parse array size: {}", e))?;  // Parse the size of the array.

    let mut responses = Vec::with_capacity(arr_size as usize);  // Prepare a vector to hold the array elements.
    let mut index = 1;

    for _ in 0..arr_size {
        if index >= parts.len() {
            return Err(anyhow::anyhow!("Unexpected end of input while parsing array elements"));  // Return an error if the input is incomplete.
        }

        let element_str = parts[index].to_string() + CRLF + parts[index + 1];  // Reconstruct the element string.
        let (response, _) = parse_message(&element_str)?;  // Parse the element as a RESP message.
        responses.push(response);  // Add the parsed element to the array.

        index += 2;  // Move to the next element.
    }

    Ok((RespResponse::RespArray(Arc::new(responses)), arr_size))  // Return the parsed array.
}
