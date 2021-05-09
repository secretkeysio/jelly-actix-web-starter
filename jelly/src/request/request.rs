use std::sync::{Arc, RwLock};

use actix_web::{HttpRequest, HttpResponse};
use actix_web::http::header::LOCATION;
use serde::Serialize;
use tera::{Tera, Context};

use crate::Result;
use crate::error::{Error, ErrorResponse};
use super::{Authentication, FlashMessages};

/// A trait for making certain types of response handling easier.
pub trait Render {
    /// Shorthand for rendering a template, with a specific HTTP response code.
    fn render(
        &self,
        code: usize,
        template: &str,
        context: Context
    ) -> Result<HttpResponse>;

    /// Shorthand for returning a JSON payload.
    fn json<S: Serialize>(
        &self,
        code: usize,
        payload: S
    ) -> Result<HttpResponse>;

    /// Handy redirects helper.
    fn redirect(&self, location: &str) -> Result<HttpResponse>;
}

impl Render for HttpRequest {
    fn render(
        &self,
        code: usize,
        template: &str,
        mut context: Context
    ) -> Result<HttpResponse> {
        let data: Option<&Arc<RwLock<Tera>>> = self.app_data();

        // We pull the user and flash messages for all requests; 
        // it's blank if a User is anonymous (not authenticated).
        let user = self.user()?; 
        let messages = self.get_flash_messages()?;
        context.insert("user", &user);
        context.insert("flash_messages", &messages);

        if let Some(eng) = data {
            let engine = eng.read().map_err(|e| {
                ErrorResponse::Other(Error::Generic(format!("Error acquiring template read lock: {:?}", e)))
            })?;

            let body = engine.render(template, &context).map_err(ErrorResponse::from)?;

            Ok(match code {
                200 => HttpResponse::Ok(),
                400 => HttpResponse::BadRequest(),
                404 => HttpResponse::NotFound(),
                _ => HttpResponse::Ok()
            }.content_type("text/html; charset=utf-8").body(body))
        } else {
            Err(ErrorResponse::Other(Error::Generic("Unable to locate Templates cache".to_string())))
        }
    }

    fn json<S: Serialize>(
        &self,
        code: usize,
        payload: S
    ) -> Result<HttpResponse> {
        let o = serde_json::to_string(&payload)?;

        Ok(match code {
            200 => HttpResponse::Ok(),
            400 => HttpResponse::BadRequest(),
            404 => HttpResponse::NotFound(),
            _ => HttpResponse::Ok()
        }.content_type("application/json").body(o))
    }
    
    fn redirect(&self, location: &str) -> Result<HttpResponse> {
        Ok(HttpResponse::Found()
            .header(LOCATION, location)
            .finish()
            .into_body()
        )
    }
}
