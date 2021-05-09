use std::collections::HashMap;
use std::env::var;

use jelly::JobResult;
use jelly::error::Error;
use jelly::email::Email;
use jelly::sqlxmq::{self, job, CurrentJob};

use crate::accounts::Account;

/// An email that gets sent if a user attempts to register
/// under an already registered email. We don't want to say
/// "this email exists already", as that reveals that a user has
/// registered for this service.
///
/// Instead we'll just send the registered account an email asking
/// if they meant to reset their password, and display to the user
/// registering the standard "verify" flow.
#[job]
pub async fn send_odd_registration_attempt_email(mut current_job: CurrentJob) -> JobResult<()> {
    let to = current_job.json::<String>()?.ok_or_else(|| {
        Error::Generic("Invalid JSON supplied".to_string())
    })?;

    let pool = current_job.pool();

    let name = Account::fetch_name_from_email(&to, &pool).await?;

    let email = Email::new("odd-registration-attempt", &[to], {
        let mut model = HashMap::new();
        model.insert("preview", "Did you want to reset your password?".into());
        model.insert("name", name);
        model.insert("action_url", format!("{}/accounts/reset/", var("DOMAIN")
                .expect("DOMAIN not set?")
        ));
        model
    });
    
    email.send()?;
    current_job.complete().await?;
    
    Ok(())
}

