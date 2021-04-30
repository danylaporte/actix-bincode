//! # Example
//! ```
//! use actix_bincode::Bincode;
//!
//! #[derive(serde::Deserialize)]
//! struct User {
//!     name: String,
//! }
//!
//! #[derive(serde::Serialize)]
//! struct Greeting {
//!     inner: String,
//! }
//!
//! #[actix_web::get("/users/hello")]
//! pub async fn greet_user(user: Bincode<User>) -> Bincode<Greeting> {
//!     let name: &str = &user.name;
//!     let inner: String = format!("Hello {}!", name);
//!     Bincode(Greeting { inner })
//! }
//! ```

#[cfg(test)]
#[macro_use]
extern crate serde;

use actix_web::{
    dev::Payload, http::StatusCode, FromRequest, HttpRequest, HttpResponse, Responder,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use body::*;
pub use config::*;
pub use error::*;
pub use http_response_builder_ext::*;

mod body;
mod config;
mod error;
mod http_response_builder_ext;

#[cfg(test)]
mod tests;

/// Extractor/Responder for BinCode encoded data.
///
/// This will encode data with the content-type `application/bincode`.
///
/// By default, it expects to receive data with that content-type as well.
///
/// # Example
/// ```
/// use actix_bincode::Bincode;
///
/// #[derive(serde::Deserialize)]
/// struct User {
///     name: String,
/// }
///
/// #[derive(serde::Serialize)]
/// struct Greeting {
///     inner: String,
/// }
///
/// #[actix_web::get("/users/hello")]
/// pub async fn greet_user(user: Bincode<User>) -> Bincode<Greeting> {
///     let name: &str = &user.name;
///     let inner: String = format!("Hello {}!", name);
///     Bincode(Greeting { inner })
/// }
/// ```
pub struct Bincode<T>(pub T);

impl<T> Bincode<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Bincode<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Bincode<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Bincode<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bincode: {:?}", self.0)
    }
}

impl<T> Responder for Bincode<T>
where
    T: Serialize,
{
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let body = match bincode::serialize(&self.0) {
            Ok(body) => body,
            Err(e) => {
                tracing::error!(
                    "Failed to serialize to Bincode. \
                     Request path: {} \
                     {}",
                    req.path(),
                    e,
                );
                return HttpResponse::InternalServerError().body("Internal Server Error");
            }
        };

        HttpResponse::build(StatusCode::OK)
            .content_type("application/bincode")
            .body(body)
    }
}

impl<T> FromRequest for Bincode<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;
    type Config = BincodeConfig;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let config = BincodeConfig::from_req(req);

        let limit = config.limit;
        let ctype = config.content_type.clone();
        let err_handler = config.err_handler.clone();

        BincodeBody::new(req, payload, ctype)
            .limit(limit)
            .map(move |res| match res {
                Err(e) => {
                    tracing::debug!(
                        "Failed to deserialize Bincode from payload. \
                         Request path: {}",
                        req2.path()
                    );

                    if let Some(err) = err_handler {
                        Err((*err)(e, &req2))
                    } else {
                        Err(e.into())
                    }
                }
                Ok(data) => Ok(Bincode(data)),
            })
            .boxed_local()
    }
}
