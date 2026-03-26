//! Event and log listener.
//!
//! Spec ref: FORMAL_SPEC.md §4 FR-006
//!
//! # Transport auto-selection
//!
//! [`Listener`] inspects the endpoint URL scheme to pick the right transport:
//!
//! | Scheme          | Strategy                                          |
//! |-----------------|---------------------------------------------------|
//! | `http://`/`https://` | HTTP polling via `eth_getLogs` (default 1 s) |
//! | `ws://`/`wss://`     | WebSocket push via `eth_subscribe`           |
//!
//! WebSocket mode reconnects automatically on disconnect — up to three times
//! with exponential back-off (50 ms × 2ⁿ).

use std::{pin::Pin, time::Duration};

use alloy_provider::{Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types::{Filter, Log};
use futures::Stream;
use thiserror::Error;
use tracing::{debug, warn};

use crate::rpc::RpcClient;

// ── Errors ────────────────────────────────────────────────────────────────────

/// Errors produced by the event listener.
#[derive(Debug, Error)]
pub enum ListenerError {
    /// Failed to establish subscription or initial log fetch.
    #[error("subscribe failed: {0}")]
    SubscribeFailed(String),

    /// All WebSocket reconnect attempts exhausted after disconnect.
    #[error("WebSocket reconnect exhausted after {0} attempts")]
    ReconnectExhausted(u32),

    /// The supplied filter is logically invalid (e.g., `from_block > to_block`).
    #[error("invalid filter: {0}")]
    FilterInvalid(String),
}

// ── Type alias ────────────────────────────────────────────────────────────────

/// A boxed, heap-allocated stream of log results.
///
/// Using a type alias keeps public function signatures concise.
pub type LogStream = Pin<Box<dyn Stream<Item = Result<Log, ListenerError>> + Send>>;

// ── Listener ──────────────────────────────────────────────────────────────────

/// Connects to an Ethereum endpoint and streams matching [`Log`] entries.
///
/// Transport is selected automatically from the endpoint URL scheme.
/// Construct with [`Listener::new`], optionally tune with
/// [`Listener::with_poll_interval`], then call [`Listener::subscribe`].
///
/// # Example
/// ```no_run
/// # #[tokio::main] async fn main() {
/// use eth_node::events::Listener;
/// use alloy_rpc_types::Filter;
/// use futures::StreamExt;
///
/// let mut stream = Listener::new("ws://127.0.0.1:8545")
///     .subscribe(Filter::new());
///
/// while let Some(result) = stream.next().await {
///     println!("{result:#?}");
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Listener {
    endpoint: String,
    /// Interval used in HTTP polling mode. Ignored for WebSocket mode.
    poll_interval: Duration,
}

impl Listener {
    /// Create a new `Listener` targeting `endpoint`.
    ///
    /// `endpoint` must begin with `http://`, `https://`, `ws://`, or `wss://`.
    /// The poll interval defaults to **1 second** (spec FR-006).
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            poll_interval: Duration::from_secs(1),
        }
    }

    /// Override the HTTP polling interval (ignored in WebSocket mode).
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Subscribe to logs matching `filter`.
    ///
    /// Transport is selected by the endpoint URL scheme (see module-level docs).
    /// Dropping the returned stream stops all background activity.
    pub fn subscribe(&self, filter: Filter) -> LogStream {
        let endpoint = self.endpoint.clone();
        let interval = self.poll_interval;

        if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
            Box::pin(ws_subscription_stream(endpoint, filter))
        } else {
            Box::pin(http_poll_stream(endpoint, filter, interval))
        }
    }
}

// ── HTTP polling ──────────────────────────────────────────────────────────────

/// Poll `eth_getLogs` at `interval` and yield each matching [`Log`] individually.
///
/// The filter's `fromBlock` cursor advances after each batch so that re-runs
/// never produce duplicate logs.  If `filter.get_from_block()` returns `None`
/// the current chain head is used as the starting block.
fn http_poll_stream(
    endpoint: String,
    filter: Filter,
    interval: Duration,
) -> impl Stream<Item = Result<Log, ListenerError>> + Send {
    async_stream::stream! {
        let client = match RpcClient::new(&endpoint) {
            Ok(c) => c,
            Err(e) => {
                yield Err(ListenerError::SubscribeFailed(e.to_string()));
                return;
            }
        };

        let start = match client.block_number().await {
            Ok(n) => n,
            Err(e) => {
                yield Err(ListenerError::SubscribeFailed(e.to_string()));
                return;
            }
        };

        let mut next_block: u64 = filter.get_from_block().unwrap_or(start);

        loop {
            tokio::time::sleep(interval).await;

            let current = match client.block_number().await {
                Ok(n) => n,
                Err(e) => {
                    warn!("http_poll: block_number failed: {e}");
                    continue;
                }
            };

            if current < next_block {
                // No new blocks yet — wait for the next tick.
                continue;
            }

            let ranged = filter.clone().from_block(next_block).to_block(current);
            match client.get_logs(&ranged).await {
                Ok(logs) => {
                    debug!(count = logs.len(), from = next_block, to = current, "http_poll: batch");
                    next_block = current + 1;
                    for log in logs {
                        yield Ok(log);
                    }
                }
                Err(e) => {
                    warn!("http_poll: get_logs failed: {e}");
                    yield Err(ListenerError::SubscribeFailed(e.to_string()));
                }
            }
        }
    }
}

