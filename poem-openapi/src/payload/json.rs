use std::ops::{Deref, DerefMut};

use poem::{FromRequest, IntoResponse, Request, RequestBody, Response, Result};
use serde_json::Value;

use crate::{
    error::ParseJsonError,
    payload::{ParsePayload, Payload},
    registry::{MetaMediaType, MetaResponse, MetaResponses, MetaSchemaRef, Registry},
    types::{ParseFromJSON, ToJSON, Type},
    ApiResponse,
};

/// A JSON payload.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Type> Payload for Json<T> {
    const CONTENT_TYPE: &'static str = "application/json";

    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    #[allow(unused_variables)]
    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

#[poem::async_trait]
impl<T: ParseFromJSON> ParsePayload for Json<T> {
    const IS_REQUIRED: bool = T::IS_REQUIRED;

    async fn from_request(request: &Request, body: &mut RequestBody) -> Result<Self> {
        let data: Vec<u8> = FromRequest::from_request(request, body).await?;
        let value = if data.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&data).map_err(|err| ParseJsonError {
                reason: err.to_string(),
            })?
        };

        let value = T::parse_from_json(value).map_err(|err| ParseJsonError {
            reason: err.into_message(),
        })?;
        Ok(Self(value))
    }
}

impl<T: ToJSON> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        poem::web::Json(self.0.to_json()).into_response()
    }
}

impl<T: ToJSON> ApiResponse for Json<T> {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(200),
                content: vec![MetaMediaType {
                    content_type: Self::CONTENT_TYPE,
                    schema: Self::schema_ref(),
                }],
                headers: vec![],
            }],
        }
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

impl_apirequest_for_payload!(Json<T>, T: ParseFromJSON);
