use serde::{Deserialize, Serialize};

pub mod auth;
pub mod write;

#[derive(Serialize, Deserialize, Debug)]
struct ValueRange {
	pub range: String,
	pub values: Vec<Vec<String>>, // Use Vec<Vec<String>> for writing
}
