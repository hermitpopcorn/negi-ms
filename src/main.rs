use ::log::info;
use dotenv::dotenv;
use log::setup_logger;
use mail::{
	Mail,
	cleaner::remove_emails,
	parsers::{
		EmailParsingScheme, gemini::GeminiParsingScheme, ocbc::OcbcPaymentNotificationScheme,
		rakuten_card::RakutenCardParsingScheme, rakuten_pay::RakutenPayParsingScheme,
	},
};
use transaction::Transaction;

mod log;
mod mail;
mod sheet;
mod transaction;

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

	let mails = transactions
		.keys()
		.map(|m| m.clone_without_body())
		.collect::<Vec<Mail>>();
	let transactions = transactions
		.into_values()
		.flatten()
		.collect::<Vec<Transaction>>();

	let client = sheet::auth::get_sheets_client().await?;
	sheet::write::append_to_sheet(&client, transactions).await?;

	remove_emails(mails).await?;

	Ok(())
}
