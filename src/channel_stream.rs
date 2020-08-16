use smol::stream;
use std::pin::Pin;
use std::sync::mpsc;
use std::task::{Context, Poll};

pub struct ChannelStream<T> {
    chan: mpsc::Receiver<T>,
    done: Option<mpsc::Receiver<()>>,
}

impl<T> From<mpsc::Receiver<T>> for ChannelStream<T> {
    fn from(chan: mpsc::Receiver<T>) -> Self {
        ChannelStream { chan, done: None }
    }
}

impl<T> stream::Stream for ChannelStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<T>> {
        match &self.done {
            Some(done) => match done.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                _ => return Poll::Ready(None),
            },
            None => {}
        }
        match self.chan.try_recv() {
            Err(mpsc::TryRecvError::Empty) => {
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(mpsc::TryRecvError::Disconnected) => Poll::Ready(None),
            Ok(data) => Poll::Ready(Some(data)),
        }
    }
}

impl<T> ChannelStream<T> {
    pub fn with_done(self, done: mpsc::Receiver<()>) -> Self {
        ChannelStream {
            chan: self.chan,
            done: Some(done),
        }
    }
}
