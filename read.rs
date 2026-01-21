use std::collections::HashMap;

pub struct DirectedAcyclicGraph {
    undirected_graph: UndirectedGraph,
}

impl DirectedAcyclicGraph {
    pub fn new(self_weight: f64, absent: f64) -> Self {
        DirectedAcyclicGraph {
            undirected_graph: UndirectedGraph::new(self_weight, absent),
        }
    }

    pub fn has_edge_from_to(&self, u: &dyn Node, v: &dyn Node) -> bool {
        if let Some(edge) = self.undirected_graph.edge_between(u, v) {
            edge.from().id() == u.id()
        } else {
            false
        }
    }

    pub fn from(&self, n: &dyn Node) -> Vec<Box<dyn Node>> {
        if !self.undirected_graph.has(n) {
            return Vec::new();
        }

        let fid = n.id();
        let mut nodes = Vec::new();
        
        if let Some(edge_set) = self.undirected_graph.edges.get(&n.id()) {
            edge_set.visit(|neighbor, edge| {
                if edge.from().id() == fid {
                    if let Some(node) = self.undirected_graph.nodes.get(&edge.to().id()) {
                        nodes.push(node.clone_box());
                    }
                }
            });
        }
        
        nodes
    }

    pub fn visit_from<F>(&self, n: &dyn Node, mut visitor: F)
    where
        F: FnMut(&dyn Node) -> bool,
    {
        if !self.undirected_graph.has(n) {
            return;
        }
        
        let fid = n.id();
        
        if let Some(edge_set) = self.undirected_graph.edges.get(&n.id()) {
            edge_set.visit(|neighbor, edge| {
                if edge.from().id() == fid {
                    if let Some(node) = self.undirected_graph.nodes.get(&edge.to().id()) {
                        if !visitor(&**node) {
                            return;
                        }
                    }
                }
            });
        }
    }

    pub fn to(&self, n: &dyn Node) -> Vec<Box<dyn Node>> {
        if !self.undirected_graph.has(n) {
            return Vec::new();
        }

        let tid = n.id();
        let mut nodes = Vec::new();
        
        if let Some(edge_set) = self.undirected_graph.edges.get(&n.id()) {
            edge_set.visit(|neighbor, edge| {
                if edge.to().id() == tid {
                    if let Some(node) = self.undirected_graph.nodes.get(&edge.from().id()) {
                        nodes.push(node.clone_box());
                    }
                }
            });
        }
        
        nodes
    }

    pub fn visit_to<F>(&self, n: &dyn Node, mut visitor: F)
    where
        F: FnMut(&dyn Node) -> bool,
    {
        if !self.undirected_graph.has(n) {
            return;
        }
        
        let tid = n.id();
        
        if let Some(edge_set) = self.undirected_graph.edges.get(&n.id()) {
            edge_set.visit(|neighbor, edge| {
                if edge.to().id() == tid {
                    if let Some(node) = self.undirected_graph.nodes.get(&edge.from().id()) {
                        if !visitor(&**node) {
                            return;
                        }
                    }
                }
            });
        }
    }

    pub fn has(&self, n: &dyn Node) -> bool {
        self.undirected_graph.has(n)
    }
}

impl Directed for DirectedAcyclicGraph {
    fn has_edge_from_to(&self, u: &dyn Node, v: &dyn Node) -> bool {
        self.has_edge_from_to(u, v)
    }

    fn to(&self, node: &dyn Node) -> Vec<Box<dyn Node>> {
        self.to(node)
    }
}

impl Graph for DirectedAcyclicGraph {
    fn has(&self, node: &dyn Node) -> bool {
        self.undirected_graph.has(node)
    }

    fn nodes(&self) -> Vec<Box<dyn Node>> {
        self.undirected_graph.nodes()
    }

    fn from(&self, node: &dyn Node) -> Vec<Box<dyn Node>> {
        self.from(node)
    }

    fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool {
        self.undirected_graph.has_edge_between(x, y)
    }

    fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<Box<dyn Edge>> {
        self.undirected_graph.edge(u, v)
    }
}
