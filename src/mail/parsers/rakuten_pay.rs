use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::mail::{Mail, parsers::parse_regex_first_match};

use super::{EmailParsingScheme, Transaction};

pub struct RakutenPayParsingScheme {
	pub account: String,
}

impl EmailParsingScheme for RakutenPayParsingScheme {
	fn can_parse(&self, mail: &Mail) -> bool {
		mail.subject.contains("楽天ペイアプリご利用内容確認メール")
	}

	fn parse(&self, mail: &Mail) -> Result<Option<Transaction>, Box<dyn std::error::Error>> {
		// Amount
		let amount_captures = parse_regex_first_match(&mail.body, r"決済総額\s+([0-9\,]+)", 1)?;
		if amount_captures.is_none() {
			eprintln!("No amount data found!");
			return Ok(None);
		}
		let amount_captures = amount_captures.unwrap();
		let amount_string = amount_captures.first().unwrap().to_owned();
		let amount_string = amount_string.replace(",", "");
		let amount = amount_string.parse::<u32>()?;
		let amount = Decimal::from_u32(amount);
		if amount.is_none() {
			return Err("Failed to parse amount".into());
		}
		let amount = amount.unwrap();

		// Timestamp
		let timestamp_captures = parse_regex_first_match(
			&mail.body,
			r"ご利用日時\s+([0-9]+)\/([0-9]+)\/([0-9]+)\((.)\) ([0-9]+):([0-9]+)",
			6,
		)?;
		if timestamp_captures.is_none() {
			eprintln!("No timestamp data found!");
			return Ok(None);
		}
		let timestamp_captures = timestamp_captures.unwrap();
		let timestamp_string = String::from(format!(
			"{}-{}-{} {}:{}:00",
			timestamp_captures[0],
			timestamp_captures[1],
			timestamp_captures[2],
			timestamp_captures[4],
			timestamp_captures[5]
		));
		let parsed_timestamp =
			NaiveDateTime::parse_from_str(&timestamp_string, "%Y-%m-%d %H:%M:%S")?;
		let jst_timestamp = chrono_tz::Asia::Tokyo
			.from_local_datetime(&parsed_timestamp)
			.unwrap();
		let timestamp = jst_timestamp.with_timezone(&Utc);

		// Subject
		let subject_captures = parse_regex_first_match(&mail.body, r"ご利用店舗\s+(.+)", 1)?;
		if subject_captures.is_none() {
			eprintln!("No subject data found!");
			return Ok(None);
		}
		let subject_captures = subject_captures.unwrap();
		let subject = subject_captures.first().unwrap().to_owned();

		Ok(Some(Transaction {
			subject: Some(subject),
			timestamp,
			amount: amount,
			account: self.account.clone(),
		}))
	}
}
