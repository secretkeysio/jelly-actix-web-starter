//! Registers background jobs for the Accounts module.

use jelly::sqlxmq::NamedJob;

mod verify;
pub use verify::send_verify_account_email;

mod welcome;
pub use welcome::send_welcome_email;

mod reset_password;
pub use reset_password::{send_password_was_reset_email, send_reset_password_email};

mod odd_registration_attempt;
pub use odd_registration_attempt::send_odd_registration_attempt_email;

pub fn configure() -> Vec<&'static NamedJob> {
    vec![
        send_welcome_email,
        send_verify_account_email,
        send_password_was_reset_email,
        send_reset_password_email,
        send_odd_registration_attempt_email
    ]
}
