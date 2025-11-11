use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use crate::{Session, crypto::Message};

pub struct Scheduler<SessionStream> {
    session_stream: SessionStream,
    current_session: Session,
    messages: Vec<Message>,
}

impl<SessionStream> Scheduler<SessionStream> {
    pub fn new(current_session: Session, session_stream: SessionStream) -> Self {
        Self {
            session_stream,
            current_session,
            messages: Vec::new(),
        }
    }

    pub fn schedule(&mut self, msg: Message, session: Session) -> Result<(), ()> {
        if session == self.current_session {
            self.messages.push(msg);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<SessionStream> Stream for Scheduler<SessionStream>
where
    SessionStream: Stream<Item = Session> + Unpin,
{
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match Pin::new(&mut this.session_stream).poll_next(cx) {
            Poll::Ready(Some(new_session)) => {
                this.current_session = new_session;
                this.messages.clear();
                Poll::Ready(Some(()))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
