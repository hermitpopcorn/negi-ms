use std::{collections::HashMap, env};

use reqwest::Client;

use crate::{mail::Mail, sheet::ValueRange, types::Transaction};

pub async fn append_to_sheet(
	client: &Client,
	transactions: HashMap<Mail, Transaction>,
) -> Result<HashMap<Mail, Transaction>, Box<dyn std::error::Error>> {
	let spreadsheet_id = env::var("SPREADSHEET_ID")?;
	let range = "Transactions!A:C";
	let url = format!(
		"https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
		spreadsheet_id, range
	);

	let mut value_range = ValueRange {
		range: range.to_string(),
		values: vec![],
	};

	for transaction in transactions.values() {
		let row = vec![
			transaction.subject.clone().unwrap_or("".to_string()),
			transaction
				.timestamp
				.format("%Y-%m-%d %H:%M:%S")
				.to_string(),
			transaction.amount.to_string(),
		];
		value_range.values.push(row);
	}

	let response = client
		.post(&url)
		.body(serde_json::to_string(&value_range)?)
		.send()
		.await?;

	match response.error_for_status_ref() {
		Ok(_) => return Ok(transactions),
		Err(e) => return Err(e.into()),
	}
}
