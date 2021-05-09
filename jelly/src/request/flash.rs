use actix_web::HttpRequest;
use actix_session::UserSession;

use crate::templates::FlashMessage;

/// `FlashMessages` implements a one-time-message (hence "Flash") that is useful
/// for old-school HTML flows that need to display messages in a standardized way
/// across pages.
///
/// This could potentially do less serialization, but it's fine for now.
/// TODO: Look at whether this can be done with just &str rather than String.
pub trait FlashMessages {
    /// Adds a flash message to the stack.
    fn flash(&self, title: &str, message: &str) -> crate::Result<()>;

    /// Internally used; loads flash messages for template use and removes the existing
    /// stack.
    fn get_flash_messages(&self) -> crate::Result<Vec<FlashMessage>>;
}

impl FlashMessages for HttpRequest {
    fn flash(&self, title: &str, message: &str) -> crate::Result<()> {
        let session = self.get_session();

        let mut messages: Vec<FlashMessage> = match session.get("flsh")? {
            Some(messages) => messages,
            None => Vec::new()
        };

        messages.push(FlashMessage {
            title: title.to_string(),
            message: message.to_string()
        });
        session.set("flsh", messages)?;

        Ok(())
    }

    fn get_flash_messages(&self) -> crate::Result<Vec<FlashMessage>> {
        let session = self.get_session();

        let messages = match session.get("flsh")? {
            Some(messages) => messages,
            None => Vec::new()
        };

        session.remove("flsh");
        Ok(messages)
    }
}
