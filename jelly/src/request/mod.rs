//! This module implements some traits for `HttpRequest` in order to make 
//! life a bit nicer. This enables things such as checking user 
//! authentication in a repeatable and scannable way, loading a user type,
//! and so on.

pub mod auth;
pub use auth::Authentication;

pub mod database;
pub use database::DatabasePool;

pub mod flash;
pub use flash::FlashMessages;

pub mod request;
pub use request::Render;
