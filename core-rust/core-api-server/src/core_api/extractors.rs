use axum::{
    async_trait,
    body::HttpBody,
    extract::{rejection::JsonRejection, FromRequest, RequestParts},
    response::IntoResponse,
};
use serde::Serialize;

use super::{client_error, ResponseError};

// We define our own `Json` extractor that customizes the error from `axum::Json`

#[derive(Debug)]
pub(crate) struct Json<T>(pub T);
pub use axum::Extension; // Re-export Extension so that it can be used easily

#[async_trait]
impl<B, T> FromRequest<B> for Json<T>
where
    axum::Json<T>: FromRequest<B, Rejection = JsonRejection>,
    B: HttpBody + Send,
{
    type Rejection = ResponseError<()>;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(client_error(format!("{rejection:?}"))),
        }
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json::<T>::into_response(axum::Json(self.0))
    }
}
