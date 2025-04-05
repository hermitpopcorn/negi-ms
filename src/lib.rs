pub mod log;
pub mod mail;
pub mod sheet;
pub mod transaction;

pub type ErrorInterface = Box<dyn std::error::Error + Send + Sync>;
