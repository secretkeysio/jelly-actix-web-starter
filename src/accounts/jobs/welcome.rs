use std::collections::HashMap;

use jelly::JobResult;
use jelly::email::Email;
use jelly::error::Error;
use jelly::sqlxmq::{self, job, CurrentJob};

use crate::accounts::Account;

/// A job for sending a Welcome email, generally dispatched after an account
/// has been verified.
#[job]
pub async fn send_welcome_email(mut current_job: CurrentJob) -> JobResult<()> {
    let to = current_job.json::<i32>()?.ok_or_else(|| {
        Error::Generic("Invalid JSON supplied".to_string())
    })?;

    let pool = current_job.pool();

    let (name, email) = Account::fetch_email(to, &pool).await?;

    let email = Email::new("welcome", &[email], {
        let mut model = HashMap::new();
        model.insert("preview", "Welcome to the service".into());
        model.insert("name", name);
        model
    });
    
    email.send()?;
    current_job.complete().await?;
    
    Ok(())
}
