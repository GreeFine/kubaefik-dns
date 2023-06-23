use std::{collections::HashMap, env, net::Ipv4Addr, str::FromStr};

use k8s_openapi::api::{core::v1::Service, networking::v1::Ingress};
use kube::{api::ListParams, Api, Client};

pub async fn clients() -> (Client, Client) {
    let client_dev = Client::try_default().await.expect("kube client");

    // This is a very hacky way of having 1 client defined by the cluster and one from a yaml config file
    std::env::remove_var("KUBECONFIG");
    let client = Client::try_default().await.expect("kube client");

    (client, client_dev)
}

/// Using [get_ingress_names] and [get_traefik_addr] make a map of URLs -> Ip address for the DNS to serve
pub async fn get_ingresses(client_prod: Client, client_dev: Client) -> HashMap<String, Ipv4Addr> {
    let prod_svc_name = env::var("traefik-svc-name");
    let prod_svc_name = prod_svc_name.as_deref().unwrap_or("traefik");
    let dev_svc_name = env::var("traefik-svc-name-dev");
    let dev_svc_name = dev_svc_name.as_deref().unwrap_or("traefik");

    let mut ingresses = HashMap::new();
    for (client, svc_name) in [(client_prod, prod_svc_name), (client_dev, dev_svc_name)] {
        let tf_address = get_traefik_addr(client.clone(), svc_name).await;

        let tf_address_ip = Ipv4Addr::from_str(&tf_address).expect("parsed tf ip address");
        let ingress_names = get_ingress_names(client).await;
        for mut ingress_name in ingress_names {
            // adding the trailing dot from the DNS spec
            ingress_name.push('.');
            ingresses.insert(ingress_name, tf_address_ip);
        }
    }
    ingresses
}

/// Get the IP address of the traefik in the cluster
pub async fn get_traefik_addr(client: Client, service_name: &str) -> String {
    let services: Api<Service> = Api::namespaced(
        client,
        env::var("traefik-ns-name").as_deref().unwrap_or("traefik"),
    );

    let service = services
        .get(service_name)
        .await
        .expect("find service of traefik");
    service
        .spec
        .unwrap()
        .cluster_ip
        .expect("get cluster ip for traefik")
}

/// List all the ingresses URLs that traefik manage
pub async fn get_ingress_names(client: Client) -> Vec<String> {
    let ingresses: Api<Ingress> = Api::all(client);

    let params = ListParams::default();
    let ingresses = ingresses
        .list(&params)
        .await
        .expect("ingresses in the cluster");

    ingresses
        .into_iter()
        .filter_map(|ingress| {
            ingress.metadata.annotations.and_then(|anotations| {
                (Some("traefik")
                    == anotations
                        .get("kubernetes.io/ingress.class")
                        .map(|a| a.as_str()))
                .then(|| {
                    ingress.spec.and_then(|spec| {
                        spec.rules
                            .map(|rules| rules.into_iter().map(|rules| rules.host))
                    })
                })
            })
        })
        .flatten()
        .flatten()
        .flatten()
        .collect()
}
