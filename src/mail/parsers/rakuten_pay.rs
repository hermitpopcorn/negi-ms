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

	fn parse(&self, mail: &Mail) -> Result<Transaction, Box<dyn std::error::Error>> {
		// Amount
		let amount_captures = parse_regex_first_match(&mail.body, r"決済総額\s+([0-9\,]+)", 1)?;
		let amount_captures = amount_captures.ok_or("No amount data found")?;
		let amount_string = amount_captures
			.first()
			.ok_or("No amount data found")?
			.to_owned();
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
			r"ご利用日時\s+([0-9]+)\/([0-9]+)\/([0-9]+)\(.\) ([0-9]+):([0-9]+)",
			5,
		)?;
		let timestamp_captures = timestamp_captures.ok_or("No timestamp data found")?;
		let timestamp_string = String::from(format!(
			"{}-{}-{} {}:{}:00",
			timestamp_captures[0],
			timestamp_captures[1],
			timestamp_captures[2],
			timestamp_captures[3],
			timestamp_captures[4]
		));
		let parsed_timestamp =
			NaiveDateTime::parse_from_str(&timestamp_string, "%Y-%m-%d %H:%M:%S")?;
		let jst_timestamp = chrono_tz::Asia::Tokyo
			.from_local_datetime(&parsed_timestamp)
			.unwrap();
		let timestamp = jst_timestamp.with_timezone(&Utc);

		// Subject
		let subject_captures = parse_regex_first_match(&mail.body, r"ご利用店舗\s+(.+)", 1)?;
		let subject_captures = subject_captures.ok_or("No subject data found")?;
		let subject = subject_captures.first().unwrap().to_owned();

		Ok(Transaction {
			subject: Some(subject),
			timestamp,
			amount: amount,
			account: self.account.clone(),
		})
	}
}
