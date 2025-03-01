use serenity::{futures::lock::MutexGuard, model::Colour, prelude::TypeMapKey};
use sqlx::Row;
use std::{
    collections::HashMap,
    sync::{Mutex, Arc},
};

use crate::sql_scripts;

/// Header that apppears at the top during runtime
pub const TITLE: &str = "
    // xxxxxxxxxxxxxxxxxxxxxxxx //
   //  --== MAGICIAN BOT ==--  //
  // xxxxxxxxxxxxxxxxxxxxxxxx //
";

/// Severity of a message sent to the terminal
pub enum LogLevel {
    Fatal,
    Error,
    Warning,
    Info
}

/// Create a string containing a formated log message of give severity, message, and timestamp
pub fn create_log_message( message: impl ToString, severity: LogLevel ) -> String {

    let current_time = chrono::offset::Local::now();
    let timestamp = current_time.format("%Y-%m-%d | %H:%M:%S").to_string();

    let log_level_message = match severity {
        LogLevel::Fatal   => "FATAL",
        LogLevel::Error   => "ERROR",
        LogLevel::Warning => " WARN",
        LogLevel::Info    => " INFO",
    };

    format!("[ {} ]  => {}:  {}", timestamp, log_level_message, message.to_string())
}

pub struct EmbedColours;
impl EmbedColours {
    pub const INFO:  Colour = Colour::from_rgb(0, 127, 255);
    pub const GOOD:  Colour = Colour::from_rgb(0, 255, 127);
    pub const ERROR: Colour = Colour::from_rgb(255, 127, 0);
}

/// A TypeMapKey used to access cached character information storred in a HashMap
///
/// With a key of Discord User IDs, each points to a Vector containing information about characters
/// as follows:  (character_id, character_name)
pub struct DatabaseCharactersCache;
impl TypeMapKey for DatabaseCharactersCache {
    type Value = Arc<Mutex<
        HashMap<u64, Vec<(u16, String)>>
    >>;
}

