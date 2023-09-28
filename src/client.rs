use log::{error, info};
use once_cell::sync::Lazy;
use std::net::*;
use trust_dns_resolver::error::ResolveError;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::{config::*, error::ResolveErrorKind};

use crate::config;

pub struct Resolvers {
    default: TokioAsyncResolver,
    failover: TokioAsyncResolver,
}

/// Resolver used to look up the IP address of address we didn't manage.
static RESOLVERS: Lazy<Resolvers> = Lazy::new(connect);

fn connect() -> Resolvers {
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[IpAddr::V4(config::DNS_SERVER_TO_QUERY)],
                53,
                true,
            ),
        ),
        ResolverOpts::default(),
    )
    .expect("failed to connect resolver");
    let failover_resolver = TokioAsyncResolver::tokio(
        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &[IpAddr::V4(config::DNS_SERVER_FAILOVER)],
                53,
                true,
            ),
        ),
        ResolverOpts::default(),
    )
    .expect("failed to connect resolver");
    Resolvers {
        default: resolver,
        failover: failover_resolver,
    }
}

#[allow(clippy::await_holding_lock)]
pub async fn query(address: &str) -> Result<Vec<IpAddr>, ResolveError> {
    let lookup = RESOLVERS.default.lookup_ip(address).await;

    match lookup {
        Ok(_) => lookup.map(|lookup| lookup.into_iter().collect()),
        Err(e) => {
            match e.kind() {
                ResolveErrorKind::NoRecordsFound { .. } => {
                    info!("Default lookup failed for {address} didn't find any records");
                }
                _ => {
                    error!("Default lookup failed");
                }
            };
            let failover_lookup = RESOLVERS.failover.lookup_ip(address).await;
            failover_lookup.map(|lookup| lookup.into_iter().collect())
        }
    }
}
