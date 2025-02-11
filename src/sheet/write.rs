use std::env;

use reqwest::Client;

use crate::{sheet::ValueRange, transaction::Transaction};

pub async fn append_to_sheet(
	client: &Client,
	transactions: Vec<Transaction>,
) -> Result<(), Box<dyn std::error::Error>> {
	let spreadsheet_id = env::var("SPREADSHEET_ID")?;
	let range = "Transactions!A:D";
	let url = format!(
		"https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
		spreadsheet_id, range
	);

	let mut value_range = ValueRange {
		range: range.to_string(),
		values: vec![],
	};

	for transaction in transactions {
		let row = vec![
			transaction.account.clone(),
			transaction.subject.clone().unwrap_or("".to_string()),
			transaction.datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
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
		Ok(_) => return Ok(()),
		Err(e) => return Err(e.into()),
	}
}
