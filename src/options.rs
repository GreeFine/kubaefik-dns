use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    #[clap(long, default_value = "0.0.0.0:1053", env = "DNSFUN_UDP")]
    pub udp: Vec<SocketAddr>,

    #[clap(long, env = "DNSFUN_TCP")]
    pub tcp: Vec<SocketAddr>,

    #[clap(long)]
    pub test_mode: Option<TestOption>,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum TestOption {
    GetTraefik,
    GetIngressNames,
}
