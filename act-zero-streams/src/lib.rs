use std::pin::Pin;
use std::task::{Context, Poll};

use act_zero::*;
use async_trait::async_trait;
use futures::{ready, stream::Stream, Future};
use pin_project_lite::pin_project;

#[async_trait]
pub trait StreamHandler<I>
where
    Self: Actor,
{
    async fn handle(&mut self, item: I);

    async fn started2(&mut self) {}

    async fn finished(&mut self) {}

    fn add_stream<S>(stream: S, addr: Addr<Self>)
    where
        S: Stream + Send + 'static,
        Self: StreamHandler<S::Item> + Sized,
        S::Item: Send,
    {
        addr.send_fut(ActorStream::new(stream, addr.clone()));
    }
}

pin_project! {
    pub struct ActorStream<S, A>
    where
S: Stream,
A: StreamHandler<S::Item>,
{
        #[pin]
        stream: S,
        actor: Addr<A>,
        started: bool,
    }
}

impl<S, A> ActorStream<S, A>
where
    S: Stream,
    A: StreamHandler<S::Item>,
{
    pub fn new(fut: S, actor: Addr<A>) -> Self {
        Self {
            stream: fut,
            actor,
            started: false,
        }
    }
}

impl<S, A> Future for ActorStream<S, A>
where
    S: Stream,
    S::Item: Send + 'static,
    A: StreamHandler<S::Item>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, task: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if !*this.started {
            *this.started = true;
            let a = this.actor.as_addr();
            //send!(<a as StreamHandler>::started());
            send!(a.started2());
        }

        let mut polled = 0;

        while let Some(msg) = ready!(this.stream.as_mut().poll_next(task)) {
            let a = this.actor.as_addr();
            send!(a.handle(msg));

            polled += 1;

            if polled == 16 {
                return Poll::Pending;
            }
        }
        let a = this.actor.as_addr();
        send!(a.finished());
        Poll::Ready(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
