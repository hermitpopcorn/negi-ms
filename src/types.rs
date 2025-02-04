pub struct Transaction {
	pub subject: Option<String>,
	pub datetime: chrono::DateTime<chrono::Utc>,
	pub amount: rust_decimal::Decimal,
	pub account: String,
}

impl std::fmt::Debug for Transaction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"--- Transaction ---\nSubject: {}\nDatetime: {}\nAmount: {}\nAccount: {}\n-------------------",
			self.subject.as_ref().unwrap_or(&"-".to_owned()),
			self.datetime,
			self.amount,
			self.account,
		)
	}
}
