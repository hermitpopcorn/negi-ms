use std::env;

use reqwest::{
	Client,
	header::{self, CONTENT_TYPE},
};
use serde::Deserialize;

use crate::ErrorInterface;
use crate::mail::Mail;

use super::{EmailParsingScheme, Transaction};

pub struct GeminiParsingScheme<'a> {
	pub model: &'a str,
	pub accounts: Vec<&'a str>,
	pub skips: Vec<&'a str>,
}

impl<'a> GeminiParsingScheme<'a> {
	fn build_client(&self) -> Result<Client, ErrorInterface> {
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
			(in UTC time, RFC 3339 format), and how much money I spent (make it negative).
			Format your result in JSON, just as I specified in the generation config's schema.
			Make the items independent, do not create some sort of header object and do not make an item if it does not have an amount or a purchase date.
			Do not fill subject with the subject of the email, fill it using the name of item I purchased or where I purchased it at. Change any half-width Japanese characters to full-width Japanese character, except spaces, from the subject. Remove suffixes such as \"/NFC\" from the subject. Trim any whitespaces such as spaces, tabs, and newlines from the start or the end of the subjects.
			If the email is in Japanese and has no purchase time specified, assume it's 00:00:00 AM JST.
			If the email is in Indonesian or English and has no purchase time specified, assume it's 00:00:00 AM WIB.
			For account, choose one that fits best the email from this list: {}.
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
impl<'a> EmailParsingScheme for GeminiParsingScheme<'a> {
	fn can_parse(&self, _: &Mail) -> bool {
		true // Should be able to parse anything
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, ErrorInterface> {
		let client = self.build_client()?;
		let url = format!(
			"https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
			self.model,
			env::var("GEMINI_API_KEY")?,
		);

		let generation_config = self.make_generation_config();
		let prompt = self.make_prompt(mail);
		let body = self.make_body(generation_config, prompt);

		let response = client.post(&url).json(&body).send().await?;
		let response_text = response.text().await?;

		#[cfg(debug_assertions)]
		{
			use log::debug;
			debug!("Gemini response: {}", response_text);
		}

		let response_json = serde_json::from_str::<ResponseFormat>(&response_text)?;
		let transactions = response_json.candidates[0].content.parts[0].text.clone();
		let transactions = serde_json::from_str::<Vec<Transaction>>(&transactions)?;

		if transactions.is_empty() {
			return Err("No transactions found".into());
		}

		Ok(transactions)
	}
}
