#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]

use async_trait::async_trait;
use futures::Stream;
use std::error::Error;
use std::task::{Poll, Context};
use swagger::{ApiError, ContextWrapper};
use serde::{Serialize, Deserialize};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &'static str = "/core";
pub const API_VERSION: &'static str = "0.1.0";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum StatusNetworkConfigurationPostResponse {
    /// Network Configuration
    NetworkConfiguration
    (models::NetworkConfigurationResponse)
    ,
    /// An error occurred
    AnErrorOccurred
    (models::ErrorResponse)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TransactionPreviewPostResponse {
    /// Transaction preview response
    TransactionPreviewResponse
    (models::TransactionPreviewResponse)
    ,
    /// An error occurred
    AnErrorOccurred
    (models::ErrorResponse)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum TransactionSubmitPostResponse {
    /// Transaction Submit Response
    TransactionSubmitResponse
    (models::TransactionSubmitResponse)
    ,
    /// An error occurred
    AnErrorOccurred
    (models::ErrorResponse)
}

/// API
#[async_trait]
pub trait Api<C: Send + Sync> {
    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Get Network Configuration
    async fn status_network_configuration_post(
        &self,
        context: &C) -> Result<StatusNetworkConfigurationPostResponse, ApiError>;

    /// Preview a transaction against the latest network state
    async fn transaction_preview_post(
        &self,
        transaction_preview_request: models::TransactionPreviewRequest,
        context: &C) -> Result<TransactionPreviewPostResponse, ApiError>;

    /// Submit transaction to the network
    async fn transaction_submit_post(
        &self,
        transaction_submit_request: models::TransactionSubmitRequest,
        context: &C) -> Result<TransactionSubmitPostResponse, ApiError>;

}

/// API where `Context` isn't passed on every API call
#[async_trait]
pub trait ApiNoContext<C: Send + Sync> {

    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Get Network Configuration
    async fn status_network_configuration_post(
        &self,
        ) -> Result<StatusNetworkConfigurationPostResponse, ApiError>;

    /// Preview a transaction against the latest network state
    async fn transaction_preview_post(
        &self,
        transaction_preview_request: models::TransactionPreviewRequest,
        ) -> Result<TransactionPreviewPostResponse, ApiError>;

    /// Submit transaction to the network
    async fn transaction_submit_post(
        &self,
        transaction_submit_request: models::TransactionSubmitRequest,
        ) -> Result<TransactionSubmitPostResponse, ApiError>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync> where Self: Sized
{
    /// Binds this API to a context.
    fn with_context(self: Self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
         ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Get Network Configuration
    async fn status_network_configuration_post(
        &self,
        ) -> Result<StatusNetworkConfigurationPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().status_network_configuration_post(&context).await
    }

    /// Preview a transaction against the latest network state
    async fn transaction_preview_post(
        &self,
        transaction_preview_request: models::TransactionPreviewRequest,
        ) -> Result<TransactionPreviewPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().transaction_preview_post(transaction_preview_request, &context).await
    }

    /// Submit transaction to the network
    async fn transaction_submit_post(
        &self,
        transaction_submit_request: models::TransactionSubmitRequest,
        ) -> Result<TransactionSubmitPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().transaction_submit_post(transaction_submit_request, &context).await
    }

}


#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;


pub mod server;

// Re-export router() as a top-level name

pub use self::server::Service;


pub mod context;

pub mod models;


pub(crate) mod header;
