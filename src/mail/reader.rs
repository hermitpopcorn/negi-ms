use std::path::PathBuf;

#[cfg(debug_assertions)]
use log::debug;

use log::info;
use mailparse::{ParsedMail, body::Body, parse_mail};
use tokio::fs;

use crate::ErrorInterface;

use super::{Mail, RawMail, get_maildir_cur_path, get_maildir_new_path};

pub async fn read_emails() -> Result<Vec<Mail>, ErrorInterface> {
	let maildir_path: Vec<PathBuf> = vec![get_maildir_new_path()?, get_maildir_cur_path()?];

	let raw_mails = walk_directory(&maildir_path).await?;
	let parsed_mails = parse_raw_emails(raw_mails);

	info!("{} emails found", parsed_mails.len());

	#[cfg(debug_assertions)]
	for mail in &parsed_mails {
		debug!("{:#?}", mail);
	}

	Ok(parsed_mails)
}

async fn walk_directory(maildir_paths: &Vec<PathBuf>) -> Result<Vec<RawMail>, ErrorInterface> {
	let mut raw_mails = vec![];

	for maildir_path in maildir_paths {
		let mut entries = fs::read_dir(maildir_path).await?;
		while let Some(entry) = entries.next_entry().await? {
			if !entry.path().is_file() {
				continue;
			}

			let contents = fs::read(entry.path()).await?;
			raw_mails.push(RawMail {
				file_path: entry.path(),
				contents,
			});
		}
	}

	Ok(raw_mails)
}

fn parse_raw_emails(mails: Vec<RawMail>) -> Vec<Mail> {
	mails
		.into_iter()
		.filter_map(|raw_mail| {
			let parsed = parse_mail(&raw_mail.contents);
			if parsed.is_err() {
				return None;
			}
			let parsed: ParsedMail<'_> = parsed.unwrap();

			// Parse subject and from fields from header
			let mut subject = String::from("");
			let mut from = String::from("");
			let headers = parsed.get_headers();
			for header in headers {
				if header.get_key().to_lowercase() == "subject" {
					subject = String::from(header.get_value());
				} else if header.get_key().to_lowercase() == "from" {
					from = String::from(header.get_value());
				}
			}

			// Parse body into String
			let mut body: String = String::from("");
			for part in parsed.parts() {
				match part.get_body_encoded() {
					Body::Base64(b) | Body::QuotedPrintable(b) => {
						let decoded = b.get_decoded_as_string().unwrap_or(String::from(""));
						body.push_str(decoded.as_str());
					}
					Body::SevenBit(b) | Body::EightBit(b) => {
						let decoded = b.get_as_string().unwrap_or(String::from(""));
						body.push_str(decoded.as_str());
					}
					_ => {}
				}
			}

			Some(Mail {
				file_path: raw_mail.file_path,
				from,
				subject,
				body,
			})
		})
		.collect::<Vec<Mail>>()
}
