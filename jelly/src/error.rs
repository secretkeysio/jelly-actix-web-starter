//! A custom Error type, along with a custom Result wrapper, that we use for 
//! returning responses. This module handles converting several differing 
//! error formats into the one we use for responding.

use std::{error, fmt};
use actix_web::{HttpResponse, ResponseError, http::StatusCode};

/// This enum represents the largest classes of errors we can expect to
/// encounter in the lifespan of our application. Feel free to add to this
/// as necessary; `Generic()` exists for anything further in the stack that
/// might not fit here by default.
#[derive(Debug)]
pub enum Error {
    Anyhow(anyhow::Error),
    MinReq(minreq::Error),
    Database(sqlx::Error),
    Generic(String),
    Template(tera::Error),
    Json(serde_json::error::Error),
    Radix(radix::RadixErr),
    InvalidPassword,
    InvalidAccountToken,
    PasswordHasher(djangohashers::HasherError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Anyhow(e) => Some(e.root_cause()),
            Error::Database(e) => Some(e),
            Error::Template(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Radix(e) => Some(e),
            Error::MinReq(e) => Some(e),
            
            Error::Generic(_) | Error::InvalidPassword |
            Error::InvalidAccountToken |
            Error::PasswordHasher(_) => None,
        }
    }
}


/// This type exists because, for whatever reason, ResponseError can't be Send+Sync.
///
/// If you ask me, it should be allowed to be: it's a glorified Debug/Display trait.
///
/// But it's not, so here we are.
///
/// The various internal methods of this project will take an `Error` and translate it
/// to an `ErrorResponse`, which is the default return type for render methods.
#[derive(Debug)]
pub enum ErrorResponse {
    ActixWeb(actix_web::error::Error),
    Other(Error)
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorResponse::ActixWeb(e) => Some(e),
            ErrorResponse::Other(e) => e.source()
        }
    }
}

impl From<Error> for ErrorResponse {
    fn from(e: Error) -> Self {
        ErrorResponse::Other(e)
    }
}

impl From<actix_web::error::Error> for ErrorResponse {
    fn from(e: actix_web::error::Error) -> Self {
        ErrorResponse::ActixWeb(e)
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body(&render(self))
    }
}

macro_rules! wrap_error_type {
    ($variant:path, $wrap:path) => (
        impl From<$wrap> for Error {
            fn from(e: $wrap) -> Self {
                $variant(e)
            }
        }

        impl From<$wrap> for ErrorResponse {
            fn from(e: $wrap) -> Self {
                ErrorResponse::Other($variant(e))
            }
        }
    )
}

wrap_error_type!(Error::MinReq, minreq::Error);
wrap_error_type!(Error::Json, serde_json::error::Error);
wrap_error_type!(Error::Database, sqlx::Error);
wrap_error_type!(Error::Anyhow, anyhow::Error);
wrap_error_type!(Error::Template, tera::Error);
wrap_error_type!(Error::Radix, radix::RadixErr);
wrap_error_type!(Error::PasswordHasher, djangohashers::HasherError);

/// A generic method for rendering an error to present to the browser.
/// This should only be called in non-production settings.
pub(crate) fn render(e: &ErrorResponse) -> String {
    format!(r#"<!DOCTYPE html>
        <html>
        <head>
            <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no, maximum-scale=1.0">
            <title>Jelly: An Error Occurred</title>
            <style>
                html, body {{
                    margin: 0;
                    padding: 0;
                    background: #F0DEE0;
                    color: #111;
                    font-family: -apple-system, "Helvetica Neue", Helvetica, "Segoe UI", Ubuntu, arial, sans-serif;
                }}
                
                h1 {{ margin: 0; background: #F05758; border-bottom: 1px solid #C7484A; padding: 20px; font-size: 30px; font-weight: 600; line-height: 40px; }}
                
                code {{
                    display: block;
                    font-family: "Anonymous Pro", Consolas, Menlo, Monaco, Lucida Console, Liberation Mono, DejaVu Sans Mono, Bitstream Vera Sans Mono, Courier New, monospace, serif; 
                    font-size: 16px;
                    line-height: 20px;
                    padding: 20px;
                }}
            </style>
        </head>
        <body>
            <h1>Error</h1>
            <code>{}<code>
        </body>
        </html>
    "#, e)
}
