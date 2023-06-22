use std::{
    collections::HashMap,
    env,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use ::kube::Client;
use log::{error, info};
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

use crate::{client, kube, Error, Options};

/// DNS Request Handler
#[derive(Clone, Debug)]
pub struct Handler {
    ingresses: HashMap<String, Ipv4Addr>,
}

fn record_from_ip(name: Name, ip: &Ipv4Addr) -> Record {
    let rdata = RData::A(*ip);
    Record::from_rdata(name, 60, rdata)
}

impl Handler {
    /// Create new handler from command-line options.
    pub async fn from_options(_options: &Options, client_prod: Client, client_dev: Client) -> Self {
        let mut ingresses = HashMap::new();

        let prod_svc_name = env::var("traefik-svc-name");
        let prod_svc_name = prod_svc_name.as_deref().unwrap_or("traefik");
        let dev_svc_name = env::var("traefik-svc-name-dev");
        let dev_svc_name = dev_svc_name.as_deref().unwrap_or("traefik");

        for (client, svc_name) in [(client_prod, prod_svc_name), (client_dev, dev_svc_name)] {
            let tf_address = kube::get_traefik_addr(client.clone(), svc_name).await;

            let tf_address_ip = Ipv4Addr::from_str(&tf_address).expect("parsed tf ip address");
            let ingress_names = kube::get_ingress_names(client).await;
            for mut ingress_name in ingress_names {
                // adding the trailing dot from the DNS spec
                ingress_name.push('.');
                ingresses.insert(ingress_name, tf_address_ip);
            }
        }
        Handler { ingresses }
    }

    async fn handle_query<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> ResponseInfo {
        let name = request.query().name().to_string();
        info!("query for: {name}");
        let mut records = Vec::new();
        if let Some(address) = self.ingresses.get(&name) {
            let record = record_from_ip(request.query().name().into(), address);
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
