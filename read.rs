use std::collections::HashMap;

impl Graph {
    fn delete_edges_locked(
        &mut self,
        from_type: VertexType,
        to_type: VertexType,
        to_namespace: &str,
        to_name: &str,
    ) {
        let to_vert = match self.get_vertex_r_locked(to_type, to_namespace, to_name) {
            Some(v) => v,
            None => return,
        };

        let mut neighbors_to_remove = Vec::new();
        let mut edges_to_remove = Vec::new();

        self.graph.visit_to(to_vert, |from| {
            let from_vert = from.as_named_vertex();
            if from_vert.vertex_type != from_type {
                return true;
            }

            if self.graph.degree(from_vert) == 1 {
                neighbors_to_remove.push(from_vert.clone());
            } else {
                edges_to_remove.push(self.graph.edge_between(from, to_vert));
            }
            true
        });

        for v in neighbors_to_remove {
            self.remove_vertex_locked(&v);
        }

        for edge in edges_to_remove {
            self.graph.remove_edge(&edge);
            self.remove_edge_from_destination_index_locked(&edge);
        }
    }

    fn remove_edge_from_destination_index_locked(&mut self, e: &dyn Edge) {
        let n = e.from();
        let edge_count = self.graph.degree(n);

        if edge_count < self.destination_edge_threshold {
            self.destination_edge_index.remove(&n.id());
            return;
        }

        if let Some(index) = self.destination_edge_index.get_mut(&n.id()) {
            if let Some(destination_edge) = (e as &dyn std::any::Any).downcast_ref::<DestinationEdge>() {
                index.decrement(destination_edge.destination_id());
            }
        }
    }

    fn add_edge_to_destination_index_locked(&mut self, e: &dyn Edge) {
        let n = e.from();
        let n_id = n.id();

        if !self.destination_edge_index.contains_key(&n_id) {
            self.recompute_destination_index_locked(n);
            return;
        }

        if let Some(index) = self.destination_edge_index.get_mut(&n_id) {
            if let Some(destination_edge) = (e as &dyn std::any::Any).downcast_ref::<DestinationEdge>() {
                index.increment(destination_edge.destination_id());
            }
        }
    }

    fn remove_vertex_locked(&mut self, v: &NamedVertex) {
        self.graph.remove_node(v);
        self.destination_edge_index.remove(&v.id());

        if let Some(namespace_map) = self.vertices.get_mut(&v.vertex_type) {
            if let Some(name_map) = namespace_map.get_mut(&v.namespace) {
                name_map.remove(&v.name);
                if name_map.is_empty() {
                    namespace_map.remove(&v.namespace);
                }
            }
        }
    }

    fn recompute_destination_index_locked(&mut self, n: &dyn Node) {
        let edge_count = self.graph.degree(n);
        let n_id = n.id();

        if edge_count < self.destination_edge_threshold {
            self.destination_edge_index.remove(&n_id);
            return;
        }

        let index = self.destination_edge_index
            .entry(n_id)
            .or_insert_with(|| IntSet::new());
        
        if self.destination_edge_index.contains_key(&n_id) {
            index.reset();
        }

        self.graph.visit_from(n, |dest| {
            if let Some(edge) = self.graph.edge_between(n, dest) {
                if let Some(destination_edge) = (edge as &dyn std::any::Any).downcast_ref::<DestinationEdge>() {
                    index.increment(destination_edge.destination_id());
                }
            }
            true
        });
    }
}

struct DestinationEdge {
    f: Box<dyn Node>,
    t: Box<dyn Node>,
    destination: Box<dyn Node>,
}

impl DestinationEdge {
    fn destination_id(&self) -> i32 {
        self.destination.id()
    }
}

impl Edge for DestinationEdge {
    fn from(&self) -> &dyn Node {
        self.f.as_ref()
    }

    fn to(&self) -> &dyn Node {
        self.t.as_ref()
    }

    fn weight(&self) -> f64 {
        0.0
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct NamedVertex {
    name: String,
    namespace: String,
    id: i32,
    vertex_type: VertexType,
}

impl Node for NamedVertex {
    fn id(&self) -> i32 {
        self.id
    }
}

struct Graph {
    graph: DirectedAcyclicGraph,
    vertices: HashMap<VertexType, NamespaceVertexMapping>,
    destination_edge_index: HashMap<i32, IntSet>,
    destination_edge_threshold: usize,
}

trait Node {
    fn id(&self) -> i32;
}

trait Edge {
    fn from(&self) -> &dyn Node;
    fn to(&self) -> &dyn Node;
    fn weight(&self) -> f64;
}

trait GraphTrait {
    fn has(&self, node: &dyn Node) -> bool;
    fn nodes(&self) -> Vec<&dyn Node>;
    fn from(&self, node: &dyn Node) -> Vec<&dyn Node>;
    fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool;
    fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<&dyn Edge>;
}
