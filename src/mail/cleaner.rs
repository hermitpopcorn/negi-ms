use std::path::Path;

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
	use log::debug;
	debug!("Not actually deleting file");
	Ok(())
}

#[cfg(not(debug_assertions))]
async fn remove_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
	use std::env;
	use std::path::PathBuf;
	use tokio::fs;

	match env::var("PROCESSED_MAIL_DIR") {
		Ok(processed_mail_dir_path_string) => {
			let filename = path.file_name().ok_or("Could not get mail filename")?;
			let to_path = PathBuf::from(processed_mail_dir_path_string)
				.join("cur")
				.join(filename);
			fs::rename(path, to_path).await?;
		}
		Err(_) => {
			fs::remove_file(path).await?;
		}
	}

	Ok(())
}
