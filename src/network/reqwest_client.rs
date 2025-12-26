use std::str::FromStr;

use reqwest::header::{self, CONTENT_TYPE};

use crate::{
	ErrorInterface,
	network::{ClientRequest, ClientResponse},
};

pub struct ReqwestClient {
	client: reqwest::Client,
}

impl ReqwestClient {
	pub fn new() -> Self {
		ReqwestClient {
			client: reqwest::Client::new(),
		}
	}
}

#[async_trait::async_trait]
impl crate::network::Client for ReqwestClient {
	async fn post(&self, request: ClientRequest) -> Result<ClientResponse, ErrorInterface> {
		let builder = self.client.post(&request.url);

		let mut headers = header::HeaderMap::new();
		headers.insert(
			CONTENT_TYPE,
			header::HeaderValue::from_static("application/json"),
		);
		if request.headers.as_ref().is_some_and(|v| v.len() > 0) {
			for (k, v) in request.headers.unwrap() {
				headers.insert(
					header::HeaderName::from_str(&k)?,
					header::HeaderValue::from_str(&v)?,
				);
			}
		}
		let builder = builder.headers(headers);

		let builder = builder.json(&request.body_json);

		#[cfg(debug_assertions)]
		{
			use log::debug;
			debug!("Request: {}", &request.url);
		}

		let response = builder.send().await?;
		let response = ClientResponse {
			code: response.status().as_u16(),
			body: response.text().await?,
		};

		#[cfg(debug_assertions)]
		{
			use log::debug;
			debug!("Response: {}", &response.body);
		}

		Ok(response)
	}
}
