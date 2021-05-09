//! Jelly implements various helpers, traits, and types for actix-web that 
//! enable a nicer building experience. It's not released as a framework, 
//! as I don't think this works long-term - instead, clone and chisel away 
//! to get what you need.

// We re-export/hoist a few things that are commonly imported.
// Less time screwing around with Cargo.toml for a framework-feel is
// ideal.
pub use actix_rt;
pub use actix_web;
pub use actix_service;
pub use actix_session;
pub use anyhow;
pub use async_trait;
pub use chrono;
pub use djangohashers;
pub use futures;
pub use serde;
pub use serde_json;
pub use sqlx;
pub use sqlxmq;
pub use tera;

#[macro_use] pub extern crate log;

pub mod accounts;
pub mod email;
pub mod error;
pub mod forms;
pub mod guards;
pub mod prelude;
pub mod request;
pub mod utils;

mod templates;
mod server;
pub use server::Server;

pub type Result<T> = std::result::Result<T, crate::error::ErrorResponse>;
pub type JobResult<T> = std::result::Result<T, crate::error::Error>;
