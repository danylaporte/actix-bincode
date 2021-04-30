use crate::BincodePayloadError;
use actix_web::{
    dev::Payload, http::header::CONTENT_LENGTH, web::BytesMut, HttpMessage, HttpRequest,
};
use futures_util::{
    future::{FutureExt, LocalBoxFuture},
    StreamExt,
};
use serde::de::DeserializeOwned;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Request's payload bincode parser, it resolves to a deserialized `T` value.
/// This future could be used with `ServiceRequest` and `ServiceFromRequest`.
///
/// Returns error:
///
/// * content type is not `application/bincode`
///   (unless specified in [`BincodeConfig`](struct.BincodeConfig.html))
/// * content length is greater than 256k
pub struct BincodeBody<U> {
    pub(crate) limit: usize,
    pub(crate) length: Option<usize>,
    #[cfg(feature = "compress")]
    pub(crate) stream: Option<Decompress<Payload>>,
    #[cfg(not(feature = "compress"))]
    pub(crate) stream: Option<Payload>,
    pub(crate) err: Option<BincodePayloadError>,
    pub(crate) fut: Option<LocalBoxFuture<'static, Result<U, BincodePayloadError>>>,
}

impl<U> BincodeBody<U>
where
    U: DeserializeOwned + 'static,
{
    /// Create `BincodeBody` for request.
    pub fn new(
        req: &HttpRequest,
        payload: &mut Payload,
        ctype: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    ) -> Self {
        // check content-type
        let mime = req.content_type();
        let is_good_mime = mime == "application/bincode"
            || mime == "bincode"
            || ctype.as_ref().map_or(false, |predicate| predicate(mime));

        if !is_good_mime {
            return BincodeBody {
                limit: 262_144,
                length: None,
                stream: None,
                fut: None,
                err: Some(BincodePayloadError::ContentType),
            };
        }

        let len = req
            .headers()
            .get(&CONTENT_LENGTH)
            .and_then(|l| l.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok());

        #[cfg(feature = "compress")]
        let payload = Decompress::from_headers(payload.take(), req.headers());
        #[cfg(not(feature = "compress"))]
        let payload = payload.take();

        BincodeBody {
            limit: 262_144,
            length: len,
            stream: Some(payload),
            fut: None,
            err: None,
        }
    }

    /// Change max size of payload. By default max size is 256Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl<U> Future for BincodeBody<U>
where
    U: DeserializeOwned + 'static,
{
    type Output = Result<U, BincodePayloadError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut fut) = self.fut {
            return Pin::new(fut).poll(cx);
        }

        if let Some(err) = self.err.take() {
            return Poll::Ready(Err(err));
        }

        let limit = self.limit;
        if let Some(len) = self.length.take() {
            if len > limit {
                return Poll::Ready(Err(BincodePayloadError::Overflow));
            }
        }
        let mut stream = self.stream.take().unwrap();

        self.fut = Some(
            async move {
                let mut body = BytesMut::with_capacity(8192);

                while let Some(item) = stream.next().await {
                    let chunk = item?;
                    if (body.len() + chunk.len()) > limit {
                        return Err(BincodePayloadError::Overflow);
                    } else {
                        body.extend_from_slice(&chunk);
                    }
                }
                Ok(bincode::deserialize(&body)?)
            }
            .boxed_local(),
        );

        self.poll(cx)
    }
}
