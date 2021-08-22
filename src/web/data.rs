use std::ops::{Deref, DerefMut};

use crate::{Error, FromRequest, Request, RequestBody, Result};

/// An extractor that can extract data from the request extension.
///
/// # Example
///
/// ```
/// use poem::{get, middleware::AddData, route, web::Data, EndpointExt};
///
/// #[get]
/// async fn index(data: Data<&i32>) {
///     assert_eq!(*data.0, 10);
/// }
///
/// let app = route().at("/", index).with(AddData::new(10));
/// ```
pub struct Data<T>(pub T);

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Data<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl<'a, T: Send + Sync + 'static> FromRequest<'a> for Data<&'a T> {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        req.extensions()
            .get::<T>()
            .ok_or_else(|| {
                Error::internal_server_error(format!(
                    "Data of type `{}` was not found.",
                    std::any::type_name::<T>()
                ))
            })
            .map(Data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{handler, middleware::AddData, Endpoint, EndpointExt};

    #[tokio::test]
    async fn test_data_extractor() {
        #[handler(internal)]
        async fn index(value: Data<&i32>) {
            assert_eq!(value.0, &100);
        }

        let app = index.with(AddData::new(100i32));
        app.call(Request::builder().finish()).await;
    }
}
