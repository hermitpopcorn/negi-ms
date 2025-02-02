use std::{env, path::PathBuf};

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

impl PartialEq for Mail {
	fn eq(&self, other: &Self) -> bool {
		self.file_path == other.file_path
	}
}

impl Eq for Mail {}

pub fn get_maildir_new_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let maildir_path_str = env::var("MAILDIR_PATH").expect("MAILDIR_PATH must be set");
	let maildir_path = PathBuf::from(&maildir_path_str);
	let maildir_path = maildir_path.join("new");

	if !maildir_path.exists() {
		return Err("Maildir path(s) do not exist.".into());
	}

	Ok(maildir_path)
}

pub fn get_maildir_cur_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let maildir_path_str = env::var("MAILDIR_PATH").expect("MAILDIR_PATH must be set");
	let maildir_path = PathBuf::from(&maildir_path_str);
	let maildir_path = maildir_path.join("cur");

	if !maildir_path.exists() {
		return Err("Maildir path(s) do not exist.".into());
	}

	Ok(maildir_path)
}
