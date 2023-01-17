use k8s_openapi::api::{core::v1::Service, networking::v1::Ingress};
use kube::{api::ListParams, Api, Client};

pub async fn get_traefik_addr() -> String {
    let client = Client::try_default().await.expect("kube client");
    let services: Api<Service> = Api::namespaced(client, "traefik");

    let service = services
        .get("neg-traefik")
        .await
        .expect("find service of traefik");
    service
        .spec
        .unwrap()
        .cluster_ip
        .expect("get cluster ip for traefik")
}

pub async fn get_ingress_names() -> Vec<String> {
    let client = Client::try_default().await.expect("kube client");
    let ingresses: Api<Ingress> = Api::all(client);

    let params = ListParams::default();
    let ingresses = ingresses
        .list(&params)
        .await
        .expect("ingresses in the cluster");

    // TODO: Filter ingress to include only the one using traefik
    ingresses
        .into_iter()
        .filter_map(|ingress| {
            ingress.spec.and_then(|spec| {
                spec.rules
                    .map(|rules| rules.into_iter().map(|rules| rules.host))
            })
        })
        .flatten()
        .flatten()
        .collect()
}
