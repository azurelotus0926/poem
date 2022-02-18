//! Types for ACME.
//!
//! Reference: <https://datatracker.ietf.org/doc/html/rfc8555>

mod auto_cert;
mod builder;
mod client;
mod jose;
mod keypair;
mod listener;
mod protocol;
mod resolver;
mod serde;

pub use auto_cert::AutoCert;
pub use builder::AutoCertBuilder;
pub use listener::{AutoCertAcceptor, AutoCertListener};

/// Let's Encrypt production directory url
pub const LETS_ENCRYPT_PRODUCTION: &str = "https://acme-v02.api.letsencrypt.org/directory";

/// Let's Encrypt staging directory url
pub const LETS_ENCRYPT_STAGING: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";
