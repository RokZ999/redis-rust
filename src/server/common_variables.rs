use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::server::redis_item::RedisItem;

//Networking
pub const SERVER_IP_AND_PORT: &str = "127.0.0.1:6379";

// Types
pub type Db = Arc<Mutex<HashMap<String, RedisItem>>>;


// Command Names
pub const PING_COMMAND: &str = "PING";
pub const ECHO_COMMAND: &str = "ECHO";
pub const SET_COMMAND: &str = "SET";
pub const GET_COMMAND: &str = "GET";
pub const CONFIG_COMMAND: &str = "CONFIG";
pub const KEYS_COMMAND: &str = "KEYS";

// Command args
pub const DIR_ARG_COMMAND: &str = "dir";
pub const DB_FILENAME_ARG_COMMAND: &str = "dbfilename";
pub const PX_ARG_COMMAND: &str = "PX";

// Responses
pub const OK_STR: &str = "OK";
pub const PONG_STR: &str = "PONG";


//SPECIAL CHARACTERS
pub const CRLF: &str = "\r\n";


//SYMBOLS
pub const PLUS_CHAR: char = '+';
pub const DOLLAR_SIGN_CHAR: char = '$';
pub const ASTERISK_: char = '*';

// HEX codes
pub const VALUE_TYPE_STRING: u8 = 0x00;
pub const EXPIRE_IN_MILLISECONDS: u8 = 0xFC;
pub const EXPIRE_IN_SECONDS: u8 = 0xFD;
pub const HASH_TABLE_SELECTOR: u8 = 0xFB;
