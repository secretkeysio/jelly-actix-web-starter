use std::collections::HashMap;
use std::env;

use jelly::JobResult;
use jelly::accounts::OneTimeUseTokenGenerator;
use jelly::email::Email;
use jelly::error::Error;
use jelly::sqlxmq::{self, job, CurrentJob};

use crate::accounts::Account;

/// Dispatches a "verify this account" email.
#[job]
pub async fn send_verify_account_email(mut current_job: CurrentJob) -> JobResult<()> {
    let to = current_job.json::<i32>()?.ok_or_else(|| {
        Error::Generic("Invalid JSON supplied".to_string())
    })?;

    let pool = current_job.pool();

    let account = Account::get(to, &pool).await?;

    let domain = env::var("DOMAIN")
        .expect("No DOMAIN value set!");

    let verify_url = format!(
        "{}/accounts/verify/{}-{}/",
        domain,
        base64_url::encode(&format!("{}", account.id)),
        account.create_reset_token()?
    );

    let email = Email::new("verify-account", &[account.email], {
        let mut model = HashMap::new();
        model.insert("preview", "Verify your new account".into());
        model.insert("action_url", verify_url);
        model
    });
    
    email.send()?;
    current_job.complete().await?;
    
    Ok(())
}
