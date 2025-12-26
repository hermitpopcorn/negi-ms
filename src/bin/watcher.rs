use std::env;
use std::sync::Arc;

use ::log::info;
use dotenv::dotenv;
use log::error;
use negi::ErrorInterface;
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
use negi::network::ClientInterface;
use negi::network::reqwest_client::ReqwestClient;
use negi::sheet::auth::get_sheets_client;
use negi::sheet::write::append_to_sheet;
use negi::transaction::Transaction;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), ErrorInterface> {
	dotenv().ok();
	setup_logger();

	let client: ClientInterface = Arc::new(Mutex::new(ReqwestClient::new()));

	let parsers = get_parsers(&client)?;
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
		Ok(_) => {
			info!("Appended to sheet");
			remove_emails(mails).await?;
		}
		Err(e) => error!("Appending error: {}", e.to_string()),
	}

	Ok(())
}

fn get_parsers(
	client_interface: &ClientInterface,
) -> Result<Vec<Box<dyn EmailParsingScheme>>, ErrorInterface> {
	Ok(vec![
		Box::new(get_gemini_parser(client_interface)?),
		Box::new(RakutenPayParsingScheme {
			account: env::var("RAKUTEN_PAY_PARSING_SCHEME_TARGET_ACCOUNT")
				.unwrap_or(String::from("Rakuten")),
		}),
		Box::new(RakutenCardParsingScheme {
			account: env::var("RAKUTEN_CARD_PARSING_SCHEME_TARGET_ACCOUNT")
				.unwrap_or(String::from("Rakuten")),
		}),
		Box::new(OcbcPaymentNotificationScheme {
			account: env::var("OCBC_PAYMENT_NOTIFICATION_PARSING_SCHEME_TARGET_ACCOUNT")
				.unwrap_or(String::from("OCBC")),
		}),
	])
}

fn get_gemini_parser(
	client_interface: &ClientInterface,
) -> Result<GeminiParsingScheme, ErrorInterface> {
	Ok(GeminiParsingScheme {
		client: client_interface.clone(),
		model: env::var("GEMINI_MODEL").unwrap_or(String::from("gemini-2.5-flash")),
		api_key: env::var("GEMINI_API_KEY")?,
		accounts: 'gemini_target_accounts: {
			let accounts_string = env::var("GEMINI_TARGET_ACCOUNTS");
			if accounts_string.is_err() {
				break 'gemini_target_accounts None;
			}
			let accounts_string = accounts_string.unwrap();
			if accounts_string.len() < 1 {
				break 'gemini_target_accounts None;
			}
			let accounts = accounts_string
				.split(",")
				.map(|s| s.to_owned())
				.collect::<Vec<String>>();
			Some(accounts)
		},
		skips: None,
	})
}
