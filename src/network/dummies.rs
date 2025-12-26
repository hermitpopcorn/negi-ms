#[cfg(test)]
use crate::network::ClientRequest;

#[cfg(test)]
use crate::{
	ErrorInterface,
	network::{Client, ClientResponse},
};

#[cfg(test)]
#[derive(Default)]
pub struct DummyClient {
	injected_response_code: Option<u16>,
	injected_response_body: Option<String>,
}

#[cfg(test)]
impl DummyClient {
	pub fn new() -> Self {
		DummyClient::default()
	}

	pub fn inject_response(&mut self, code: u16, body: String) {
		self.injected_response_code = Some(code);
		self.injected_response_body = Some(body);
	}
}

#[cfg(test)]
#[async_trait::async_trait]
impl Client for DummyClient {
	async fn post(&self, _: ClientRequest) -> Result<ClientResponse, ErrorInterface> {
		Ok(ClientResponse {
			code: self.injected_response_code.unwrap_or(200),
			body: self
				.injected_response_body
				.as_ref()
				.cloned()
				.unwrap_or("".into()),
		})
	}
}
