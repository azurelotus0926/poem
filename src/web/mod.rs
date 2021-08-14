//! Commonly used as the type of extractor or response.

mod data;
mod form;
mod json;
#[cfg(feature = "multipart")]
mod multipart;
mod path;
mod query;
mod typed_header;

use std::convert::Infallible;

use bytes::Bytes;

/// Commonly used typed headers.
pub mod type_headers {
    pub use typed_headers::Accept;
    pub use typed_headers::AcceptEncoding;
    pub use typed_headers::Allow;
    pub use typed_headers::AuthScheme;
    pub use typed_headers::Authorization;
    pub use typed_headers::ContentCoding;
    pub use typed_headers::ContentEncoding;
    pub use typed_headers::ContentLength;
    pub use typed_headers::ContentType;
    pub use typed_headers::Credentials;
    pub use typed_headers::Host;
    pub use typed_headers::HttpDate;
    pub use typed_headers::ProxyAuthorization;
    pub use typed_headers::RetryAfter;
    pub use typed_headers::Token68;
    pub use typed_headers::{Quality, QualityItem};
}

pub use data::Data;
pub use form::Form;
pub use json::Json;
#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
pub use multipart::{Field, Multipart};
pub use path::Path;
pub use query::Query;
pub use typed_header::TypedHeader;

use crate::body::Body;
use crate::error::{Error, Result};
use crate::http::header::HeaderMap;
use crate::http::{Method, StatusCode, Uri, Version};
use crate::request::Request;
use crate::response::Response;

/// Types that can be created from requests.
#[async_trait::async_trait]
pub trait FromRequest: Sized {
    /// Perform the extraction.
    async fn from_request(req: &mut Request) -> Result<Self>;
}

/// Trait for generating responses.
///
/// Types that implement [IntoResponse] can be returned from endpoints/handlers.
pub trait IntoResponse {
    /// Consume itself and return [`Response`].
    fn into_response(self) -> Result<Response>;
}

impl IntoResponse for String {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(Body::empty())
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(Body::empty())
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Result<Response> {
        Response::builder().status(self).body(Body::empty())
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Result<Response> {
        let mut resp = self.1.into_response()?;
        resp.set_status(self.0);
        Ok(resp)
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, HeaderMap, T) {
    fn into_response(self) -> Result<Response> {
        let mut resp = self.2.into_response()?;
        resp.set_status(self.0);
        resp.headers_mut().extend(self.1.into_iter());
        Ok(resp)
    }
}

impl<T: IntoResponse, E: Into<Error>> IntoResponse for Result<T, E> {
    fn into_response(self) -> Result<Response> {
        self.map_err(Into::into)
            .and_then(IntoResponse::into_response)
    }
}

/// An HTML response.
pub struct Html<T>(pub T);

impl<T: Into<String>> IntoResponse for Html<T> {
    fn into_response(self) -> Result<Response> {
        Response::builder()
            .content_type("text/html")
            .body(self.0.into().into())
    }
}

#[async_trait::async_trait]
impl FromRequest for Uri {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.uri().clone())
    }
}

#[async_trait::async_trait]
impl FromRequest for Method {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.method().clone())
    }
}

#[async_trait::async_trait]
impl FromRequest for Version {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.version())
    }
}

#[async_trait::async_trait]
impl FromRequest for HeaderMap {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.headers().clone())
    }
}

#[async_trait::async_trait]
impl FromRequest for Body {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.take_body())
    }
}

#[async_trait::async_trait]
impl FromRequest for String {
    async fn from_request(req: &mut Request) -> Result<Self> {
        String::from_utf8(req.take_body().into_bytes().await?.to_vec()).map_err(Error::bad_request)
    }
}

#[async_trait::async_trait]
impl FromRequest for Bytes {
    async fn from_request(req: &mut Request) -> Result<Self> {
        req.take_body().into_bytes().await
    }
}

#[async_trait::async_trait]
impl FromRequest for Vec<u8> {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(req.take_body().into_bytes().await?.to_vec())
    }
}

#[async_trait::async_trait]
impl<T: FromRequest> FromRequest for Option<T> {
    async fn from_request(req: &mut Request) -> Result<Self> {
        Ok(T::from_request(req).await.ok())
    }
}
