use ::log::info;
use dotenv::dotenv;
use log::setup_logger;
use mail::{
	cleaner::remove_emails,
	parsers::{
		EmailParsingScheme, gemini::GeminiParsingScheme, ocbc::OcbcPaymentNotificationScheme,
		rakuten_card::RakutenCardParsingScheme, rakuten_pay::RakutenPayParsingScheme,
	},
};

mod log;
mod mail;
mod sheet;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	let parsers: Vec<Box<dyn EmailParsingScheme>> = vec![
		Box::new(GeminiParsingScheme {
			model: "gemini-2.0-flash",
			accounts: vec!["Rakuten", "OCBC", "BCA", "Jenius"],
			skips: vec![
				"デイリーヤマザキアプ",
				"ローソンアプリ",
				"ファミリーマートアプ",
			],
		}),
		Box::new(RakutenPayParsingScheme {
			account: String::from("Rakuten"),
		}),
		Box::new(RakutenCardParsingScheme {
			account: String::from("Rakuten"),
		}),
		Box::new(OcbcPaymentNotificationScheme {
			account: String::from("OCBC"),
		}),
	];

	setup_logger();

	let mails = mail::reader::read_emails().await?;
	let transactions = mail::parsers::parse_emails(mails, &parsers).await?;

	if transactions.values().map(|t| t.len()).sum::<usize>() < 1 {
		info!("No transactions found. Exiting early");
		return Ok(());
	}

	let client = sheet::auth::get_sheets_client().await?;
	let inserted_transactions = sheet::write::append_to_sheet(&client, transactions).await?;

	remove_emails(inserted_transactions.into_keys().collect()).await?;

	Ok(())
}
