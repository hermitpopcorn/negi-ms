use ::log::info;
use dotenv::dotenv;
use log::error;
use negi::log::setup_logger;
use negi::mail::parsers::parse_emails;
use negi::mail::reader::read_emails;
use negi::mail::{
	Mail,
	cleaner::remove_emails,
	parsers::{
		EmailParsingScheme, gemini::GeminiParsingScheme, ocbc::OcbcPaymentNotificationScheme,
		rakuten_card::RakutenCardParsingScheme, rakuten_pay::RakutenPayParsingScheme,
	},
};
use negi::sheet::auth::get_sheets_client;
use negi::sheet::write::append_to_sheet;
use negi::transaction::Transaction;

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
				"楽天ペイアプリセブン",
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

	let mails = read_emails().await?;
	let transactions = parse_emails(mails, &parsers).await?;

	let transactions_count = transactions.values().map(|t| t.len()).sum::<usize>();
	if transactions_count < 1 {
		info!("No transactions found. Exiting early");
		return Ok(());
	}
	info!("Found {} transactions", transactions_count);

	let mails = transactions
		.keys()
		.map(|m| m.clone_without_body())
		.collect::<Vec<Mail>>();
	let transactions = transactions
		.into_values()
		.flatten()
		.collect::<Vec<Transaction>>();

	let client = get_sheets_client().await?;
	match append_to_sheet(&client, transactions).await {
		Ok(_) => info!("Appended to sheet"),
		Err(e) => error!("Appending error: {}", e.to_string()),
	}

	remove_emails(mails).await?;

	Ok(())
}
