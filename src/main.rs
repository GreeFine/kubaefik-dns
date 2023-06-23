#![warn(unused_extern_crates)]

use anyhow::Result;
use clap::Parser;
use error::Error;
use handler::Handler;
use log::info;
use options::{Options, TestOption};
use std::env;
use tokio::net::{TcpListener, UdpSocket};
use trust_dns_server::ServerFuture;

use crate::config::TCP_TIMEOUT;

mod client;
mod config;
mod error;
mod handler;
mod kube;
mod options;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::init();

    let options = Options::parse();
    let (client, client_dev) = kube::clients().await;

    if let Some(test) = options.test_mode {
        match test {
            TestOption::GetTraefik => {
                let prod_svc_name = env::var("traefik-svc-name");
                let prod_svc_name = prod_svc_name.as_deref().unwrap_or("traefik");
                info!("{}", kube::get_traefik_addr(client, prod_svc_name).await)
            }
            TestOption::GetIngressNames => info!("{:#?}", kube::get_ingress_names(client).await),
        };
        return Ok(());
    }
    client::connect().await;

    info!("DNS server listening on:");

    let handler = Handler::from_options(&options, client, client_dev).await;

    // create DNS server
    let mut server = ServerFuture::new(handler);

    // register UDP listeners
    for udp in &options.udp {
        server.register_socket(UdpSocket::bind(udp).await?);
        info!("UDP: {udp}");
    }

    // register TCP listeners
    for tcp in &options.tcp {
        server.register_listener(TcpListener::bind(&tcp).await?, TCP_TIMEOUT);
        info!("TCP: {tcp}");
    }

    // run DNS server
    server.block_until_done().await?;

    Ok(())
}
