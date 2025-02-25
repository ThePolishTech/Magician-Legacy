use serenity::model::Colour;

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
pub fn create_log_message( message: &str, severity: LogLevel ) -> String {

    let current_time = chrono::offset::Local::now();
    let timestamp = current_time.format("%Y-%m-%d | %H:%M:%S").to_string();

    let log_level_message = match severity {
        LogLevel::Fatal   => "FATAL",
        LogLevel::Error   => "ERROR",
        LogLevel::Warning => " WARN",
        LogLevel::Info    => " INFO",
    };

    format!("[ {} ]  => {}:  {}", timestamp, log_level_message, message)
}

pub struct EmbedColours;
impl EmbedColours {
    pub const INFO:  Colour = Colour::from_rgb(0, 127, 255);
    pub const GOOD:  Colour = Colour::from_rgb(0, 255, 127);
    pub const ERROR: Colour = Colour::from_rgb(255, 127, 0);
}

