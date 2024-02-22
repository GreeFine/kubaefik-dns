use std::{net::Ipv4Addr, time::Duration};

/// Timeout for TCP connections.
pub const TCP_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(debug_assertions)]
pub const STATE_REFRESH_MINUTES: i64 = 1;
#[cfg(not(debug_assertions))]
pub const STATE_REFRESH_MINUTES: i64 = 5;

/// Address of the server used to query after we fail to find an answer in our map
pub const DNS_SERVER_TO_QUERY: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
pub const DNS_SERVER_FAILOVER: Ipv4Addr = Ipv4Addr::new(145, 239, 186, 86);
