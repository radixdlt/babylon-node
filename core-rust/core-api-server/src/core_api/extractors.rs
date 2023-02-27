use axum::{
    async_trait,
    body::HttpBody,
    extract::{rejection::JsonRejection, FromRequest, FromRequestParts},
    http::request::Parts,
    http::Request,
    response::IntoResponse,
};
use serde::Serialize;

use super::{client_error, ResponseError};

// We define our own `Json` extractor that customizes the error from `axum::Json`

#[derive(Debug)]
pub(crate) struct Json<T>(pub T);
pub use axum::extract::State; // Re-export Extension so that it can be used easily

#[async_trait]
impl<S, T> FromRequestParts<S> for Json<T>
where
    axum::Json<T>: FromRequestParts<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = ResponseError<()>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(client_error(format!("{:?}", rejection))),
        }
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Json<T>
where
    axum::Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
    S: Send + Sync,
    B: HttpBody + Send + 'static,
{
    type Rejection = ResponseError<()>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
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
