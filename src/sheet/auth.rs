use std::env;

use reqwest::{
	Client,
	header::{self, AUTHORIZATION, CONTENT_TYPE},
};
use yup_oauth2::ServiceAccountAuthenticator;

pub async fn get_sheets_client() -> Result<Client, Box<dyn std::error::Error>> {
	let token = authorize().await?;
	let client = build_client(&token)?;

	Ok(client)
}

async fn authorize() -> Result<String, Box<dyn std::error::Error>> {
	let credentials_path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
	let credentials = yup_oauth2::read_service_account_key(credentials_path).await?;

	let scopes = &["https://www.googleapis.com/auth/spreadsheets"];

	let authenticator = ServiceAccountAuthenticator::builder(credentials)
		.build()
		.await?;
	let get_token = authenticator.token(scopes).await?;
	let token = get_token.token().unwrap();

	Ok(token.to_owned())
}

fn build_client(token: &str) -> Result<Client, Box<dyn std::error::Error>> {
	let mut headers = header::HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		header::HeaderValue::from_str(&format!("Bearer {}", token))?,
	);
	headers.insert(
		CONTENT_TYPE,
		header::HeaderValue::from_static("application/json"),
	);
	let client = Client::builder().default_headers(headers).build()?;

	Ok(client)
}
