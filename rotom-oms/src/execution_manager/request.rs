use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use rotom_data::error::SocketError;

///////
#[derive(Debug)]
#[pin_project::pin_project]
pub struct ExecutionRequestFuture<Request, ResponseFuture, ResponseFutureOk> {
    request: Request,
    #[pin]
    response_future: tokio::time::Timeout<ResponseFuture>,
    _marker: PhantomData<ResponseFutureOk>,
}

impl<Request, ResponseFuture, ResponseFutureOk> Future
    for ExecutionRequestFuture<Request, ResponseFuture, ResponseFutureOk>
where
    Request: Clone,
    ResponseFuture: Future<Output = Result<ResponseFutureOk, SocketError>>,
{
    type Output = Result<ResponseFutureOk, (SocketError, Request)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.response_future.poll(cx).map(|result| match result {
            Ok(res) => match res {
                Ok(res_inner) => Ok(res_inner),
                Err(error) => Err((error, this.request.clone())),
            },
            Err(_error) => Err((
                SocketError::Misc(String::from("FARK")),
                this.request.clone(),
            )),
        })
    }
}

impl<Request, ResponseFuture, ResponseFutureOk>
    ExecutionRequestFuture<Request, ResponseFuture, ResponseFutureOk>
where
    ResponseFuture: Future,
{
    pub fn new(future: ResponseFuture, timeout: std::time::Duration, request: Request) -> Self {
        Self {
            request,
            response_future: tokio::time::timeout(timeout, future),
            _marker: PhantomData::<ResponseFutureOk>,
        }
    }
}
