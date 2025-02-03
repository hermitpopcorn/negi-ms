use std::path::Path;
#[cfg(not(debug_assertions))]
use tokio::fs;

use super::Mail;

pub async fn remove_emails(mails: Vec<Mail>) -> Result<(), Box<dyn std::error::Error>> {
	for mail in mails {
		if !mail.file_path.exists() {
			continue;
		}
		remove_file(&mail.file_path).await?;
	}

	Ok(())
}

#[cfg(debug_assertions)]
async fn remove_file(_: &Path) -> Result<(), Box<dyn std::error::Error>> {
	println!("Not actually deleting file.");
	Ok(())
}

#[cfg(not(debug_assertions))]
async fn remove_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
	fs::remove_file(path).await?;
	Ok(())
}
