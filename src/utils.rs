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
    let timestamp = current_time.format("%d-%m-%Y | %H:%M:%S").to_string();

    let log_level_message = match severity {
        LogLevel::Fatal   => "FATAL",
        LogLevel::Error   => "ERROR",
        LogLevel::Warning => " WARN",
        LogLevel::Info    => " INFO",
    };

    format!("[ {} ]  => {}:  {}", timestamp, log_level_message, message)
}
