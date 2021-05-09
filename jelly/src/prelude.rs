//! These are useful for many common use-cases, so they're exported here - 
//! you can (generally) safely just use `crate::jelly::prelude::*;` 
//! and move on with your life.

pub use super::{
    actix_web::{HttpRequest, HttpResponse},

    error::{Error, ErrorResponse},

    // A trait used for calling validate() on form field types. Your form(s) can also implement
    // this, so it's exported here for ease of use.
    forms::Validation,
    
    // A string that is stored as jsonb in the database, keyed for locales.
    //i18n::{I18nString},

    // Enables various helpers for actix_web's `HttpRequest` type.
    request::{Authentication, DatabasePool, FlashMessages, Render},

    tera::Context
};
