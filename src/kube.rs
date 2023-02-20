use std::env;

use k8s_openapi::api::{core::v1::Service, networking::v1::Ingress};
use kube::{api::ListParams, Api, Client};

pub async fn clients() -> (Client, Client) {
    let client_dev = Client::try_default().await.expect("kube client");

    // This is a very hacky way of having 1 client defined by the cluster and one from a yaml config file
    std::env::remove_var("KUBECONFIG");
    let client = Client::try_default().await.expect("kube client");

    (client, client_dev)
}

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
