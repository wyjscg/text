pub trait Node {
    fn id(&self) -> i64;
}

pub trait Edge {
    fn from(&self) -> &dyn Node;
    fn to(&self) -> &dyn Node;
    fn weight(&self) -> f64;
}

pub trait Graph {
    fn has(&self, node: &dyn Node) -> bool;

    fn nodes(&self) -> Vec<Box<dyn Node>>;

    fn from(&self, node: &dyn Node) -> Vec<Box<dyn Node>>;

    fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool;

    fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<Box<dyn Edge>>;
}

pub trait Undirected: Graph {
    fn edge_between(&self, x: &dyn Node, y: &dyn Node) -> Option<Box<dyn Edge>>;
}

pub trait Directed: Graph {
    fn has_edge_from_to(&self, u: &dyn Node, v: &dyn Node) -> bool;

    fn to(&self, node: &dyn Node) -> Vec<Box<dyn Node>>;
}

pub trait Weighter {
    fn weight(&self, x: &dyn Node, y: &dyn Node) -> Option<f64>;
}

pub trait NodeAdder {
    fn new_node_id(&mut self) -> i64;

    fn add_node(&mut self, node: Box<dyn Node>);
}

pub trait NodeRemover {
    fn remove_node(&mut self, node: &dyn Node);
}

pub trait EdgeSetter {
    fn set_edge(&mut self, e: Box<dyn Edge>);
}

pub trait EdgeRemover {
    fn remove_edge(&mut self, edge: &dyn Edge);
}

pub trait Builder: NodeAdder + EdgeSetter {}

pub trait UndirectedBuilder: Undirected + Builder {}

pub trait DirectedBuilder: Directed + Builder {}

pub fn copy<B: Builder + Graph>(dst: &mut B, src: &dyn Graph) {
    let nodes = src.nodes();
    for n in &nodes {
        dst.add_node(n.clone());
    }
    for u in &nodes {
        for v in src.from(&**u) {
            if let Some(edge) = src.edge(&**u, &*v) {
                dst.set_edge(edge);
            }
        }
    }
}
