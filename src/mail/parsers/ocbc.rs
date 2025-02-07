use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::mail::{Mail, parsers::parse_regex_first_match};

use super::{EmailParsingScheme, Transaction};

pub struct OcbcPaymentNotificationScheme {
	pub account: String,
}

#[async_trait::async_trait]
impl EmailParsingScheme for OcbcPaymentNotificationScheme {
	fn can_parse(&self, mail: &Mail) -> bool {
		mail.from.eq("Notifikasi OCBC <notifikasi@ocbc.id>")
			&& mail.subject.contains("Successful Payment to")
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
		// Amount
		let amount_captures = parse_regex_first_match(&mail.body, r"IDR\s+([0-9\,]+)", 1)?;
		let amount_captures = amount_captures.ok_or("No amount data found")?;
		let amount_string = amount_captures
			.first()
			.ok_or("No amount data found")?
			.to_owned();
		let amount_string = amount_string.replace(",", "");
		let amount = amount_string.parse::<u32>()?;
		let mut amount = Decimal::from_u32(amount).ok_or("Failed to parse amount")?;
		amount.set_sign_negative(true);

		// Datetime
		let datetime_captures = parse_regex_first_match(
			&mail.body,
			r#"<b>PAYMENT DATE:<\/b><br\/>\s*<span style="color:#5f5f5f">(.+)\sWIB</span></span>"#,
			1,
		)?;
		let datetime_captures = datetime_captures.ok_or("No datetime data found")?;
		let datetime_string = datetime_captures.first().ok_or("No datetime data found")?;
		let parsed_datetime = NaiveDateTime::parse_from_str(&datetime_string, "%d %b %Y %H:%M:%S")?;
		let wib_datetime = chrono_tz::Asia::Jakarta
			.from_local_datetime(&parsed_datetime)
			.unwrap();
		let datetime = wib_datetime.with_timezone(&Utc);

		// Subject
		let subject = mail.subject.trim().replace("Successful Payment to ", "");

		Ok(vec![Transaction {
			subject: Some(subject),
			datetime,
			amount,
			account: self.account.clone(),
		}])
	}
}
