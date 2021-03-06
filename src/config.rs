use crate::BincodePayloadError;
use actix_web::{web, HttpRequest};
use std::sync::Arc;

// Allow shared refs to default.
const DEFAULT_CONFIG: BincodeConfig = BincodeConfig {
    limit: 32_768, // 2^15 bytes, (~32kB)
    err_handler: None,
    content_type: None,
};

#[derive(Clone)]
pub struct BincodeConfig {
    pub(crate) limit: usize,
    pub(crate) err_handler:
        Option<Arc<dyn Fn(BincodePayloadError, &HttpRequest) -> actix_web::Error + Send + Sync>>,
    pub(crate) content_type: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl Default for BincodeConfig {
    fn default() -> Self {
        DEFAULT_CONFIG.clone()
    }
}

impl BincodeConfig {
    /// Change max size of payload. By default max size is 32Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(BincodePayloadError, &HttpRequest) -> actix_web::Error + Send + Sync + 'static,
    {
        self.err_handler = Some(Arc::new(f));
        self
    }

    /// Set predicate for allowed content types
    pub fn content_type_raw<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.content_type = Some(Arc::new(predicate));
        self
    }

    /// Extract payload config from app data. Check both `T` and `Data<T>`, in that order, and fall
    /// back to the default payload config.
    pub(crate) fn from_req(req: &HttpRequest) -> &Self {
        req.app_data::<Self>()
            .or_else(|| req.app_data::<web::Data<Self>>().map(|d| d.as_ref()))
            .unwrap_or_else(|| &DEFAULT_CONFIG)
    }
}
