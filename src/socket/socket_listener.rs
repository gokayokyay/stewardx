/*
    Thanks Alice!
    https://stackoverflow.com/a/67251333/7487508
*/

use hyper::server::accept::Accept;
use tokio::net::UnixListener;

use std::pin::Pin;
use std::task::{Context, Poll};

pub struct UDSAccept {
    pub inner: UnixListener,
}

impl Accept for UDSAccept {
    type Conn = tokio::net::UnixStream;
    type Error = std::io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match self.inner.poll_accept(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok((socket, _addr))) => Poll::Ready(Some(Ok(socket))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
        }
    }
}
