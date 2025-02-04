use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::mail::{Mail, parsers::parse_regex_first_match};

use super::{EmailParsingScheme, Transaction};

pub struct OcbcPaymentNotificationScheme {
	pub account: String,
}

impl EmailParsingScheme for OcbcPaymentNotificationScheme {
	fn can_parse(&self, mail: &Mail) -> bool {
		mail.from.eq("Notifikasi OCBC <notifikasi@ocbc.id>")
			&& mail.subject.contains("Successful Payment to")
	}

	fn parse(&self, mail: &Mail) -> Result<Option<Transaction>, Box<dyn std::error::Error>> {
		// Amount
		let amount_captures = parse_regex_first_match(&mail.body, r"IDR\s+([0-9\,]+)", 1)?;
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
			r#"<b>PAYMENT DATE:<\/b><br\/>\s*<span style="color:#5f5f5f">(.+)\sWIB</span></span>"#,
			1,
		)?;
		if timestamp_captures.is_none() {
			eprintln!("No timestamp data found!");
			return Ok(None);
		}
		let timestamp_captures = timestamp_captures.unwrap();
		let timestamp_string = timestamp_captures.first().unwrap();
		let parsed_timestamp =
			NaiveDateTime::parse_from_str(&timestamp_string, "%d %b %Y %H:%M:%S")?;
		let wib_timestamp = chrono_tz::Asia::Jakarta
			.from_local_datetime(&parsed_timestamp)
			.unwrap();
		let timestamp = wib_timestamp.with_timezone(&Utc);

		// Subject
		let subject = mail.subject.trim().replace("Successful Payment to ", "");

		Ok(Some(Transaction {
			subject: Some(subject),
			timestamp,
			amount: amount,
			account: self.account.clone(),
		}))
	}
}
