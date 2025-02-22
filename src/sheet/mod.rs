use serde::{Deserialize, Serialize};

pub mod auth;
pub mod fetch;
pub mod write;

#[derive(Serialize, Deserialize, Debug)]
struct ValueRange {
	pub range: String,
	pub values: Vec<Vec<String>>, // Use Vec<Vec<String>> for writing
}

#[derive(Debug, Clone)]
pub struct ValueRow {
	pub row_number: usize,
	pub account: String,
	pub subject: String,
	pub date_value: f64,
	pub amount: i64,
}
