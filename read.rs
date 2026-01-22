use std::time::Instant;

impl Graph {
    pub fn add_pod(&mut self, pod: &Pod) {
        let start = Instant::now();
        let _timer = scopeguard::guard((), |_| {
            GRAPH_ACTIONS_DURATION
                .with_label_values(&["AddPod"])
                .observe(start.elapsed().as_secs_f64());
        });

        self.delete_vertex_locked(VertexType::Pod, &pod.namespace, &pod.name);
        let pod_vertex = self.get_or_create_vertex_locked(
            VertexType::Pod,
            &pod.namespace,
            &pod.name,
        );
        let node_vertex = self.get_or_create_vertex_locked(
            VertexType::Node,
            "",
            &pod.spec.node_name,
        );
        self.add_edge_locked(Some(pod_vertex), Some(node_vertex), Some(node_vertex));

        if pod.annotations.contains_key(MIRROR_POD_ANNOTATION_KEY) {
            return;
        }

        if !pod.spec.service_account_name.is_empty() {
            let service_account_vertex = self.get_or_create_vertex_locked(
                VertexType::ServiceAccount,
                &pod.namespace,
                &pod.spec.service_account_name,
            );
            self.add_edge_locked(
                Some(service_account_vertex),
                Some(pod_vertex),
                Some(node_vertex),
            );
        }

        visit_pod_secret_names(pod, |secret| {
            let secret_vertex = self.get_or_create_vertex_locked(
                VertexType::Secret,
                &pod.namespace,
                secret,
            );
            self.add_edge_locked(Some(secret_vertex), Some(pod_vertex), Some(node_vertex));
            true
        });

        visit_pod_configmap_names(pod, |configmap| {
            let configmap_vertex = self.get_or_create_vertex_locked(
                VertexType::ConfigMap,
                &pod.namespace,
                configmap,
            );
            self.add_edge_locked(Some(configmap_vertex), Some(pod_vertex), Some(node_vertex));
            true
        });

        for v in &pod.spec.volumes {
            let claim_name = if let Some(pvc) = &v.persistent_volume_claim {
                Some(pvc.claim_name.clone())
            } else if let Some(ephemeral) = &v.ephemeral {
                Some(ephemeral::volume_claim_name(pod, v))
            } else {
                None
            };

            if let Some(claim_name) = claim_name {
                if !claim_name.is_empty() {
                    let pvc_vertex = self.get_or_create_vertex_locked(
                        VertexType::Pvc,
                        &pod.namespace,
                        &claim_name,
                    );
                    self.add_edge_locked(Some(pvc_vertex), Some(pod_vertex), Some(node_vertex));
                }
            }
        }

        for pod_resource_claim in &pod.spec.resource_claims {
            if let Ok(Some(claim_name)) = resourceclaim::name(pod, pod_resource_claim) {
                let claim_vertex = self.get_or_create_vertex_locked(
                    VertexType::ResourceClaim,
                    &pod.namespace,
                    &claim_name,
                );
                self.add_edge_locked(Some(claim_vertex), Some(pod_vertex), Some(node_vertex));
            }
        }

        if let Some(extended_status) = &pod.status.extended_resource_claim_status {
            if !extended_status.resource_claim_name.is_empty() {
                let claim_vertex = self.get_or_create_vertex_locked(
                    VertexType::ResourceClaim,
                    &pod.namespace,
                    &extended_status.resource_claim_name,
                );
                self.add_edge_locked(Some(claim_vertex), Some(pod_vertex), Some(node_vertex));
            }
        }
    }

    fn add_edge_locked(
        &mut self,
        from: Option<&NamedVertex>,
        to: Option<&NamedVertex>,
        destination: Option<&NamedVertex>,
    ) {
        let from = from.expect("from vertex cannot be None");
        let to = to.expect("to vertex cannot be None");

        if let Some(dest) = destination {
            let e = new_destination_edge(from, to, dest);
            self.graph.set_edge(e.clone());
            self.add_edge_to_destination_index_locked(&e);
            return;
        }

        if VERTEX_TYPE_WITH_AUTHORITATIVE_INDEX.contains(&from.vertex_type) {
            panic!(
                "vertex of type {:?} must have destination edges only",
                from.vertex_type
            );
        }
        self.graph.set_edge(SimpleEdge {
            f: from.clone(),
            t: to.clone(),
        });
    }

    pub fn delete_pod(&mut self, name: &str, namespace: &str) {
        let start = Instant::now();
        let _timer = scopeguard::guard((), |_| {
            GRAPH_ACTIONS_DURATION
                .with_label_values(&["DeletePod"])
                .observe(start.elapsed().as_secs_f64());
        });

        self.delete_vertex_locked(VertexType::Pod, namespace, name);
    }
}

fn new_destination_edge(
    from: &NamedVertex,
    to: &NamedVertex,
    destination: &NamedVertex,
) -> DestinationEdge {
    DestinationEdge {
        f: Box::new(from.clone()),
        t: Box::new(to.clone()),
        destination: Box::new(destination.clone()),
    }
}

#[derive(Clone)]
struct SimpleEdge {
    f: NamedVertex,
    t: NamedVertex,
}

impl Edge for SimpleEdge {
    fn from(&self) -> &dyn Node {
        &self.f
    }

    fn to(&self) -> &dyn Node {
        &self.t
    }

    fn weight(&self) -> f64 {
        1.0
    }
}

fn visit_pod_secret_names<F>(pod: &Pod, mut f: F)
where
    F: FnMut(&str) -> bool,
{
    for secret in &pod.spec.secrets {
        if !f(&secret.name) {
            break;
        }
    }
}

fn visit_pod_configmap_names<F>(pod: &Pod, mut f: F)
where
    F: FnMut(&str) -> bool,
{
    for configmap in &pod.spec.configmaps {
        if !f(&configmap.name) {
            break;
        }
    }
}

const MIRROR_POD_ANNOTATION_KEY: &str = "kubernetes.io/config.mirror";

static VERTEX_TYPE_WITH_AUTHORITATIVE_INDEX: &[VertexType] = &[
    VertexType::Secret,
    VertexType::ConfigMap,
    VertexType::Pvc,
    VertexType::ServiceAccount,
    VertexType::ResourceClaim,
];
