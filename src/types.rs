pub struct Transaction {
	pub subject: Option<String>,
	pub timestamp: chrono::DateTime<chrono::Utc>,
	pub amount: rust_decimal::Decimal,
}
