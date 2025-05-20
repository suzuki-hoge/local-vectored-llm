use chrono::Local;
use colored::*;
use std::fmt;

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "{}", "INFO".green()),
            LogLevel::Warn => write!(f, "{}", "WARN".yellow()),
            LogLevel::Error => write!(f, "{}", "ERROR".red()),
        }
    }
}

pub fn log(level: LogLevel, message: String) {
    let now = Local::now();
    println!("[{}] {} {}", now.format("%Y-%m-%d %H:%M:%S"), level, message);
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Info, format!($($arg)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Warn, format!($($arg)*))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Error, format!($($arg)*))
    }
}
