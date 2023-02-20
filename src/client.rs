use once_cell::sync::Lazy;
use std::net::*;
use tokio::sync::Mutex;
use trust_dns_resolver::config::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::TokioAsyncResolver;

static RESOLVER: Lazy<Mutex<Option<TokioAsyncResolver>>> = Lazy::new(|| Mutex::new(None));

pub async fn connect() {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())
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
