use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::ErrorInterface;
use crate::mail::{Mail, parsers::parse_regex_first_match};

use super::{EmailParsingScheme, Transaction};

pub struct RakutenPayParsingScheme {
	pub account: String,
}

#[async_trait::async_trait]
impl EmailParsingScheme for RakutenPayParsingScheme {
	fn can_parse(&self, mail: &Mail) -> bool {
		mail.subject.contains("楽天ペイアプリご利用内容確認メール")
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, ErrorInterface> {
		// Amount
		let amount_captures = parse_regex_first_match(&mail.body, r"決済総額\s+([0-9\,]+)", 1)?;
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
			r"ご利用日時\s+([0-9]+)\/([0-9]+)\/([0-9]+)\(.\) ([0-9]+):([0-9]+)",
			5,
		)?;
		let datetime_captures = datetime_captures.ok_or("No datetime data found")?;
		let datetime_string = String::from(format!(
			"{}-{}-{} {}:{}:00",
			datetime_captures[0],
			datetime_captures[1],
			datetime_captures[2],
			datetime_captures[3],
			datetime_captures[4]
		));
		let parsed_datetime = NaiveDateTime::parse_from_str(&datetime_string, "%Y-%m-%d %H:%M:%S")?;
		let jst_datetime = chrono_tz::Asia::Tokyo
			.from_local_datetime(&parsed_datetime)
			.unwrap();
		let datetime = jst_datetime.with_timezone(&Utc);

		// Subject
		let subject_captures = parse_regex_first_match(&mail.body, r"ご利用店舗\s+(.+)", 1)?;
		let subject_captures = subject_captures.ok_or("No subject data found")?;
		let subject = subject_captures.first().unwrap().to_owned();

		Ok(vec![Transaction {
			subject: Some(subject),
			datetime,
			amount,
			account: self.account.clone(),
		}])
	}
}
