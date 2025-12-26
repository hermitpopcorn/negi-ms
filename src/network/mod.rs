use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use tokio::sync::Mutex;

use crate::ErrorInterface;

pub mod dummies;
pub mod reqwest_client;

pub struct ClientRequest {
	pub url: String,
	pub headers: Option<HashMap<String, String>>,
	pub body_json: Value,
}

pub struct ClientResponse {
	pub code: u16,
	pub body: String,
}

#[async_trait::async_trait]
pub trait Client: Send + Sync {
	async fn post(&self, request: ClientRequest) -> Result<ClientResponse, ErrorInterface>;
}

pub type ClientInterface = Arc<Mutex<dyn crate::network::Client>>;
