use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::anyhow;

use crate::server::arg_handler::ArgsCli;
use crate::server::common_variables::{Db, EXPIRE_IN_MILLISECONDS, EXPIRE_IN_SECONDS, HASH_TABLE_SELECTOR, VALUE_TYPE_STRING};
use crate::server::redis_item::RedisItem;

/// `RdbParser` is responsible for parsing the RDB file and populating the in-memory database.
#[derive(Debug)]
pub struct RdbParser {
    dir: String,
    db_filname: String,
    db: Db,
}


impl RdbParser {
    /// Creates a new instance of `RdbParser`.
    ///
    /// # Arguments
    ///
    /// * `args_cli` - Command-line arguments parsed into an `ArgsCli` struct.
    ///
    /// # Returns
    ///
    /// Returns `RdbParser` instance wrapped in `Result`, or an error if the directory or filename is missing.
    pub fn new(args_cli: ArgsCli) -> Self {
        RdbParser {
            dir: args_cli.dir.clone().unwrap(),
            db_filname: args_cli.dbfilename.clone().unwrap(),
            db: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    /// Populates the database by reading and parsing the RDB file.
    ///
    /// # Returns
    ///
    /// Returns the populated database wrapped in `Result`, or an error if the file couldn't be read or parsed.
    pub fn populate_database(self) -> Result<Db, anyhow::Error> {
        let file_contents = match read_file(self.dir.as_str(), self.db_filname.as_str()) {
            Ok(contents) => contents,
            Err(_) => {
                return Ok(self.db)
            }
        };

        match parse_rdb_file(file_contents) {
            Ok(db) => Ok(db),
            Err(e) => {
                return Err(anyhow!("Could not parse the file! {:?}", e))
            }
        }
    }
}

/// Reads the contents of the specified file.
///
/// # Arguments
///
/// * `dir` - Directory where the file is located.
/// * `db_filename` - Name of the file to read.
///
/// # Returns
///
/// Returns a vector of bytes wrapped in `Result`, or an error if the file could not be read.
pub fn read_file(dir: &str, db_filename: &str) -> Result<Vec<u8>, anyhow::Error> {
    let full_path = Path::new(dir).join(db_filename);
    let mut file = File::open(&full_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Parses the contents of an RDB file and returns the populated database.
///
/// # Arguments
///
/// * `contents` - Byte content of the RDB file.
///
/// # Returns
///
/// Returns the populated database wrapped in `Result`, or an error if parsing fails.
fn parse_rdb_file(contents: Vec<u8>) -> Result<Db, anyhow::Error> {
    let mut db = HashMap::new();
    let mut pos;

    pos = skip_header_metadata(&contents);
    let mut current_expiry: Option<SystemTime> = None;
    let mut global_key: Option<String> = None;
    let mut global_value: Option<String> = None;

    while pos < contents.len() {
        match contents[pos] {
            VALUE_TYPE_STRING => {
                pos += 1;
                let (key, new_pos) = get_decoded_string(&contents, pos)?;
                pos = new_pos;
                let (value, new_pos) = get_decoded_string(&contents, pos)?;
                pos = new_pos;

                global_key = Some(key);
                global_value = Some(value);

                if contents[pos] == EXPIRE_IN_MILLISECONDS || contents[pos] == EXPIRE_IN_SECONDS {
                    continue;
                }
            }
            EXPIRE_IN_MILLISECONDS => {
                pos += 1;
                let (expiry, new_pos) = get_decoded_expiry_time_ms(&contents, pos)?;
                current_expiry = Some(expiry);
                pos = new_pos;
            }
            EXPIRE_IN_SECONDS => {
                pos += 1;
                let (expiry, new_pos) = get_decoded_expiry_time_seconds(&contents, pos)?;
                current_expiry = Some(expiry);
                pos = new_pos;
            }
            _ => {
                pos += 1;
            }
        }

        if let (Some(key), Some(value)) = (global_key.take(), global_value.take()) {
            let redis_item = if let Some(expiry) = current_expiry.take() {
                RedisItem::new_with_expiration(value, expiry)
            } else {
                RedisItem::new(value)
            };

            db.insert(key.clone(), redis_item);
        }
    }

    Ok(Arc::new(Mutex::new(db)))
}

/// Skips the header metadata of the RDB file and returns the position of the first data byte.
///
/// # Arguments
///
/// * `contents` - Byte content of the RDB file.
///
/// # Returns
///
/// Returns the position of the first data byte.
fn skip_header_metadata(contents: &[u8]) -> usize {
    let mut pos: usize = 9;

    while pos < contents.len() {
        match contents[pos] {
            HASH_TABLE_SELECTOR => {
                pos += 3;
                match contents[pos] {
                    EXPIRE_IN_MILLISECONDS => {
                        pos += 1;
                        let (_, new_pos) = get_decoded_expiry_time_ms(&contents, pos).unwrap();
                        pos = new_pos;
                    }
                    EXPIRE_IN_SECONDS => {
                        pos += 1;
                        let (_, new_pos) = get_decoded_expiry_time_seconds(&contents, pos).unwrap();
                        pos = new_pos;
                    }
                    _ => {}
                }
                return pos;
            }
            _ => pos += 1
        }
    }
    pos
}


/// Decodes an expiry time in milliseconds from the RDB file contents.
///
/// # Arguments
///
/// * `contents` - Byte content of the RDB file.
/// * `pos` - Current position in the byte content.
///
/// # Returns
///
/// Returns the expiry time and the new position wrapped in `Result`, or an error if decoding fails.
fn get_decoded_expiry_time_ms(contents: &[u8], pos: usize) -> Result<(SystemTime, usize), anyhow::Error> {
    if contents.len() < pos + 8 {
        return Err(anyhow!("Insufficient bytes for millisecond expiry"));
    }
    let millis = u64::from_le_bytes([
        contents[pos], contents[pos + 1], contents[pos + 2], contents[pos + 3],
        contents[pos + 4], contents[pos + 5], contents[pos + 6], contents[pos + 7]
    ]);
    let expiry = UNIX_EPOCH + Duration::from_millis(millis);
    Ok((expiry, pos + 8))
}

/// Decodes an expiry time in seconds from the RDB file contents.
///
/// # Arguments
///
/// * `contents` - Byte content of the RDB file.
/// * `pos` - Current position in the byte content.
///
/// # Returns
///
/// Returns the expiry time and the new position wrapped in `Result`, or an error if decoding fails.
fn get_decoded_expiry_time_seconds(contents: &[u8], pos: usize) -> Result<(SystemTime, usize), anyhow::Error> {
    if contents.len() < pos + 4 {
        return Err(anyhow!("Insufficient bytes for second expiry"));
    }
    let seconds = u32::from_le_bytes([
        contents[pos], contents[pos + 1], contents[pos + 2], contents[pos + 3]
    ]);
    let expiry = UNIX_EPOCH + Duration::from_secs(seconds as u64);
    Ok((expiry, pos + 4))
}


/// Decodes a string from the RDB file contents.
///
/// # Arguments
///
/// * `contents` - Byte content of the RDB file.
/// * `pos` - Current position in the byte content.
///
/// # Returns
///
/// Returns the decoded string and the new position wrapped in `Result`, or an error if decoding fails.
fn get_decoded_string(contents: &[u8], pos: usize) -> Result<(String, usize), std::string::FromUtf8Error> {
    let string_size = contents[pos] as usize;
    let string_slice = &contents[pos + 1..pos + 1 + string_size];

    let decoded_string = String::from_utf8(string_slice.to_vec())?;

    Ok((decoded_string, pos + 1 + string_size))
}

