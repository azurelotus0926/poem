//! OpenAPI support for Poem.
//!
//! `Poem-openapi` allows you to easily implement APIs that comply with the
//! `OpenAPIv3` specification. It uses procedural macros to generate a lots of
//! boilerplate code, so that you only need to focus on the more
//! important business implementations.
//!
//! # Table of contents
//!
//! - [Features](#features)
//! - [Quickstart](#quickstart)
//! - [Crate features](#crate-features)
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
//! ## Quickstart
//!
//! Cargo.toml
//!
//! ```toml
//! [package]
//! name = "helloworld"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [dependencies]
//! poem = "1.2"
//! poem-openapi = { version = "1.2", features = ["swagger-ui"] }
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! ```
//!
//! main.rs
//!
//! ```no_run
//! use poem::{listener::TcpListener, Route, Server};
//! use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
//!
//! struct Api;
//!
//! #[OpenApi]
//! impl Api {
//!     /// Hello world
//!     #[oai(path = "/", method = "get")]
//!     async fn index(&self) -> PlainText<&'static str> {
//!         PlainText("Hello World")
//!     }
//! }
//!
//! let api_service =
//!     OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000");
//! let ui = api_service.swagger_ui();
//! let app = Route::new().nest("/", api_service).nest("/docs", ui);
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! Server::new(TcpListener::bind("127.0.0.1:3000"))
//!     .run(app)
//!     .await;
//! # });
//! ```
//!
//! ## Check it
//!
//! Open your browser at [http://127.0.0.1:3000](http://127.0.0.1:3000).
//!
//! You will see the plaintext response as:
//!
//! ```text
//! Hello World
//! ```
//!
//! ## Interactive API docs
//!
//! Now go to [http://127.0.0.1:3000/docs](http://127.0.0.1:3000/docs).
//!
//! You will see the automatic interactive API documentation (provided by
//! [Swagger UI](https://github.com/swagger-api/swagger-ui)):
//!
//! ![swagger-ui](https://raw.githubusercontent.com/poem-web/poem/master/poem-openapi/assets/swagger-ui.jpg)
//!
//! ## Crate features
//!
//! To avoid compiling unused dependencies, Poem gates certain features, some of
//! which are disabled by default:
//!
//! | Feature    | Description |
//! |------------|-----------------------------------------------------------------------|
//! | chrono     | Integrate with the [`chrono` crate](https://crates.io/crates/chrono) |
//! | time  | Integrate with the [`time` crate](https://crates.io/crates/time). |
//! | humantime  | Integrate with the [`humantime` crate](https://crates.io/crates/humantime) |
//! | swagger-ui | Add swagger UI support |
//! | rapidoc    | Add RapiDoc UI support |
//! | redoc      | Add Redoc UI support |
//! | email      | Support for email address string |
//! | hostname   | Support for hostname string |
//! | uuid       | Integrate with the [`uuid` crate](https://crates.io/crates/uuid)|
//! | url        | Integrate with the [`url` crate](https://crates.io/crates/url) |
//! | bson        | Integrate with the [`bson` crate](https://crates.io/crates/bson) |
//! | rust_decimal | Integrate with the [`rust_decimal` crate](https://crates.io/crates/rust_decimal) |
//! | static-files | Support for static file response |

#![doc(html_favicon_url = "https://raw.githubusercontent.com/poem-web/poem/master/favicon.ico")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/poem-web/poem/master/logo.png")]
#![forbid(unsafe_code)]
#![deny(private_in_public, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[macro_use]
mod macros;

pub mod auth;
pub mod error;
pub mod param;
pub mod payload;
#[doc(hidden)]
pub mod registry;
pub mod response;
pub mod types;
#[doc(hidden)]
pub mod validation;

mod base;
mod openapi;
#[cfg(any(feature = "swagger-ui", feature = "rapidoc", feature = "redoc"))]
mod ui;

pub use base::{
    ApiExtractor, ApiExtractorType, ApiResponse, ExtractParamOptions, OAuthScopes, OpenApi,
    ResponseContent, Tags, Webhook,
};
pub use openapi::{ExternalDocumentObject, LicenseObject, OpenApiService, ServerObject};
#[doc = include_str!("docs/request.md")]
pub use poem_openapi_derive::ApiRequest;
#[doc = include_str!("docs/response.md")]
pub use poem_openapi_derive::ApiResponse;
#[doc = include_str!("docs/enum.md")]
pub use poem_openapi_derive::Enum;
#[doc = include_str!("docs/multipart.md")]
pub use poem_openapi_derive::Multipart;
pub use poem_openapi_derive::NewType;
#[doc = include_str!("docs/oauth_scopes.md")]
pub use poem_openapi_derive::OAuthScopes;
#[doc = include_str!("docs/object.md")]
pub use poem_openapi_derive::Object;
#[doc = include_str!("docs/openapi.md")]
pub use poem_openapi_derive::OpenApi;
#[doc = include_str!("docs/response_content.md")]
pub use poem_openapi_derive::ResponseContent;
#[doc = include_str!("docs/security_scheme.md")]
pub use poem_openapi_derive::SecurityScheme;
#[doc = include_str!("docs/tags.md")]
pub use poem_openapi_derive::Tags;
#[doc = include_str!("docs/union.md")]
pub use poem_openapi_derive::Union;
#[doc = include_str!("docs/webhook.md")]
pub use poem_openapi_derive::Webhook;
pub use validation::Validator;

#[doc(hidden)]
pub mod __private {
    pub use mime;
    pub use poem;
    pub use serde;
    pub use serde_json;

    pub use crate::base::UrlQuery;
}
