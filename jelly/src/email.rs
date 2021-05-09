//! Provides a very, very basic `Email` struct that can send via Postmark. 
//! This is designed for transactional emails - if you need otherwise, 
//! you're free to import Lettre or whatever.
//!
//! If you prefer a different provider than Postmark, you can swap the
//! send implementation in here.

use std::collections::HashMap;
use std::env::var;

use chrono::{Datelike, Utc};
use serde::Serialize;

use crate::error::Error;

/// Represents information that Postmark can use to send emails. This by 
/// default relies on templates existing on the Postmark side - you'll send 
/// less data over the wire this way.
#[derive(Debug, Default, Serialize)]
pub struct Email<'a> {
    /// The template alias (e.g, 'verify-email`).
    #[serde(rename = "TemplateAlias")]
    pub alias: &'a str,

    /// Data that the template can use to render.
    #[serde(rename = "TemplateModel")]
    pub model: HashMap<&'a str, String>,

    /// Who's sending this.
    #[serde(rename = "From")]
    pub from: String,
    
    /// Who to send to. Comma-delimited.
    #[serde(
        rename = "To"
    )]
    pub to: String
}

impl<'a> Email<'a> {
    /// Construct a new `Email`.
    pub fn new(
        alias: &'a str,
        to: &[String],
        model: HashMap<&'a str, String>
    ) -> Self {
        Email {
            alias,
            model,
            to: to.join(","),
            from: var("POSTMARK_DEFAULT_FROM")
                .expect("POSTMARK_DEFAULT_FROM not set!")
        }
    }

    /// Send the email. Relies on you ensuring that `POSTMARK_API_KEY` 
    /// is set in your `.env`.
    pub fn send(mut self) -> Result<(), Error> {
        let now = Utc::now();
        let year = now.year();
        self.model.insert("year", year.to_string());

        let api_key = std::env::var("POSTMARK_API_KEY")
            .expect("POSTMARK_API_KEY not set!");

        minreq::post("https://api.postmarkapp.com/email/withTemplate")
            .with_header("X-Postmark-Server-Token", api_key)
            .with_json(&self)?.send()?;

        Ok(())
    }
}
