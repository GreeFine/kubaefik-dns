use crate::{kube, Error, Options};
use log::{error, info};
use std::{collections::HashMap, net::Ipv4Addr, str::FromStr};
use trust_dns_server::{
    authority::MessageResponseBuilder,
    proto::{
        op::{Header, MessageType, OpCode, ResponseCode},
        rr::RData,
        rr::Record,
    },
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};

/// DNS Request Handler
#[derive(Clone, Debug)]
pub struct Handler {
    ingresses: HashMap<String, Ipv4Addr>,
}

impl Handler {
    /// Create new handler from command-line options.
    pub async fn from_options(_options: &Options) -> Self {
        let mut ingresses = HashMap::new();
        let tf_address = kube::get_traefik_addr().await;
        let tf_address_ip = Ipv4Addr::from_str(&tf_address).expect("parsed tf ip address");
        let ingress_names = kube::get_ingress_names().await;
        for mut ingress_name in ingress_names {
            // adding the trailing dot from the DNS spec
            ingress_name.push('.');
            ingresses.insert(ingress_name, tf_address_ip);
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
            let rdata = RData::A(*address);
            records.push(Record::from_rdata(request.query().name().into(), 60, rdata))
        } else {
            error!("name: {name} not found in ingresses")
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
