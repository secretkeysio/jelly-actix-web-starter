use jelly::prelude::*;
use jelly::accounts::OneTimeUseTokenGenerator;
use jelly::request::DatabasePool;
use jelly::Result;

use crate::accounts::Account;

/// Decodes the pieces used in verify and reset-password URL structures,
/// and validates them. If they're valid, it will return the Account in
/// question - if not, it will raise a generic error.
///
/// Flows should silence this error and display a generic message to
/// the user to avoid leaking information.
pub async fn validate_token(
    request: &HttpRequest,
    uidb64: &str,
    ts: &str,
    token: &str
) -> Result<Account> {
    if let Ok(uid_bytes) = base64_url::decode(&uidb64) {
        if let Ok(uid_str) = std::str::from_utf8(&uid_bytes) {
            if let Ok(uid) = uid_str.parse::<i32>() {
                let db = request.db_pool()?;

                if let Ok(account) = Account::get(uid, db).await {
                    // Actix-web route params are iffy here, so...
                    // we rebuild the full token before passing in.
                    let token = format!("{}-{}", ts, token);

                    if account.is_token_valid(&token) {
                        return Ok(account);
                    }
                }
            }
        }
    }

    Err(ErrorResponse::Other(Error::InvalidAccountToken))
}
