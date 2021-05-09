use std::collections::HashMap;
use std::env;

use jelly::JobResult;
use jelly::accounts::OneTimeUseTokenGenerator;
use jelly::email::Email;
use jelly::error::Error;
use jelly::sqlxmq::{self, job, CurrentJob};

use crate::accounts::Account;

/// Sends a "reset your password" email.
#[job]
pub async fn send_reset_password_email(mut current_job: CurrentJob) -> JobResult<()> {
    let to = current_job.json::<String>()?.ok_or_else(|| {
        Error::Generic("Invalid JSON supplied".to_string())
    })?;

    let pool = current_job.pool();

    let account = Account::get_by_email(&to, &pool).await?;

    let domain = env::var("DOMAIN")
        .expect("No DOMAIN value set!");

    let verify_url = format!(
        "{}/accounts/reset/{}-{}/",
        domain,
        base64_url::encode(&format!("{}", account.id)),
        account.create_reset_token()?
    );

    let email = Email::new("reset-password", &[account.email], {
        let mut model = HashMap::new();
        model.insert("preview", "Reset your account password".into());
        model.insert("action_url", verify_url);
        model
    });
    
    email.send()?;
    current_job.complete().await?;
    
    Ok(())
}

/// Sends a "reset" confirmation email.
#[job]
pub async fn send_password_was_reset_email(mut current_job: CurrentJob) -> JobResult<()> {
    let to = current_job.json::<String>()?.ok_or_else(|| {
        Error::Generic("Invalid JSON supplied".to_string())
    })?;

    let email = Email::new("password-was-reset", &[to], {
        let mut model = HashMap::new();
        model.insert("preview", "Your Password Was Reset".into());
        model
    });
    
    email.send()?;
    
    current_job.complete().await?;
    
    Ok(())
}

