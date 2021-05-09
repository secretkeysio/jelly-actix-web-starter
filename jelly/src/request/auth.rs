use actix_web::HttpRequest;
use actix_session::UserSession;

use crate::Result;
use crate::accounts::User;

/// `Authentication` is kind of a request guard - it returns a Future which will resolve
/// with either the current authenticated user, or "error" out if the user has no session data
/// that'd tie them to a user profile, or if the session cache can't be read, or if the database
/// has issues, or... pick your poison I guess.
///
pub trait Authentication {
    /// Returns whether a user session exists and is valid.
    fn is_authenticated(&self) -> Result<bool>;

    /// Sets a serializable user instance.
    fn set_user(&self, account: User) -> Result<()>;

    /// Returns a User, if it can be extracted properly.
    fn user(&self) -> Result<User>;
}

impl Authentication for HttpRequest {
    #[inline(always)]
    fn is_authenticated(&self) -> Result<bool> {
        Ok(self.get_session().get::<serde_json::Value>("sku")?.is_some())
    }

    fn set_user(&self, account: User) -> Result<()> {
        self.get_session().set("sku", account)?;
        Ok(())
    }

    fn user(&self) -> Result<User> {
        match self.get_session().get("sku")? {
            Some(user) => Ok(user),
            None => Ok(User::default())
        }
    }
}
