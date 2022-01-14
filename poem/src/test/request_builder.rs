use std::collections::HashMap;

use http::{header, header::HeaderName, Extensions, HeaderMap, HeaderValue, Method};
use serde::Serialize;
use serde_json::Value;

use crate::{
    test::{TestClient, TestForm, TestResponse},
    Body, Endpoint, Request,
};

/// A request builder for testing.
pub struct TestRequestBuilder<'a, E> {
    cli: &'a TestClient<E>,
    uri: String,
    method: Method,
    query: HashMap<String, Value>,
    headers: HeaderMap,
    body: Body,
    extensions: Extensions,
}

impl<'a, E> TestRequestBuilder<'a, E>
where
    E: Endpoint,
{
    pub(crate) fn new(cli: &'a TestClient<E>, method: Method, uri: String) -> Self {
        Self {
            cli,
            uri,
            method,
            query: Default::default(),
            headers: Default::default(),
            body: Body::empty(),
            extensions: Default::default(),
        }
    }

    /// Sets the query string for this request.
    #[must_use]
    pub fn query(mut self, name: impl Into<String>, value: &impl Serialize) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.query.insert(name.into(), value);
        }
        self
    }

    /// Sets the header value for this request.
    #[must_use]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        let key = key.try_into().map_err(|_| ()).expect("valid header name");
        let value = value
            .try_into()
            .map_err(|_| ())
            .expect("valid header value");
        self.headers.append(key, value);
        self
    }

    /// Sets the content type for this request.
    #[must_use]
    pub fn content_type(self, content_type: impl AsRef<str>) -> Self {
        self.header(header::CONTENT_TYPE, content_type.as_ref())
    }

    /// Sets the body for this request.
    #[must_use]
    pub fn body(self, body: impl Into<Body>) -> Self {
        Self {
            body: body.into(),
            ..self
        }
    }

    /// Sets the JSON body for this request.
    #[must_use]
    pub fn body_json(self, body: &impl Serialize) -> Self {
        Self {
            body: serde_json::to_string(&body).expect("valid json").into(),
            ..self
        }
    }

    /// Sets the multipart body for this request.
    #[must_use]
    pub fn multipart(self, form: TestForm) -> Self {
        self.content_type(format!("multipart/form-data; boundary={}", form.boundary()))
            .body(Body::from_async_read(form.into_async_read()))
    }

    fn make_request(self) -> Request {
        let uri = if self.query.is_empty() {
            self.uri
        } else {
            format!(
                "{}?{}",
                self.uri,
                serde_urlencoded::to_string(&self.query).unwrap()
            )
        };

        let mut req = Request::builder()
            .method(self.method)
            .uri(uri.parse().expect("valid uri"))
            .finish();
        req.headers_mut().extend(self.cli.default_headers.clone());
        req.headers_mut().extend(self.headers);
        *req.extensions_mut() = self.extensions;
        req.set_body(self.body);

        req
    }

    /// Sets the extension data for this request.
    #[must_use]
    pub fn data<T>(mut self, data: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.extensions.insert(data);
        self
    }

    /// Send this request to endpoint to get the response.
    pub async fn send(self) -> TestResponse {
        let ep = &self.cli.ep;
        let req = self.make_request();
        let resp = ep.get_response(req).await;
        TestResponse::new(resp)
    }
}
