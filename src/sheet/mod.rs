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
	pub category: String,
}

impl ValueRow {
	pub fn marked_nondup(&self) -> bool {
		return self.subject.starts_with("!");
	}

	pub fn marked_dup(&self) -> bool {
		return self.subject.starts_with("?");
	}

	pub fn subject_matches(&self, match_target: &str) -> bool {
		return self.subject.to_lowercase().contains(&match_target.to_lowercase());
	}
}

#[cfg(test)]
mod tests {
	use super::ValueRow;

	#[test]
	fn subject_matches_case_insensitive() {
		let row = ValueRow {
			row_number: 1,
			account: "Bank".to_string(),
			subject: "ちぇーストŌKaChIMaChI".to_string(),
			date_value: 0.0,
			amount: 1000,
			category: "".to_string(),
		};

		assert!(row.subject_matches("ちぇーストōkachiMACHI"));
	}
}
