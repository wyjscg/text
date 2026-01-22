use std::sync::RwLock;
use std::time::Instant;

// Graph 结构体 - 不包含锁，只有数据和需要在锁内执行的方法
struct Graph {
    graph: DirectedAcyclicGraph,
    vertices: HashMap<VertexType, NamespaceVertexMapping>,
    destination_edge_index: HashMap<i32, IntSet>,
    destination_edge_threshold: usize,
}

impl Graph {
    // 这些方法假设调用者已经持有锁
    fn delete_edges_locked(
        &mut self,
        from_type: VertexType,
        to_type: VertexType,
        to_namespace: &str,
        to_name: &str,
    ) {
        // ... 之前翻译的代码
    }

    fn remove_edge_from_destination_index_locked(&mut self, e: &dyn Edge) {
        // ... 之前翻译的代码
    }

    fn add_edge_to_destination_index_locked(&mut self, e: &dyn Edge) {
        // ... 之前翻译的代码
    }

    fn remove_vertex_locked(&mut self, v: &NamedVertex) {
        // ... 之前翻译的代码
    }

    fn recompute_destination_index_locked(&mut self, n: &dyn Node) {
        // ... 之前翻译的代码
    }

    fn delete_vertex_locked(&mut self, vertex_type: VertexType, namespace: &str, name: &str) {
        // ... 实现
    }

    fn get_or_create_vertex_locked(
        &mut self,
        vertex_type: VertexType,
        namespace: &str,
        name: &str,
    ) -> &NamedVertex {
        // ... 实现
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
}

// GraphLock 结构体 - 包含锁和对外的公共 API
pub struct GraphLock {
    inner: RwLock<Graph>,
}

impl GraphLock {
    pub fn new(
        destination_edge_threshold: usize,
    ) -> Self {
        Self {
            inner: RwLock::new(Graph {
                graph: DirectedAcyclicGraph::new(),
                vertices: HashMap::new(),
                destination_edge_index: HashMap::new(),
                destination_edge_threshold,
            }),
        }
    }

    pub fn add_pod(&self, pod: &Pod) {
        let start = Instant::now();
        let _timer = scopeguard::guard((), |_| {
            GRAPH_ACTIONS_DURATION
                .with_label_values(&["AddPod"])
                .observe(start.elapsed().as_secs_f64());
        });

        let mut graph = self.inner.write().unwrap();

        graph.delete_vertex_locked(VertexType::Pod, &pod.namespace, &pod.name);
        let pod_vertex = graph.get_or_create_vertex_locked(
            VertexType::Pod,
            &pod.namespace,
            &pod.name,
        );
        let node_vertex = graph.get_or_create_vertex_locked(
            VertexType::Node,
            "",
            &pod.spec.node_name,
        );
        graph.add_edge_locked(Some(pod_vertex), Some(node_vertex), Some(node_vertex));

        if pod.annotations.contains_key(MIRROR_POD_ANNOTATION_KEY) {
            return;
        }

        if !pod.spec.service_account_name.is_empty() {
            let service_account_vertex = graph.get_or_create_vertex_locked(
                VertexType::ServiceAccount,
                &pod.namespace,
                &pod.spec.service_account_name,
            );
            graph.add_edge_locked(
                Some(service_account_vertex),
                Some(pod_vertex),
                Some(node_vertex),
            );
        }

        visit_pod_secret_names(pod, |secret| {
            let secret_vertex = graph.get_or_create_vertex_locked(
                VertexType::Secret,
                &pod.namespace,
                secret,
            );
            graph.add_edge_locked(Some(secret_vertex), Some(pod_vertex), Some(node_vertex));
            true
        });

        visit_pod_configmap_names(pod, |configmap| {
            let configmap_vertex = graph.get_or_create_vertex_locked(
                VertexType::ConfigMap,
                &pod.namespace,
                configmap,
            );
            graph.add_edge_locked(Some(configmap_vertex), Some(pod_vertex), Some(node_vertex));
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
                    let pvc_vertex = graph.get_or_create_vertex_locked(
                        VertexType::Pvc,
                        &pod.namespace,
                        &claim_name,
                    );
                    graph.add_edge_locked(Some(pvc_vertex), Some(pod_vertex), Some(node_vertex));
                }
            }
        }

        for pod_resource_claim in &pod.spec.resource_claims {
            if let Ok(Some(claim_name)) = resourceclaim::name(pod, pod_resource_claim) {
                let claim_vertex = graph.get_or_create_vertex_locked(
                    VertexType::ResourceClaim,
                    &pod.namespace,
                    &claim_name,
                );
                graph.add_edge_locked(Some(claim_vertex), Some(pod_vertex), Some(node_vertex));
            }
        }

        if let Some(extended_status) = &pod.status.extended_resource_claim_status {
            if !extended_status.resource_claim_name.is_empty() {
                let claim_vertex = graph.get_or_create_vertex_locked(
                    VertexType::ResourceClaim,
                    &pod.namespace,
                    &extended_status.resource_claim_name,
                );
                graph.add_edge_locked(Some(claim_vertex), Some(pod_vertex), Some(node_vertex));
            }
        }
    }

    pub fn delete_pod(&self, name: &str, namespace: &str) {
        let start = Instant::now();
        let _timer = scopeguard::guard((), |_| {
            GRAPH_ACTIONS_DURATION
                .with_label_values(&["DeletePod"])
                .observe(start.elapsed().as_secs_f64());
        });

        let mut graph = self.inner.write().unwrap();
        graph.delete_vertex_locked(VertexType::Pod, namespace, name);
    }
}
