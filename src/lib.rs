mod crypto;

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
    };

    #[tokio::test]
    async fn crypto() {
        let mut session_stream = IntervalStream::new(interval(Duration::from_secs(1)))
            .enumerate()
            .map(|(i, _)| i as Session)
            .fork();

        let mut cryptos =
            CryptoProcessors::new(session_stream.next().await.unwrap(), session_stream.clone());

        loop {
            tokio::select! {
                // Keep polling, so that the crypto processor can receive/process new sessions.
                Some(session) = cryptos.next() => {
                    println!("CryptoProcessors switched to session {session}");
                }
                // Whenever a new session arrives, try decapsulating a new message built with the
                // new session.
                // This is for simulating new incoming messages received from peers
                // as soon as a new session starts.
                Some(session) = session_stream.next() => {
                    println!("Trying decapsulating with new session {session}");
                    assert_eq!(
                        cryptos
                            .decapsulate(build_message(session))
                            // NOTE: This would sometimes fail due to the timing issue.
                            // This can happen anytime in inter-service or inter-node communication,
                            // but I think we should somehow avoid this inside the service or inside a single task at least.
                            .expect("decapsulation must succeed with the new session")
                            .0,
                        session
                    );
                    println!("Trying decapsulating with old session {}", session - 1);
                    assert_eq!(
                        cryptos
                            .decapsulate(build_message(session - 1))
                            .expect("decapsulation must succeed with the old session")
                            .0,
                        session - 1
                    );
                }
            }
        }
    }
}
