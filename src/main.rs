use anyhow::Result;
use clap::Parser;
use error::Error;
use handler::Handler;
use log::info;
use options::{Options, TestOption};
use std::{env, time::Duration};
use tokio::net::{TcpListener, UdpSocket};
use trust_dns_server::ServerFuture;

mod error;
mod handler;
mod kube;
mod options;

/// Timeout for TCP connections.
const TCP_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init();

    let options = Options::parse();

    if let Some(test) = options.test_mode {
        match test {
            TestOption::GetIngressNames => info!("{:#?}", kube::get_ingress_names().await),
            TestOption::GetTraefik => info!("{}", kube::get_traefik_addr().await),
        };
        return Ok(());
    }

    let handler = Handler::from_options(&options).await;

    // create DNS server
    let mut server = ServerFuture::new(handler);

    // register UDP listeners
    for udp in &options.udp {
        server.register_socket(UdpSocket::bind(udp).await?);
    }

    // register TCP listeners
    for tcp in &options.tcp {
        server.register_listener(TcpListener::bind(&tcp).await?, TCP_TIMEOUT);
    }

    // run DNS server
    server.block_until_done().await?;

    Ok(())
}
