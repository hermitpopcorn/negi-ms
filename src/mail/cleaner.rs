use tokio::fs;

use super::Mail;

pub async fn remove_emails(mails: Vec<Mail>) -> Result<(), Box<dyn std::error::Error>> {
	for mail in mails {
		if !mail.file_path.exists() {
			continue;
		}
		fs::remove_file(mail.file_path).await?;
	}

	Ok(())
}
