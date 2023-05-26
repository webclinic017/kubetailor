use serde_json::{json, Value};

use crate::prelude::*;

pub static KUBETAILOR_FINALIZER: &str = "tailoredapps.mpw.sh";

pub async fn add(client: Client, namespace: &str, name: &str) -> Result<TailoredApp, Error> {
    let api: Api<TailoredApp> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": [KUBETAILOR_FINALIZER]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}

pub async fn delete(client: Client, namespace: &str, name: &str) -> Result<TailoredApp, Error> {
    let api: Api<TailoredApp> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}
