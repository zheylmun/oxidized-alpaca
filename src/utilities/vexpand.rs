use futures::task::Context;
use futures::task::Poll;
use futures::Sink;
use futures::SinkExt as _;
use futures::Stream;
use futures::StreamExt as _;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::pin::Pin;

/// A wrapper around a stream that expands a stream of vectors of items into a stream of items.
#[derive(Debug)]
#[doc(hidden)]
#[must_use = "streams do nothing unless polled"]
pub(crate) struct Vexpand<S, T, E> {
    /// The wrapped stream & sink.
    wrapped: S,
    /// A queue of messages
    queue: VecDeque<T>,
    /// Phantom data to make sure that we "use" `E`.
    _phantom: PhantomData<E>,
}

impl<S, T, E> Vexpand<S, T, E> {
    /// Create a new `Vexpand` object wrapping the provided stream.
    pub(crate) fn new(wrapped: S) -> Self {
        Self {
            wrapped,
            queue: VecDeque::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S, T, E> Stream for Vexpand<S, T, E>
where
    S: Stream<Item = Result<Result<Vec<T>, E>, E>> + Unpin,
    T: Unpin,
    E: Unpin,
{
    type Item = Result<Result<T, E>, E>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if !self.queue.is_empty() {
                // In order to preserve ordering we pop from the front. That is
                // vastly ineffective for large vectors, though, because
                // subsequent elements all will have to be copied.
                let message = self.queue.pop_front().unwrap();
                break Poll::Ready(Some(Ok(Ok(message))));
            } else {
                match self.wrapped.poll_next_unpin(ctx) {
                    Poll::Pending => {
                        // No new data is available yet. There is nothing to do for us
                        // except bubble up this result.
                        break Poll::Pending;
                    }
                    Poll::Ready(None) => {
                        // The stream is exhausted. Bubble up the result and be done.
                        break Poll::Ready(None);
                    }
                    Poll::Ready(Some(Err(err))) => break Poll::Ready(Some(Err(err))),
                    Poll::Ready(Some(Ok(Err(err)))) => break Poll::Ready(Some(Ok(Err(err)))),
                    Poll::Ready(Some(Ok(Ok(messages)))) => {
                        self.queue = VecDeque::from(messages);
                    }
                }
            }
        }
    }
}

impl<S, T, E, U> Sink<U> for Vexpand<S, T, E>
where
    S: Sink<U, Error = E> + Unpin,
    T: Unpin,
    E: Unpin,
{
    type Error = E;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.wrapped.poll_ready_unpin(ctx)
    }

    fn start_send(mut self: Pin<&mut Self>, message: U) -> Result<(), Self::Error> {
        self.wrapped.start_send_unpin(message)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.wrapped.poll_flush_unpin(ctx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.wrapped.poll_close_unpin(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::stream::iter;
    use futures::TryStreamExt as _;

    /// Check that we can unfold a stream of vectors of messages.
    #[tokio::test]
    #[allow(unused_qualifications)]
    async fn unfolding() {
        let it = iter([vec![1], vec![2, 3, 4], vec![], vec![5, 6]])
            .map(Result::<_, ()>::Ok)
            .map(Ok);

        let stream = Vexpand::new(it);
        let result = stream.try_collect::<Vec<_>>().await.unwrap();
        let expected = (1..=6).map(Ok).collect::<Vec<_>>();
        assert_eq!(result, expected);
    }
}
