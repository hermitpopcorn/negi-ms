use chrono::{NaiveDateTime, TimeZone, Utc};
use regex::Regex;
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::{ErrorInterface, mail::Mail};

use super::{EmailParsingScheme, Transaction};

pub struct RakutenCardParsingScheme {
	pub account: String,
}

impl RakutenCardParsingScheme {
	fn in_skip_list(&self, _: &str) -> bool {
		return false;
	}
}

#[async_trait::async_trait]
impl EmailParsingScheme for RakutenCardParsingScheme {
	fn can_parse(&self, mail: &Mail) -> bool {
		mail.subject.contains("カード利用のお知らせ")
			&& mail.from.contains("info@mail.rakuten-card.co.jp")
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, ErrorInterface> {
		let mut transactions = vec![];

		let regex = Regex::new(
			"■利用日: ([0-9/]+)\n■利用先: (.+)\n■利用者: 本人\n■支払方法: [0-9]*回\n■利用金額: ([0-9,]+) 円\n■支払月: [0-9/]+",
		)?;
		for captures in regex.captures_iter(&mail.body) {
			let transaction: Result<Option<Transaction>, ErrorInterface> = 'parseOne: {
				// Subject
				let subject = captures
					.get(2)
					.ok_or("No subject data found")?
					.as_str()
					.to_owned();
				if self.in_skip_list(&subject) {
					break 'parseOne Ok(None);
				}

				// Datetime
				let datetime_string = captures
					.get(1)
					.ok_or("No datetime data found")?
					.as_str()
					.to_owned();
				let datetime_string = datetime_string + " 00:00:00";
				let parsed_datetime =
					NaiveDateTime::parse_from_str(&datetime_string, "%Y/%m/%d %H:%M:%S")?;
				let jst_datetime = chrono_tz::Asia::Tokyo
					.from_local_datetime(&parsed_datetime)
					.unwrap();
				let datetime = jst_datetime.with_timezone(&Utc);

				// Amount
				let amount_string: String = captures
					.get(3)
					.ok_or("No amount data found")?
					.as_str()
					.to_owned();
				let amount_string = amount_string.replace(",", "");
				let amount = amount_string.parse::<u32>()?;
				let mut amount = Decimal::from_u32(amount).ok_or("Failed to parse amount")?;
				amount.set_sign_negative(true);

				Ok(Some(Transaction {
					subject: Some(subject),
					datetime,
					amount,
					account: self.account.clone(),
				}))
			};

			match transaction? {
				Some(t) => transactions.push(t),
				None => continue,
			}
		}

		Ok(transactions)
	}
}
