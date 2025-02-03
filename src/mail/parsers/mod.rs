use std::collections::HashMap;

use regex::Regex;

use crate::types::Transaction;

use super::Mail;

pub mod rakuten_pay;

pub trait EmailParsingScheme {
	fn can_parse(&self, mail: &Mail) -> bool;
	fn parse(&self, mail: &Mail) -> Result<Option<Transaction>, Box<dyn std::error::Error>>;
}

pub fn parse_emails(
	mails: Vec<Mail>,
	parsers: &Vec<Box<dyn EmailParsingScheme>>,
) -> Result<HashMap<Mail, Transaction>, Box<dyn std::error::Error>> {
	let mut map = HashMap::new();

	for mail in mails {
		for parser in parsers {
			if !parser.can_parse(&mail) {
				continue;
			}
			let transaction = parser.parse(&mail)?;
			if transaction.is_none() {
				continue;
			}
			let transaction = transaction.unwrap();

			#[cfg(debug_assertions)]
			println!("{:#?}", transaction);

			map.insert(mail, transaction);
			break;
		}
	}

	Ok(map)
}

fn parse_regex_first_match(
	text: &str,
	regex_literal: &str,
	capture_count: usize,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
	let mut captures_vec = vec![];

	let regex = Regex::new(regex_literal)?;
	while let Some(captures) = regex.captures_at(text, 0) {
		for i in 1..=capture_count {
			let capture = captures.get(i);
			if capture.is_none() {
				continue;
			}
			let capture = capture.unwrap();
			captures_vec.push(capture.as_str().to_owned());
		}
		break; // Break after first match
	}

	if captures_vec.len() == capture_count {
		return Ok(Some(captures_vec));
	}
	return Ok(None);
}
