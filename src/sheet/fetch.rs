use std::env;

use reqwest::Client;
use serde::Deserialize;

use crate::{ErrorInterface, sheet::ValueRow};

#[derive(Deserialize, Debug)]
struct ResponseFormat {
	values: Vec<serde_json::Value>,
}

pub async fn fetch_from_sheet(client: &Client) -> Result<Vec<ValueRow>, ErrorInterface> {
	let spreadsheet_id = env::var("SPREADSHEET_ID")?;
	let range = "Transactions!A2:D";
	let url = format!(
		"https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueRenderOption=UNFORMATTED_VALUE",
		spreadsheet_id, range
	);

	let response = client.get(&url).send().await?;

	if response.error_for_status_ref().is_err() {
		return Err(response
			.error_for_status_ref()
			.err()
			.unwrap()
			.to_string()
			.into());
	}

	let response_text = response.text().await?;

	#[cfg(debug_assertions)]
	{
		use log::debug;
		debug!("Sheet fetch response: {}", response_text);
	}

	let response_json = serde_json::from_str::<ResponseFormat>(&response_text)?;

	let mut row_number: usize = 1; // start from A2
	let values: Vec<ValueRow> = response_json
		.values
		.iter()
		.map(|i| {
			row_number += 1;
			ValueRow {
				row_number,
				account: i[0].as_str().unwrap_or("").to_owned(),
				subject: i[1].as_str().unwrap_or("").to_owned(),
				date_value: i[2].as_f64().unwrap_or(0.0),
				amount: i[3].as_i64().unwrap_or(0),
			}
		})
		.collect();

	Ok(values)
}
