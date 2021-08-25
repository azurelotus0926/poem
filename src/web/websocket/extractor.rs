use std::{borrow::Cow, future::Future};

use hyper::upgrade::OnUpgrade;
use tokio_tungstenite::tungstenite::protocol::Role;

use super::{utils::sign, WebSocketStream};
use crate::{
    http::{
        header::{self, HeaderValue},
        Method, StatusCode,
    },
    Body, Error, FromRequest, IntoResponse, Request, RequestBody, Response, Result,
};

/// An extractor that can accept websocket connections.
pub struct WebSocket {
    key: HeaderValue,
    on_upgrade: OnUpgrade,
    protocols: Option<Box<[Cow<'static, str>]>>,
    sec_websocket_protocol: Option<HeaderValue>,
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for WebSocket {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        if req.method() != Method::GET
            || req.headers().get(header::CONNECTION) != Some(&HeaderValue::from_static("Upgrade"))
            || req.headers().get(header::UPGRADE) != Some(&HeaderValue::from_static("websocket"))
            || req.headers().get(header::SEC_WEBSOCKET_VERSION)
                != Some(&HeaderValue::from_static("13"))
        {
            return Err(Error::new(StatusCode::BAD_REQUEST));
        }

        let key = req
            .headers()
            .get(header::SEC_WEBSOCKET_KEY)
            .cloned()
            .ok_or_else(|| Error::new(StatusCode::BAD_REQUEST))?;

        let sec_websocket_protocol = req.headers().get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        let on_upgrade = match req.state().on_upgrade.lock().take() {
            Some(on_upgrade) => on_upgrade,
            None => return Err(Error::new(StatusCode::INTERNAL_SERVER_ERROR)),
        };

        Ok(Self {
            key,
            on_upgrade,
            protocols: None,
            sec_websocket_protocol,
        })
    }
}

impl WebSocket {
    /// Set the known protocols.
    ///
    /// If the protocol name specified by `Sec-WebSocket-Protocol` header
    /// to match any of them, the upgrade response will include
    /// `Sec-WebSocket-Protocol` header and return the protocol name.
    ///
    /// ```
    /// use futures_util::{SinkExt, StreamExt};
    /// use poem::{handler, route, web::websocket::WebSocket, IntoResponse};
    ///
    /// #[handler]
    /// async fn index(ws: WebSocket) -> impl IntoResponse {
    ///     ws.protocols(vec!["graphql-rs", "graphql-transport-ws"])
    ///         .on_upgrade(|socket| async move {
    ///             // ...
    ///         })
    /// }
    ///
    /// let mut app = route();
    /// app.at("/").get(index);
    /// ```
    #[must_use]
    pub fn protocols<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        self.protocols = Some(
            protocols
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        );
        self
    }

    /// Finalize upgrading the connection and call the provided `callback` with
    /// the stream.
    ///
    /// Note that the return value of this function must be returned from the
    /// handler.
    #[must_use]
    pub fn on_upgrade<F, Fut>(self, callback: F) -> impl IntoResponse
    where
        F: FnOnce(WebSocketStream) -> Fut + Send + Sync + 'static,
        Fut: Future + Send + 'static,
    {
        WebSocketUpgraded {
            websocket: self,
            callback,
        }
    }
}

struct WebSocketUpgraded<F> {
    websocket: WebSocket,
    callback: F,
}

impl<F, Fut> IntoResponse for WebSocketUpgraded<F>
where
    F: FnOnce(WebSocketStream) -> Fut + Send + Sync + 'static,
    Fut: Future + Send + 'static,
{
    fn into_response(self) -> Response {
        // check requested protocols
        let protocol = self
            .websocket
            .sec_websocket_protocol
            .as_ref()
            .and_then(|req_protocols| {
                let req_protocols = req_protocols.to_str().ok()?;
                let protocols = self.websocket.protocols.as_ref()?;
                req_protocols
                    .split(',')
                    .map(|req_p| req_p.trim())
                    .find(|req_p| protocols.iter().any(|p| p == req_p))
            });

        let mut builder = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, "upgrade")
            .header(header::UPGRADE, "websocket")
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                sign(self.websocket.key.as_bytes()),
            );

        if let Some(protocol) = protocol {
            builder = builder.header(
                header::SEC_WEBSOCKET_PROTOCOL,
                HeaderValue::from_str(protocol).unwrap(),
            );
        }

        let resp = builder.body(Body::empty());

        tokio::spawn(async move {
            let upgraded = match self.websocket.on_upgrade.await {
                Ok(upgraded) => upgraded,
                Err(_) => return,
            };

            let stream =
                tokio_tungstenite::WebSocketStream::from_raw_socket(upgraded, Role::Server, None)
                    .await;
            (self.callback)(WebSocketStream::new(stream)).await;
        });

        resp
    }
}
