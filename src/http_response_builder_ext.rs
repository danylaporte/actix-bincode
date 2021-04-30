use actix_web::{http::header::ContentType, HttpResponse, HttpResponseBuilder};
use serde::Serialize;
use tracing::error;

/// Allow to serialize in bincode on the `HttpResponseBuilder`.
pub trait HttpResponseBuilderExt {
    /// Set a bincode body and generate `Response`
    ///
    /// `ResponseBuilder` can not be used after this call.
    fn bincode<T: Serialize>(&mut self, value: T) -> HttpResponse;

    /// Set a bincode body and generate `Response`
    ///
    /// `ResponseBuilder` can not be used after this call.
    fn bincode2<T: Serialize>(&mut self, value: &T) -> HttpResponse;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn bincode<T: Serialize>(&mut self, value: T) -> HttpResponse {
        self.bincode2(&value)
    }

    fn bincode2<T: Serialize>(&mut self, value: &T) -> HttpResponse {
        match bincode::serialize(value) {
            Ok(body) => {
                self.insert_header(ContentType("application/bincode".parse().unwrap()));
                self.body(actix_web::dev::Body::from(body)).into()
            }
            Err(e) => {
                error!("Serialize error: {}", e);
                HttpResponse::InternalServerError()
                    .reason("unable to serialize bincode.")
                    .finish()
            }
        }
    }
}
