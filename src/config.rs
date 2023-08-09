use std::{net::Ipv4Addr, time::Duration};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use trust_dns_resolver::TokioAsyncResolver;

/// Timeout for TCP connections.
pub const TCP_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolver used to look up the IP address of address we didn't manage.
pub static RESOLVER: Lazy<Mutex<Option<TokioAsyncResolver>>> = Lazy::new(|| Mutex::new(None));

#[cfg(debug_assertions)]
pub const STATE_REFRESH_MINUTES: i64 = 1;
#[cfg(not(debug_assertions))]
pub const STATE_REFRESH_MINUTES: i64 = 5;

/// Address of the server used to query after we fail to find an answer in our map
pub const DNS_SERVER_TO_QUERY: Ipv4Addr = Ipv4Addr::new(194, 250, 191, 230);
