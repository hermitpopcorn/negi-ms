use std::collections::HashMap;

use dotenv::dotenv;
use log::{error, info};
use negi::log::setup_logger;
use negi::sheet::ValueRow;
use negi::sheet::auth::get_sheets_client;
use negi::sheet::fetch::fetch_from_sheet;
use negi::sheet::write::mark_duplicates;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	setup_logger();

	let client = get_sheets_client().await?;
	let sheet_values = fetch_from_sheet(&client).await?;
	let grouped_map = make_grouped_map(sheet_values);
	let possible_duplicates = find_possible_duplicates(&grouped_map);

	info!("Found {} possible duplicates", possible_duplicates.len());

	if possible_duplicates.len() < 1 {
		return Ok(());
	}

	match mark_duplicates(&client, possible_duplicates).await {
		Ok(_) => info!("Marked all of them as possible duplicates"),
		Err(e) => error!("Marking error: {}", e.to_string()),
	}

	Ok(())
}

type GroupedMap = HashMap<i64, Vec<ValueRow>>;

fn make_grouped_map(values: Vec<ValueRow>) -> GroupedMap {
	let mut map = HashMap::new();

	// group rows
	for v in values {
		// skip ones marked as not duplicate
		if v.subject.starts_with("!") {
			continue;
		}

		// skip ones already marked as duplicate
		if v.subject.starts_with("?") {
			continue;
		}

		match map.get_mut(&v.amount) {
			None => {
				map.insert(v.amount, vec![v]);
			}
			Some(vector) => {
				vector.push(v);
			}
		};
	}

	// sort vector of rows by datetime
	for group in map.values_mut() {
		group.sort_by(|a, b| a.date_value.total_cmp(&b.date_value));
	}

	map
}

fn find_possible_duplicates(map: &GroupedMap) -> Vec<ValueRow> {
	let mut possible_duplicates = vec![];

	for group in map.values() {
		let mut i = 0;
		while i < group.len().saturating_sub(1) {
			if (group[i + 1].date_value - group[i].date_value).abs() <= 1.0
				&& group[i + 1].account.trim() == group[i].account.trim()
			{
				let mut cloned_duplicate = group[i + 1].clone();
				let mut original_subject = cloned_duplicate.subject;
				if original_subject.len() > 0 {
					original_subject = format!(" {}", original_subject); // prepend space if non-empty
				}
				cloned_duplicate.subject =
					format!("?dupof({}){}", group[i].row_number, original_subject);
				possible_duplicates.push(cloned_duplicate);
			}

			i += 1;
		}
	}

	possible_duplicates
}
