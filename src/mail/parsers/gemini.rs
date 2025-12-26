use serde::Deserialize;

use crate::{ErrorInterface, network::ClientInterface};
use crate::{mail::Mail, network::ClientRequest};

use super::{EmailParsingScheme, Transaction};

pub struct GeminiParsingScheme {
	pub client: ClientInterface,
	pub api_key: String,
	pub model: String,
	pub accounts: Option<Vec<String>>,
	pub skips: Option<Vec<String>>,
}

impl GeminiParsingScheme {
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
		if self.accounts.as_ref().is_some_and(|v| v.len() > 0) {
			for account in self.accounts.as_deref().unwrap() {
				accounts_str.push_str(&format!("'{}',", account));
			}
		}
		accounts_str.pop();

		let mut skips_str = String::new();
		if self.skips.as_ref().is_some_and(|v| v.len() > 0) {
			skips_str.push_str(
				"Skip an entry if it has a subject or place of purchase that contains any of this: ",
			);
			for skip in self.skips.as_deref().unwrap() {
				skips_str.push_str(&format!("'{}',", skip));
			}
			skips_str.pop();
		}

		format!(
			"Parse the following email contents and give me the time of purchase, where/what I purchased, when the purchase happened
			(in UTC time, RFC 3339 format), and how much money I spent (make it negative).
			Format your result in JSON, just as I specified in the generation config's schema.
			Make the items independent, do not create some sort of header object and do not make an item if it does not have an amount or a purchase date.
			Do not fill subject with the subject of the email, fill it using the name of item I purchased or where I purchased it at.
			Change any half-width Japanese kana to full-width, except spaces, from the subject. Change full-width spaces to regular, half-width spaces.
			Change full-width alphabets into regular, half-width alphabets.
			Remove suffixes such as \"/NFC\" from the subject. Trim any whitespaces such as spaces, tabs, and newlines from the start or the end of the subjects.
			If the email is in Japanese and has no purchase time specified, assume it's 00:00:00 AM JST.
			If the email is in Indonesian or English and has no purchase time specified, assume it's 00:00:00 AM WIB.
			For account, choose one that fits best the email from this list: {}.
			{}.
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
		return self.accounts.as_ref().is_some_and(|v| v.len() > 0);
	}

	async fn parse(&self, mail: &Mail) -> Result<Vec<Transaction>, ErrorInterface> {
		let url = format!(
			"https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
			self.model, self.api_key,
		);

		let generation_config = self.make_generation_config();
		let prompt = self.make_prompt(mail);
		let body_json = self.make_body(generation_config, prompt);

		let request = ClientRequest {
			url,
			headers: None,
			body_json,
		};

		let response;
		{
			let client_guard = self.client.lock().await;
			response = client_guard.post(request).await?;
		}

		if response.code != 200 {
			return Err(format!(
				"Response failed, error code: {}, body: {}",
				response.code, response.body,
			)
			.into());
		}

		let response_json = serde_json::from_str::<ResponseFormat>(&response.body)?;
		let transactions = response_json.candidates[0].content.parts[0].text.clone();
		let transactions = serde_json::from_str::<Vec<Transaction>>(&transactions)?;

		if transactions.is_empty() {
			return Err("No transactions found".into());
		}

		Ok(transactions)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use tokio::sync::Mutex;

	use crate::{
		mail::{
			Mail,
			parsers::{EmailParsingScheme, gemini::GeminiParsingScheme},
		},
		network::dummies::DummyClient,
	};

	#[test]
	fn can_only_parse_if_target_accounts_defined() {
		let mail = Mail::create_test_mail();
		let client = Arc::new(Mutex::new(DummyClient::new()));

		{
			let scheme = GeminiParsingScheme {
				client: client.clone(),
				api_key: "key".into(),
				model: String::from("some-model"),
				accounts: None,
				skips: None,
			};
			assert_eq!(false, scheme.can_parse(&mail));
		}

		{
			let scheme: GeminiParsingScheme = GeminiParsingScheme {
				client: client.clone(),
				api_key: "key".into(),
				model: String::from("some-model"),
				accounts: Some(vec![]),
				skips: None,
			};
			assert_eq!(false, scheme.can_parse(&mail));
		}

		{
			let scheme: GeminiParsingScheme = GeminiParsingScheme {
				client: client.clone(),
				api_key: "key".into(),
				model: String::from("some-model"),
				accounts: Some(vec!["Some Account".into()]),
				skips: None,
			};
			assert_eq!(true, scheme.can_parse(&mail));
		}
	}

	#[tokio::test]
	async fn non_200_response_returns_expected_err() {
		let mail = Mail::create_test_mail();
		let client = Arc::new(Mutex::new(DummyClient::new()));
		{
			client.lock().await.inject_response(500, "ERR!".into());
		}

		{
			let scheme: GeminiParsingScheme = GeminiParsingScheme {
				client: client.clone(),
				api_key: "key".into(),
				model: String::from("some-model"),
				accounts: Some(vec!["Some Account".into()]),
				skips: None,
			};
			assert_eq!(true, scheme.can_parse(&mail));

			let parse_result = scheme.parse(&mail).await;
			assert_eq!(true, parse_result.is_err());
			let error_message = parse_result.err().unwrap().to_string();
			assert_eq!(
				"Response failed, error code: 500, body: ERR!",
				error_message
			);
		}
	}
}
