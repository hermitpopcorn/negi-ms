use std::env;

use reqwest::{
	Client,
	header::{self, CONTENT_TYPE},
};
use serde::Deserialize;

use crate::mail::Mail;

use super::{EmailParsingScheme, Transaction};

pub struct GeminiParsingScheme {
	pub accounts: Vec<&'static str>,
	pub skips: Vec<&'static str>,
}

impl GeminiParsingScheme {
	fn build_client(&self) -> Result<Client, Box<dyn std::error::Error>> {
		let mut headers = header::HeaderMap::new();

		headers.insert(
			CONTENT_TYPE,
			header::HeaderValue::from_static("application/json"),
		);
		let client = Client::builder().default_headers(headers).build()?;

		Ok(client)
	}

	fn make_generation_config(&self) -> serde_json::Value {
		serde_json::json!({
			"response_mime_type": "application/json",
			"response_schema": {
				"type": "ARRAY",
				"items": {
					"type": "OBJECT",
					"properties": {
						"subject": {"type":"STRING"},
						"datetime": {"type":"STRING"},
						"amount": {"type":"NUMBER"},
						"account": {"type":"STRING"}
					}
				}
			}
		})
	}

	fn make_prompt(&self, mail: &Mail) -> String {
		let mut accounts_str = String::new();
		for account in &self.accounts {
			accounts_str.push_str(&format!("'{}',", *account));
		}
		accounts_str.pop();

		let mut skips_str = String::new();
		for skip in &self.skips {
			skips_str.push_str(&format!("'{}',", *skip));
		}
		skips_str.pop();

		format!(
			"Parse the following email contents and give me the time of purchase, where/what I purchased, when the purchase happened
			(in UTC time, RFC 3339 format), and how much money I spent (make it negative). Format your result in JSON,
			just as I specified in the generation config's schema. For account, choose one that fits best the email from this list: {}.
			Skip an entry if it has a subject or place of purchase that contains any of this: {}.
			Return an empty array if you can't parse the email or can't choose a suitable account from the list.
			This is the email: {}",
			accounts_str,
			skips_str,
			mail.body,
		)
	}

	fn make_body(&self, generation_config: serde_json::Value, prompt: String) -> serde_json::Value {
		serde_json::json!({
			"generationConfig": generation_config,
			"contents": [{
				"parts": [{ "text": prompt }]
			}]
		})
	}
}

#[derive(Deserialize, Debug)]
struct ResponseFormat {
	candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
	content: Content,
}

#[derive(Deserialize, Debug)]
struct Content {
	parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
	text: String,
}

#[async_trait::async_trait]
impl EmailParsingScheme for GeminiParsingScheme {
	fn can_parse(&self, _: &Mail) -> bool {
		true // Should be able to parse anything
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
		let client = self.build_client()?;
		let url = format!(
			"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-lite-preview-02-05:generateContent?key={}",
			env::var("GEMINI_API_KEY")?,
		);

		let generation_config = self.make_generation_config();
		let prompt = self.make_prompt(mail);
		let body = self.make_body(generation_config, prompt);

		let response = client.post(&url).json(&body).send().await?;
		let response_json = response.json::<ResponseFormat>().await?;
		let transactions = response_json.candidates[0].content.parts[0].text.clone();
		let transactions = serde_json::from_str::<Vec<Transaction>>(&transactions)?;

		if transactions.is_empty() {
			return Err("No transactions found".into());
		}

		Ok(transactions)
	}
}
