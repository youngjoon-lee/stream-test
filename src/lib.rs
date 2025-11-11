mod crypto;
mod scheduler;

pub type Session = u8;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fork_stream::StreamExt as _;
    use futures::StreamExt as _;
    use tokio::time::interval;
    use tokio_stream::wrappers::IntervalStream;

    use crate::{
        Session,
        crypto::{CryptoProcessors, build_message},
        scheduler::Scheduler,
    };

    #[tokio::test]
    async fn crypto() {
        let mut session_stream = IntervalStream::new(interval(Duration::from_secs(1)))
            .enumerate()
            .map(|(i, _)| i as Session)
            .fork();

        let current_session = session_stream.next().await.unwrap();
        let mut cryptos = CryptoProcessors::new(current_session, session_stream.clone());
        let mut scheduler = Scheduler::new(current_session, session_stream.clone());

        loop {
            tokio::select! {
                // Keep polling, so that the crypto processor can receive/process new sessions.
                Some(()) = cryptos.next() => {
                    println!("CryptoProcessors switched to new session");
                }
                // Keep polling, so that the scheduler can receive/process new sessions.
                Some(()) = scheduler.next() => {
                    println!("Scheduler switched to the new session");
                }
                // Whenever a new session arrives, try decapsulating a new message built with the
                // new session.
                // This is for simulating new incoming messages received from peers
                // as soon as a new session starts.
                Some(session) = session_stream.next() => {
                    println!("Trying decapsulating with new session {session}");
                    let (msg_session, msg) = cryptos
                        .decapsulate(build_message(session))
                        // NOTE: This would sometimes fail due to the timing issue.
                        // This can easily happen since receiving incoming messages is inter-node,
                        // or inter-service communication.
                        .expect("decapsulation must succeed with the new session");
                    assert_eq!(msg_session, session);

                    println!("Scheduling message for session {msg_session}");
                    scheduler.schedule(msg, msg_session)
                        // NOTE: This would sometimes fail due to the timing issue.
                        // I personally think this failure should be avoided somehow because
                        // the interaction between crypto processor and scheduler is an
                        // intra-service communication.
                        // If the processor successfully processed new session messages,
                        // we should be able to expect that scheduler can also handle them.
                        .expect("scheduling must succeed with the new session");
                }
            }
        }
    }
}
