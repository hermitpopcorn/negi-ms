use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use dotenv::dotenv;
use log::{error, info, warn};
use negi::ErrorInterface;
use negi::log::setup_logger;
use negi::sheet::ValueRow;
use negi::sheet::auth::get_sheets_client;
use negi::sheet::fetch::fetch_from_sheet;
use negi::sheet::write::{mark_duplicates_in_sheet, set_categories_in_sheet};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), ErrorInterface> {
	dotenv().ok();
	setup_logger();

	let client = get_sheets_client().await?;
	let sheet_values = fetch_from_sheet(&client).await?;

	mark_duplicates(&client, sheet_values.clone()).await;
	set_categories(&client, sheet_values).await;

	Ok(())
}

async fn mark_duplicates(client: &Client, values: Vec<ValueRow>) {
	let grouped_map = make_grouped_map(values);
	let possible_duplicates: Vec<ValueRow> = find_possible_duplicates(&grouped_map);

	info!("Found {} possible duplicates", possible_duplicates.len());

	if possible_duplicates.len() < 1 {
		return;
	}

	match mark_duplicates_in_sheet(&client, possible_duplicates).await {
		Ok(_) => info!("Marked all of them as possible duplicates"),
		Err(e) => error!("Marking error: {}", e.to_string()),
	};
}

type GroupedMap = HashMap<i64, Vec<ValueRow>>;

fn make_grouped_map(values: Vec<ValueRow>) -> GroupedMap {
	let mut map = HashMap::new();

	// group rows
	for v in values {
		// skip ones already marked as duplicate
		if v.marked_dup() {
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

struct DuplicateIndexes {
	original: usize,
	suspected_duplicate: usize,
}

fn should_flip_by_time(current: &ValueRow, next: &ValueRow) -> bool {
	let current_time_of_day = current.date_value.fract() * 86400.0;
	let next_time_of_day = next.date_value.fract() * 86400.0;

	let current_minutes = (current_time_of_day as i64 / 60) % 60;
	let current_seconds = (current_time_of_day as i64) % 60;
	let next_minutes = (next_time_of_day as i64 / 60) % 60;
	let next_seconds = (next_time_of_day as i64) % 60;

	current_minutes == 0 && current_seconds == 0 && (next_minutes != 0 || next_seconds != 0)
}

fn find_possible_duplicates(map: &GroupedMap) -> Vec<ValueRow> {
	let mut possible_duplicates = vec![];

	for group in map.values() {
		let group: &Vec<ValueRow> = group;

		let mut i = 0;
		while i < group.len().saturating_sub(1) {
			let current_item = &group[i];
			let next_item = &group[i + 1];

			// if not 2 days apart and same account
			if (next_item.date_value - current_item.date_value).abs() <= 2.0
				&& next_item.account.trim() == current_item.account.trim()
			{
				// skip if both marked as not duplicate
				if current_item.marked_nondup() && next_item.marked_nondup() {
					i += 1;
					continue;
				}

				// if only the latter is marked as not duplicate, or the earlier row is at HH:00:00,
				// flip the original/suspected-duplicate decision
				// (we prefer exact-timed transactions over transactions dated at HH:00:00)
				let flip = {
					if !current_item.marked_nondup() && next_item.marked_nondup() {
						true
					} else if should_flip_by_time(&current_item, &next_item) {
						true
					} else {
						false
					}
				};

				let indexes = DuplicateIndexes {
					original: if flip { i + 1 } else { i },
					suspected_duplicate: if flip { i } else { i + 1 },
				};

				let mut cloned_duplicate = group[indexes.suspected_duplicate].clone();
				let mut original_subject = cloned_duplicate.subject;
				if original_subject.len() > 0 {
					original_subject = format!(" {}", original_subject); // prepend space if non-empty
				}
				cloned_duplicate.subject = format!(
					"?dupof({}){}",
					group[indexes.original].row_number, original_subject
				);
				possible_duplicates.push(cloned_duplicate);
			}

			i += 1;
		}
	}

	possible_duplicates
}

async fn set_categories(client: &Client, values: Vec<ValueRow>) {
	let category_map = read_category_map();
	if category_map.is_err() {
		let error = category_map.unwrap_err();
		error!("Could not open category map: {}", error);
		return;
	}
	let category_map = category_map.unwrap();

	let matched_values = match_subject_to_categories(values, &category_map);

	info!("Found {} subject-to-category matches", matched_values.len());

	if matched_values.len() < 1 {
		return;
	}

	match set_categories_in_sheet(&client, matched_values).await {
		Ok(_) => info!("Marked the categories for all of them"),
		Err(e) => error!("Marking error: {}", e.to_string()),
	};
}

type CategoryMap = HashMap<String, String>;

fn read_category_map() -> Result<CategoryMap, ErrorInterface> {
	let category_map_file_path = env::var("CATEGORY_MAP_FILE")?;
	let path = Path::new(&category_map_file_path);
	let file = File::open(path)?;
	let reader = BufReader::new(file);

	let mut map: CategoryMap = HashMap::new();

	for (line_num, line_result) in reader.lines().enumerate() {
		let line = line_result?;
		let trimmed_line = line.trim();

		if trimmed_line.is_empty() {
			continue;
		}

		let parts: Vec<&str> = trimmed_line.split(',').collect();

		if parts.len() != 2 {
			warn!("Line {} has expected number of items", line_num + 1);
			continue;
		}

		map.insert(parts[0].to_string(), parts[1].to_string());
	}

	Ok(map)
}

fn match_subject_to_categories(values: Vec<ValueRow>, category_map: &CategoryMap) -> Vec<ValueRow> {
	values
		.into_iter()
		// filter out items already having a category and those without subjects
		.filter(|i| i.subject.len() > 0 && i.category.len() < 1)
		// set the category if subject contains the keyword
		.map(|mut i| {
			for (k, v) in category_map.iter() {
				if !i.subject_matches(k) {
					continue;
				}

				i.category = v.clone();
			}

			i
		})
		// filter out non-matches
		.filter(|i| i.category.len() > 0)
		.collect::<Vec<ValueRow>>()
}

#[cfg(test)]
mod tests {
	use super::ValueRow;
	use super::should_flip_by_time;

	#[test]
	fn flips_when_earlier_row_is_on_the_hour_and_later_row_has_nonzero_minutes_or_seconds() {
		const CURRENT_DATE_VALUE: f64 = 1.5; // 12:00:00 in a serialized Excel-style day value
		const NEXT_DATE_VALUE: f64 = 1.5006944444444445; // 12:00:30 in the same scale

		let current = ValueRow {
			row_number: 2,
			account: "Bank".to_string(),
			subject: "".to_string(),
			date_value: CURRENT_DATE_VALUE,
			amount: 1000,
			category: "".to_string(),
		};
		let next = ValueRow {
			row_number: 3,
			account: "Bank".to_string(),
			subject: "".to_string(),
			date_value: NEXT_DATE_VALUE,
			amount: 1000,
			category: "".to_string(),
		};

		assert!(should_flip_by_time(&current, &next));
	}
}
