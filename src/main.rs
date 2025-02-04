use dotenv::dotenv;
use mail::{
	cleaner::remove_emails,
	parsers::{
		EmailParsingScheme, ocbc::OcbcPaymentNotificationScheme,
		rakuten_pay::RakutenPayParsingScheme,
	},
};

mod mail;
mod sheet;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	let parsers: Vec<Box<dyn EmailParsingScheme>> = vec![
		Box::new(RakutenPayParsingScheme {
			account: String::from("Rakuten Pay"),
		}),
		Box::new(OcbcPaymentNotificationScheme {
			account: String::from("OCBC"),
		}),
	];

	let mails = mail::reader::read_emails().await?;
	let transactions = mail::parsers::parse_emails(mails, &parsers)?;

	let client = sheet::auth::get_sheets_client().await?;
	let inserted_transactions = sheet::write::append_to_sheet(&client, transactions).await?;

	remove_emails(inserted_transactions.into_keys().collect()).await?;

	Ok(())
}
