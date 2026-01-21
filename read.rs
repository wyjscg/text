use std::collections::HashMap;
use std::sync::RwLock;

pub struct Graph {
    lock: RwLock<GraphInner>,
}

struct GraphInner {
    graph: DirectedAcyclicGraph,
    vertices: HashMap<VertexType, NamespaceVertexMapping>,
    destination_edge_index: HashMap<i64, IntSet>,
    destination_edge_threshold: usize,
}

type NamespaceVertexMapping = HashMap<String, NameVertexMapping>;

type NameVertexMapping = HashMap<String, Box<NamedVertex>>;

impl Graph {
    pub fn new() -> Self {
        Graph {
            lock: RwLock::new(GraphInner {
                vertices: HashMap::new(),
                graph: DirectedAcyclicGraph::new(0, 0),
                destination_edge_index: HashMap::new(),
                destination_edge_threshold: 200,
            }),
        }
    }
}

lazy_static::lazy_static! {
    static ref VERTEX_TYPE_WITH_AUTHORITATIVE_INDEX: HashMap<VertexType, bool> = {
        let mut m = HashMap::new();
        m.insert(VertexType::ConfigMap, true);
        m.insert(VertexType::Slice, true);
        m.insert(VertexType::Pod, true);
        m.insert(VertexType::Pvc, true);
        m.insert(VertexType::ResourceClaim, true);
        m.insert(VertexType::Va, true);
        m.insert(VertexType::ServiceAccount, true);
        m.insert(VertexType::Pcr, true);
        m
    };
}
