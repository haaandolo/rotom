use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use rotom_data::error::SocketError;

#[derive(Debug)]
pub struct ExecutionRequestError<Request> {
    pub request: Request,
    pub request_retry_count: u8,
    pub error: SocketError,
}

#[derive(Debug)]
#[pin_project::pin_project]
pub struct ExecutionRequestFuture<Request, ResponseFuture> {
    request: Request,
    #[pin]
    response_future: tokio::time::Timeout<ResponseFuture>,
    request_retry_count: u8,
}

impl<Request, ResponseFuture, T> Future for ExecutionRequestFuture<Request, ResponseFuture>
where
    Request: Clone,
    ResponseFuture: Future<Output = Result<T, SocketError>>,
{
    type Output = Result<T, ExecutionRequestError<Request>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.response_future.poll(cx).map(|result| match result {
            Ok(res) => match res {
                Ok(res_inner) => Ok(res_inner),
                Err(error) => Err(ExecutionRequestError {
                    request: this.request.clone(),
                    request_retry_count: *this.request_retry_count + 1,
                    error,
                }),
            },
            Err(error) => Err(ExecutionRequestError {
                request: this.request.clone(),
                request_retry_count: *this.request_retry_count + 1,
                error: SocketError::TimeOut(error),
            }),
        })
    }
}

impl<Request, ResponseFuture> ExecutionRequestFuture<Request, ResponseFuture>
where
    ResponseFuture: Future,
{
    pub fn new(
        future: ResponseFuture,
        timeout: std::time::Duration,
        request: Request,
        request_retry_count: u8,
    ) -> Self {
        Self {
            request,
            response_future: tokio::time::timeout(timeout, future),
            request_retry_count,
        }
    }
}
