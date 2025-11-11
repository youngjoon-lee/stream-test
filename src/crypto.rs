use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use crate::Session;

type Message = u8;

pub struct CryptoProcessors<SessionStream> {
    current: CryptoProcessor,
    old: Option<CryptoProcessor>,
    session_stream: SessionStream,
}

impl<SessionStream> CryptoProcessors<SessionStream> {
    pub fn new(current_session: Session, session_stream: SessionStream) -> Self {
        Self {
            current: CryptoProcessor(current_session),
            old: None,
            session_stream,
        }
    }

    pub fn decapsulate(&self, msg: Message) -> Result<(Session, Message), ()> {
        match self.current.decapsulate(msg) {
            Ok(result) => Ok((self.current.session(), result)),
            Err(_) => {
                if let Some(old_processor) = &self.old {
                    old_processor
                        .decapsulate(msg)
                        .map(|result| (old_processor.session(), result))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl<SessionStream> Stream for CryptoProcessors<SessionStream>
where
    SessionStream: Stream<Item = Session> + Unpin,
{
    type Item = Session;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match Pin::new(&mut this.session_stream).poll_next(cx) {
            Poll::Ready(Some(new_session)) => {
                let new_processor = CryptoProcessor(new_session);
                this.old = Some(std::mem::replace(&mut this.current, new_processor));
                Poll::Ready(Some(new_session))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

struct CryptoProcessor(Session);

impl CryptoProcessor {
    fn decapsulate(&self, msg: Message) -> Result<Message, ()> {
        if self.session() == msg {
            Ok(msg)
        } else {
            Err(())
        }
    }

    fn session(&self) -> Session {
        self.0
    }
}

pub fn build_message(session: Session) -> Message {
    session
}
