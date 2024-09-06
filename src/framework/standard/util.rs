use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

// Fallback will likely be called a lot, yet does nothing, so
// avoid the cost of boxing a future every call by using a ZST
// which will not allocate, but can still return a value
#[repr(transparent)]
pub struct ZSTOkFut<E>(PhantomData<E>);
// SAFETY: ZSTOkFut literally has zero state
unsafe impl<E> Send for ZSTOkFut<E> {}

impl<E> ZSTOkFut<E> {
    pub const fn new() -> Self {
        ZSTOkFut(PhantomData)
    }
}

impl<E> Future for ZSTOkFut<E> {
    type Output = Result<(), E>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(()))
    }
}
