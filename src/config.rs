use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use trust_dns_resolver::TokioAsyncResolver;

/// Timeout for TCP connections.
pub const TCP_TIMEOUT: Duration = Duration::from_secs(10);

pub static RESOLVER: Lazy<Mutex<Option<TokioAsyncResolver>>> = Lazy::new(|| Mutex::new(None));

pub const STATE_REFRESH_MINUTES: i64 = 5;
