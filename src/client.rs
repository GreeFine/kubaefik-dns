use std::net::*;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::TokioAsyncResolver;

use crate::config::RESOLVER;

pub async fn connect() {
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[IpAddr::V4(Ipv4Addr::new(194, 250, 191, 230))],
                53,
                true,
            ),
        ),
        ResolverOpts::default(),
    )
    .expect("failed to connect resolver");

    let mut resolver_static = RESOLVER.lock().await;
    *resolver_static = Some(resolver);
}

#[allow(clippy::await_holding_lock)]
pub async fn query(address: &str) -> Result<Vec<IpAddr>, ResolveError> {
    let mut resolver = RESOLVER.lock().await;
    let resolver = resolver.as_mut().unwrap();

    let lookup = resolver.lookup_ip(address).await;

    lookup.map(|lookup| lookup.into_iter().collect())
}
