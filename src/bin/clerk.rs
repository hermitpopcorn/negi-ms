#[macro_use]
extern crate rocket;

use std::env;
use std::net::Ipv4Addr;
use std::path::Path;

use chrono::DateTime;
use dotenv::dotenv;
use negi::log::setup_logger;
use negi::sheet::auth::get_sheets_client;
use negi::sheet::write::append_to_sheet;
use negi::transaction::Transaction;
use reqwest::Client;
use rocket::State;
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::serde::{Deserialize, json::Json};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct InputData {
	account: String,
	datetime: DateTime<chrono::Utc>,
	amount: f64,
	subject: Option<String>,
}

#[post("/api/submit", format = "json", data = "<input>")]
async fn submit(sheets_client: &State<Client>, input: Json<InputData>) -> Status {
	let mut data = input.into_inner();

	let amount = Decimal::from_f64(data.amount);
	if amount.is_none() {
		return Status::BadRequest;
	}
	let amount = amount.unwrap();

	if data.subject.is_some() && data.subject.as_ref().unwrap().is_empty() {
		data.subject = None;
	}

	let transactions = vec![Transaction {
		account: data.account,
		datetime: data.datetime,
		amount,
		subject: data.subject,
	}];

	let result = append_to_sheet(sheets_client, transactions).await;
	if let Err(e) = result {
		error!("Failed to append to sheet: {}", e);
		return Status::InternalServerError;
	}

	Status::Ok
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	setup_logger();
	let sheets_client = get_sheets_client().await?;

	let mut port = 7000;
	if let Ok(port_str) = env::var("CLERK_PORT") {
		if let Ok(port_num) = port_str.parse::<u16>() {
			port = port_num;
		}
	}
	let figment = rocket::Config::figment().merge(("port", port)).merge(("address", Ipv4Addr::LOCALHOST));

	let result = rocket::custom(figment)
		.mount("/", FileServer::from(Path::new("clerk-fe-public")))
		.manage(sheets_client)
		.mount("/", routes![submit])
		.launch()
		.await;
	if let Err(e) = result {
		return Err(format!("Rocket error: {}", e.to_string()).into());
	}

	Ok(())
}
