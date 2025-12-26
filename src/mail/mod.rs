use std::{collections::HashMap, env, path::PathBuf};

use crate::ErrorInterface;
use crate::transaction::Transaction;

pub mod cleaner;
pub mod parsers;
pub mod reader;

pub struct RawMail {
	pub file_path: PathBuf,
	pub contents: Vec<u8>,
}

#[derive(Hash)]
pub struct Mail {
	pub file_path: PathBuf,
	pub from: String,
	pub subject: String,
	pub body: String,
}

impl Mail {
	pub fn clone_without_body(&self) -> Self {
		Self {
			file_path: self.file_path.clone(),
			from: self.from.clone(),
			subject: self.subject.clone(),
			body: String::new(),
		}
	}
}

impl PartialEq for Mail {
	fn eq(&self, other: &Self) -> bool {
		self.file_path == other.file_path
	}
}

impl Eq for Mail {}

impl std::fmt::Debug for Mail {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"------- Mail ------\nFile path: {}\nFrom: {}\nSubject: {}\nBody:\n---- Body Start ---\n{}\n----- Body End ----\n-------------------",
			self.file_path.to_str().unwrap(),
			self.from,
			self.subject,
			self.body,
		)
	}
}

pub type TransactionsParsedFromMail = HashMap<Mail, Vec<Transaction>>;

pub fn get_maildir_new_path() -> Result<PathBuf, ErrorInterface> {
	let maildir_path_str = env::var("MAILDIR_PATH").expect("MAILDIR_PATH must be set");
	let maildir_path = PathBuf::from(&maildir_path_str);
	let maildir_path = maildir_path.join("new");

	if !maildir_path.exists() {
		return Err("Maildir path(s) do not exist".into());
	}

	Ok(maildir_path)
}

pub fn get_maildir_cur_path() -> Result<PathBuf, ErrorInterface> {
	let maildir_path_str = env::var("MAILDIR_PATH").expect("MAILDIR_PATH must be set");
	let maildir_path = PathBuf::from(&maildir_path_str);
	let maildir_path = maildir_path.join("cur");

	if !maildir_path.exists() {
		return Err("Maildir path(s) do not exist".into());
	}

	Ok(maildir_path)
}

#[cfg(test)]
impl Mail {
	pub fn create_test_mail() -> Self {
		Self {
			file_path: "/tmp/fake-path".into(),
			from: "sender".into(),
			subject: "subject".into(),
			body: "contents".into(),
		}
	}
}
