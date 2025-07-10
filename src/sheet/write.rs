use std::env;

use log::error;
use reqwest::Client;

use crate::ErrorInterface;
use crate::{sheet::ValueRange, transaction::Transaction};

use super::ValueRow;

pub async fn append_to_sheet(
	client: &Client,
	transactions: Vec<Transaction>,
) -> Result<(), ErrorInterface> {
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
			transaction.account.trim().to_string(),
			transaction
				.subject
				.unwrap_or("".to_string())
				.trim()
				.to_string(),
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

pub async fn mark_duplicates_in_sheet(
	client: &Client,
	rows: Vec<ValueRow>,
) -> Result<(), ErrorInterface> {
	let spreadsheet_id = env::var("SPREADSHEET_ID")?;

	let mut successful_updates = 0;
	let total_rows = rows.len();

	for row in rows {
		let make_url = |range: &str| -> String {
			format!(
				"https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED&includeValuesInResponse=0",
				spreadsheet_id, range
			)
		};

		let write_subject = {
			let range = format!("Transactions!B{}:B{}", row.row_number, row.row_number);
			let url = make_url(&range);
			let value_range = ValueRange {
				range,
				values: vec![vec![row.subject]],
			};

			let response = client
				.put(&url)
				.body(serde_json::to_string(&value_range)?)
				.send()
				.await?;

			response.error_for_status()
		};

		if write_subject.is_err() {
			error!(
				"Could not update subject for row {}. Error: {}",
				row.row_number,
				write_subject.unwrap_err()
			);
			continue;
		}

		let write_amount = {
			let range = format!("Transactions!D{}:D{}", row.row_number, row.row_number);
			let url = make_url(&range);
			let value_range = ValueRange {
				range,
				values: vec![vec!["0".to_string()]],
			};

			let response = client
				.put(&url)
				.body(serde_json::to_string(&value_range)?)
				.send()
				.await?;

			response.error_for_status()
		};

		if write_amount.is_err() {
			error!(
				"Could not update amount for row {}. Error: {}",
				row.row_number,
				write_amount.unwrap_err()
			);
			continue;
		}

		successful_updates += 1;
	}

	#[cfg(debug_assertions)]
	{
		use log::debug;
		debug!(
			"Updated rows: {}, total rows: {}",
			successful_updates, total_rows
		);
	}

	match successful_updates == total_rows {
		true => return Ok(()),
		false => {
			return Err(format!(
				"Failed to update {} out of {} rows",
				total_rows - successful_updates,
				total_rows
			)
			.into());
		}
	}
}