// ── WebSocket subscription ────────────────────────────────────────────────────

const MAX_RECONNECT: u32 = 3;

/// Subscribe to Ethereum logs over WebSocket, yielding each [`Log`] individually.
///
/// Reconnects on disconnect (up to [`MAX_RECONNECT`] = 3 times) with
/// exponential back-off starting at 50 ms.

//Lefty: what is the reason why we only reconnect 3 times?
//What happens afterwards?
fn ws_subscription_stream(
    ws_url: String,
    filter: Filter,
) -> impl Stream<Item = Result<Log, ListenerError>> + Send {
    async_stream::stream! {
        let mut attempts: u32 = 0;

        loop {
            match connect_ws(&ws_url).await {
                Err(e) => {
                    warn!("ws_stream: connect failed (attempt {attempts}): {e}");
                    yield Err(ListenerError::SubscribeFailed(e));
                    attempts += 1;
                    if attempts >= MAX_RECONNECT {
                        yield Err(ListenerError::ReconnectExhausted(attempts));
                        return;
                    }
                    tokio::time::sleep(backoff(attempts)).await;
                }

                Ok(provider) => {
                    match provider.subscribe_logs(&filter).await {
                        Err(e) => {
                            warn!("ws_stream: subscribe_logs failed: {e}");
                            yield Err(ListenerError::SubscribeFailed(e.to_string()));
                            attempts += 1;
                            if attempts >= MAX_RECONNECT {
                                yield Err(ListenerError::ReconnectExhausted(attempts));
                                return;
                            }
                            tokio::time::sleep(backoff(attempts)).await;
                        }

                        Ok(sub) => {
                            debug!("ws_stream: subscribed, receiving events");
                            attempts = 0; // reset the counter on a clean connection
                            use futures::StreamExt as _;
                            let mut inner = sub.into_stream();
                            while let Some(log) = inner.next().await {
                                yield Ok(log);
                            }
                            // Stream ended — connection dropped, schedule reconnect.
                            warn!("ws_stream: stream ended, scheduling reconnect");
                        }
                    }
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Connect to an Ethereum WebSocket endpoint and return a [`RootProvider`].
async fn connect_ws(
    url: &str,
) -> Result<alloy_provider::RootProvider<alloy_network::Ethereum>, String> {
    let connect = WsConnect::new(url);
    ProviderBuilder::default()
        .on_ws(connect)
        .await
        .map_err(|e| e.to_string())
}

/// Capped exponential back-off: 50 ms × 2ⁿ, max 10 doublings.
fn backoff(attempt: u32) -> Duration {
    Duration::from_millis(50 * (1u64 << attempt.min(10)))
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    //Lefty: is this the complete list of possibilities?
    fn listener_error_display() {
        assert_eq!(
            ListenerError::SubscribeFailed("timeout".into()).to_string(),
            "subscribe failed: timeout"
        );
        assert_eq!(
            ListenerError::ReconnectExhausted(3).to_string(),
            "WebSocket reconnect exhausted after 3 attempts"
        );
        assert_eq!(
            ListenerError::FilterInvalid("from > to".into()).to_string(),
            "invalid filter: from > to"
        );
    }

    #[test]
    fn poll_config_default_is_one_second() {
        let l = Listener::new("http://127.0.0.1:8545");
        assert_eq!(l.poll_interval, Duration::from_secs(1));
    }

    #[test]
    fn poll_interval_override() {
        let l = Listener::new("http://127.0.0.1:8545")
            .with_poll_interval(Duration::from_millis(250));
        assert_eq!(l.poll_interval, Duration::from_millis(250));
    }

    #[test]
    //Lefty: how does this work? How do I know we really got a ws here 
    // and an http in the following test?
    fn subscribe_returns_ws_stream_for_ws_scheme() {
        // subscribe() should not panic or error for either scheme; we verify
        // the stream is created without blocking.
        let _stream = Listener::new("ws://127.0.0.1:8545").subscribe(Filter::new());
    }

    #[test]
    fn subscribe_returns_http_stream_for_http_scheme() {
        let _stream = Listener::new("http://127.0.0.1:8545").subscribe(Filter::new());
    }

    #[test]
    //Lefty: maybe I misunderstood, I thoght we only do 3 retries?
    //Also, I am happy if we actually keep on retrying.
    fn backoff_values() {
        assert_eq!(backoff(0), Duration::from_millis(50));
        assert_eq!(backoff(1), Duration::from_millis(100));
        assert_eq!(backoff(2), Duration::from_millis(200));
        // cap at 2^10 = 1024
        assert_eq!(backoff(20), Duration::from_millis(50 * 1024));
    }
}
