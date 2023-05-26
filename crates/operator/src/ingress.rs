use k8s_openapi::api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, IngressBackend, IngressRule, IngressServiceBackend,
    IngressSpec, IngressTLS, ServiceBackendPort,
};

use crate::prelude::*;

fn new(namespace: &str, name: &str, app: &TailoredApp) -> Ingress {
    let app = app.spec.clone();
    let paths = vec![HTTPIngressPath {
        path: Some(String::from("/")),
        path_type: String::from("Prefix"),
        backend: IngressBackend {
            resource: None,
            service: Some(IngressServiceBackend {
                name: String::from(name),
                port: Some(ServiceBackendPort {
                    name: None,
                    number: Some(app.deployment.container.port),
                }),
            }),
        },
    }];
    let mut domains = Vec::new();
    domains.push(app.ingress.domains.shared.clone());
    // If there is a custom domain, add it to the hosts vector
    if let Some(custom_domain) = &app.ingress.domains.custom {
        domains.push(custom_domain.clone());
    }

    //create ingress rules
    let rules: Vec<IngressRule> = domains
        .iter()
        .map(|host| IngressRule {
            host: Some(host.to_owned()),
            http: Some(HTTPIngressRuleValue {
                paths: paths.clone(),
            }),
        })
        .collect();

    Ingress {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            namespace: Some(namespace.to_owned()),
            labels: Some(app.labels.clone()),
            annotations: Some(app.ingress.annotations.clone()),
            ..ObjectMeta::default()
        },
        status: None,
        spec: Some(IngressSpec {
            default_backend: None,
            ingress_class_name: Some(app.ingress.class_name.clone()),
            rules: { Some(rules) },
            tls: Some(vec![IngressTLS {
                hosts: Some(domains),
                secret_name: Some(format!(
                    "{}-{}-kubetailor-tls",
                    name,
                    app.labels.get("owner").unwrap()
                )),
            }]),
        }),
    }
}

pub async fn deploy(
    client: Client,
    namespace: &str,
    name: &str,
    app: &TailoredApp,
) -> Result<Ingress, Error> {
    let ingress = new(namespace, name, app);
    let api: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    match api.create(&PostParams::default(), &ingress).await {
        Ok(s) => Ok(s),
        Err(kube::Error::Api(e)) if e.code == 409 => update(client, namespace, name, app).await,
        Err(e) => Err(Error::KubeError { source: e }),
    }
}

pub async fn update(
    client: Client,
    namespace: &str,
    name: &str,
    app: &TailoredApp,
) -> Result<Ingress, Error> {
    let mut ingress = new(namespace, name, &app.clone());
    let api: Api<Ingress> = Api::namespaced(client, namespace);
    let resource_version = api.get(name).await?.metadata.resource_version;
    ingress.metadata.resource_version = resource_version;

    Ok(api.replace(name, &PostParams::default(), &ingress).await?)
}

pub async fn delete(client: Client, namespace: &str, name: &str) -> Result<(), Error> {
    let api: Api<Ingress> = Api::namespaced(client, namespace);
    match api.delete(name, &DeleteParams::default()).await {
        Ok(_) => Ok(()),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Ingress {name} already deleted");
            Ok(())
        },
        Err(e) => Err(Error::KubeError { source: e }),
    }
}

pub async fn exists(client: Client, namespace: &str, name: &str) -> Result<bool, Error> {
    let api: Api<Ingress> = Api::namespaced(client, namespace);
    match api.get(name).await {
        Ok(_) => Ok(true),
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(false),
        Err(e) => Err(Error::KubeError { source: e }),
    }
}
