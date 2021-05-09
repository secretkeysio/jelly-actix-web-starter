use std::task::{Context, Poll};

use actix_web::{Error, HttpResponse};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::LOCATION;
use actix_service::{Service, Transform};
use futures::future::{ok, Either, Ready};

use crate::error::render;
use crate::request::Authentication;

/// A guard that enables route and scope authentication gating.
#[derive(Debug)]
pub struct Auth {
    /// Where to redirect the user to if they fail an 
    /// authentication check.
    pub redirect_to: &'static str
}

impl<S, B> Transform<S> for Auth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service,
            redirect_to: self.redirect_to
        })
    }
}

/// Middleware for checking user authentication status and redirecting depending
/// on the result. You generally don't need this type, but it needs to be exported
/// for compiler reasons.
pub struct AuthMiddleware<S> {
    /// Where to redirect to.
    redirect_to: &'static str,

    /// The service provided.
    service: S,
}

impl<S, B> Service for AuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let (request, payload) = req.into_parts();

        let status = request.is_authenticated();

        match status {
            Ok(v) if v == true => {
                let req = ServiceRequest::from_parts(request, payload).ok().unwrap();
                Either::Left(self.service.call(req))
            },

            Ok(_) => Either::Right(ok(ServiceResponse::new(
                request,
                HttpResponse::Found()
                    .header(LOCATION, self.redirect_to)
                    .finish()
                    .into_body()
            ))),

            Err(e) => Either::Right(ok(ServiceResponse::new(
                request,
                HttpResponse::InternalServerError()
                    .body(&render(&e.into()))
                    .into_body()
            )))
        }
    }
}

