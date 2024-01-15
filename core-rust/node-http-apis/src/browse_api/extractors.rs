use axum::{
    async_trait,
    body::HttpBody,
    extract::{
        rejection::{BytesRejection, FailedToBufferBody, JsonRejection},
        FromRequest,
    },
    http::Request,
    response::IntoResponse,
};
use serde::Serialize;
use crate::browse_api::models;
use super::{client_error, ResponseError};

pub use axum::extract::State;

#[derive(Debug)]
pub(crate) struct Json<T>(pub T);

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Json<T>
where
    axum::Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
    S: Send + Sync,
    B: HttpBody + Send + 'static,
{
    type Rejection = ResponseError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(client_error(format!("{rejection:?}"), models::ErrorDetails::InvalidRequestDetails {})),
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
