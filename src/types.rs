pub struct Transaction {
	pub subject: Option<String>,
	pub timestamp: chrono::DateTime<chrono::Utc>,
	pub amount: rust_decimal::Decimal,
}

impl std::fmt::Debug for Transaction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"--- Transaction ---\nSubject: {}\nTimestamp: {}\nAmount: {}\n-------------------",
			self.subject.as_ref().unwrap_or(&"-".to_owned()),
			self.timestamp,
			self.amount
		)
	}
}
