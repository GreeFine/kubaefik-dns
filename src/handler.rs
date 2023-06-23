use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

use ::kube::Client;
use chrono::{Duration, NaiveDateTime, Utc};
use log::{error, info};
use tokio::sync::RwLock;
use trust_dns_resolver::Name;
use trust_dns_server::{
    authority::MessageResponseBuilder,
    proto::{
        op::{Header, MessageType, OpCode, ResponseCode},
        rr::RData,
        rr::Record,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};

use crate::{client, config::STATE_REFRESH_MINUTES, kube, Error, Options};

/// DNS Request Handler
pub struct Handler {
    ingresses: RwLock<HashMap<String, Ipv4Addr>>,
    kube_client_prod: Client,
    kube_client_dev: Client,
    updated_at: NaiveDateTime,
}

fn record_from_ip(name: Name, ip: &Ipv4Addr) -> Record {
    let rdata = RData::A(*ip);
    Record::from_rdata(name, 60, rdata)
}

impl Handler {
    /// Create new handler from command-line options.
    pub async fn from_options(_options: &Options, client_prod: Client, client_dev: Client) -> Self {
        let ingresses =
            RwLock::new(kube::get_ingresses(client_prod.clone(), client_dev.clone()).await);

        Handler {
            ingresses,
            kube_client_prod: client_prod,
            kube_client_dev: client_dev,
            updated_at: Utc::now().naive_utc(),
        }
    }

    async fn refresh_state(&self) {
        if self
            .updated_at
            .signed_duration_since(Utc::now().naive_utc())
            > Duration::minutes(STATE_REFRESH_MINUTES)
        {
            let mut ingress = self.ingresses.write().await;
            *ingress =
                kube::get_ingresses(self.kube_client_prod.clone(), self.kube_client_dev.clone())
                    .await;
        }
    }

    async fn handle_query<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> ResponseInfo {
        self.refresh_state().await;

        let name = request.query().name().to_string();
        info!("query for: {name}");
        let mut records = Vec::new();
        let local_record = {
            let ingresses_r = self.ingresses.read().await;
            ingresses_r
                .get(&name)
                .map(|addr| record_from_ip(request.query().name().into(), addr))
        };
        if let Some(record) = local_record {
            records.push(record);
        } else {
            error!("name: {name} not found in ingresses");
            let result = client::query(&name).await.expect("address query result");
            records.append(
                &mut result
                    .into_iter()
                    .filter_map(|ip| match ip {
                        IpAddr::V4(ip) => Some(record_from_ip(request.query().name().into(), &ip)),
                        IpAddr::V6(_) => None,
                    })
                    .collect(),
            );
        }
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(false);
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        responder
            .send_response(response)
            .await
            .expect("sending response")
    }

    fn send_error(error: Error) -> ResponseInfo {
        error!("Error in RequestHandler: {error}");
        let mut header = Header::new();
        header.set_response_code(ResponseCode::ServFail);
        header.into()
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        responder: R,
    ) -> ResponseInfo {
        if request.op_code() != OpCode::Query {
            return Handler::send_error(Error::InvalidOpCode(request.op_code()));
        }
        if request.message_type() != MessageType::Query {
            return Handler::send_error(Error::InvalidMessageType(request.message_type()));
        }

        self.handle_query(request, responder).await
    }
}
