//! OpenAPI support for Poem.
//!
//! `Poem-openapi` allows you to easily implement APIs that comply with the
//! `OpenAPIv3` specification. It uses procedural macros to generate a lots of
//! boilerplate code, so that you only need to focus on the more
//! important business implementations.
//!
//! * [Book](https://poem-web.github.io/poem/)
//! * [Docs](https://docs.rs/poem-openapi)
//! * [Cargo package](https://crates.io/crates/poem-openapi)
//!
//! ## Features
//!
//! * **Type safety** If your codes can be compiled, then it is fully compliant
//!   with the `OpenAPI v3` specification.
//! * **Rustfmt friendly** Do not create any DSL that does not conform to Rust's
//!   syntax specifications.
//! * **IDE friendly** Any code generated by the procedural macro will not be
//!   used directly.
//! * **Minimal overhead** All generated code is necessary, and there is almost
//!   no overhead.
//!
//! ## Crate features
//!
//! To avoid compiling unused dependencies, Poem gates certain features, some of
//! which are disabled by default:
//!
//! | Feature    | Description |
//! |------------|-----------------------------------------------------------------------|
//! | chrono     | Integrate with the [`chrono` crate](https://crates.io/crates/chrono). |
//! | swagger-ui | Add swagger UI support |
//! | rapidoc    | Add RapiDoc UI support |
//! | redoc      | Add Redoc UI support |
//! | email      | Support for email address string |
//! | hostname   | Support for hostname string |
//! | uuid       | Integrate with the [`uuid` crate](https://crates.io/crates/uuid)|
//!
//! ## Example
//!
//! ```ignore
//! use poem::{listener::TcpListener, Route};
//! use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
//!
//! struct Api;
//!
//! #[OpenApi]
//! impl Api {
//!     #[oai(path = "/hello", method = "get")]
//!     async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
//!         match name.0 {
//!             Some(name) => PlainText(format!("hello, {}!", name)),
//!             None => PlainText("hello!".to_string()),
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let api_service =
//!         OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
//!     let ui = api_service.swagger_ui();
//!     let app = Route::new().nest("/api", api_service).nest("/", ui);
//!
//!     poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
//!         .run(app)
//!         .await
//! }
//! ```
//!
//! ## Run example
//!
//! Open `http://localhost:3000/ui` in your browser, you will see the `Swagger UI` that contains these API definitions.
//!
//! ```shell
//! > cargo run --example hello_world
//!
//! > curl http://localhost:3000
//! hello!
//!
//! > curl http://localhost:3000\?name\=sunli
//! hello, sunli!
//! ```

#![doc(html_favicon_url = "https://poem.rs/assets/favicon.ico")]
#![doc(html_logo_url = "https://poem.rs/en/assets/logo.png")]
#![forbid(unsafe_code)]
#![deny(private_in_public, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[macro_use]
mod macros;

pub mod auth;
pub mod param;
pub mod payload;
#[doc(hidden)]
pub mod registry;
pub mod types;
#[doc(hidden)]
pub mod validation;

mod base;
mod error;
mod openapi;
#[cfg(any(feature = "swagger-ui", feature = "rapidoc", feature = "redoc"))]
mod ui;

pub use base::{
    ApiExtractor, ApiExtractorType, ApiResponse, CombinedAPI, ExtractParamOptions, OAuthScopes,
    OpenApi, Tags,
};
pub use error::ParseRequestError;
pub use openapi::{LicenseObject, OpenApiService, ServerObject};
#[doc = include_str!("docs/request.md")]
pub use poem_openapi_derive::ApiRequest;
#[doc = include_str!("docs/response.md")]
pub use poem_openapi_derive::ApiResponse;
#[doc = include_str!("docs/enum.md")]
pub use poem_openapi_derive::Enum;
#[doc = include_str!("docs/multipart.md")]
pub use poem_openapi_derive::Multipart;
#[doc = include_str!("docs/oauth_scopes.md")]
pub use poem_openapi_derive::OAuthScopes;
#[doc = include_str!("docs/object.md")]
pub use poem_openapi_derive::Object;
#[doc = include_str!("docs/oneof.md")]
pub use poem_openapi_derive::OneOf;
#[doc = include_str!("docs/openapi.md")]
pub use poem_openapi_derive::OpenApi;
#[doc = include_str!("docs/security_scheme.md")]
pub use poem_openapi_derive::SecurityScheme;
#[doc = include_str!("docs/tags.md")]
pub use poem_openapi_derive::Tags;

#[doc(hidden)]
pub mod __private {
    pub use mime;
    pub use poem;
    pub use serde;
    pub use serde_json;

    pub use crate::base::UrlQuery;
}
