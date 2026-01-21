use std::collections::{HashMap, HashSet};

const MAX_INT: i64 = i64::MAX;

pub struct UndirectedGraph {
    nodes: HashMap<i64, Box<dyn Node>>,
    edges: HashMap<i64, Box<dyn EdgeHolder>>,
    self_weight: f64,
    absent: f64,
    free_ids: IntSet,
    used_ids: IntSet,
}

impl UndirectedGraph {
    pub fn new(self_weight: f64, absent: f64) -> Self {
        UndirectedGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            self_weight,
            absent,
            free_ids: IntSet::new(),
            used_ids: IntSet::new(),
        }
    }

    pub fn new_node_id(&mut self) -> i64 {
        if self.nodes.is_empty() {
            return 0;
        }
        if self.nodes.len() == MAX_INT as usize {
            panic!("simple: cannot allocate node: no slot");
        }

        if self.free_ids.len() != 0 {
            if let Some(id) = self.free_ids.take_min() {
                return id;
            }
        }

        let max_id = self.used_ids.max();
        if max_id < MAX_INT {
            return max_id + 1;
        }

        for id in 0..MAX_INT {
            if !self.used_ids.has(id) {
                return id;
            }
        }
        panic!("unreachable")
    }

    pub fn add_node(&mut self, n: Box<dyn Node>) {
        let id = n.id();
        if self.nodes.contains_key(&id) {
            panic!("simple: node ID collision: {}", id);
        }
        self.nodes.insert(id, n);
        self.edges.insert(id, Box::new(SliceEdgeHolder::new(id)));

        self.free_ids.remove(id);
        self.used_ids.insert(id);
    }

    pub fn remove_node(&mut self, n: &dyn Node) {
        let id = n.id();
        if !self.nodes.contains_key(&id) {
            return;
        }
        self.nodes.remove(&id);

        if let Some(edge_holder) = self.edges.get(&id) {
            let neighbors: Vec<i64> = edge_holder.get_neighbors();
            for neighbor in neighbors {
                if let Some(neighbor_holder) = self.edges.get_mut(&neighbor) {
                    *neighbor_holder = neighbor_holder.delete(id);
                }
            }
        }
        self.edges.remove(&id);

        self.free_ids.insert(id);
        self.used_ids.remove(id);
    }

    pub fn set_edge(&mut self, e: Box<dyn Edge>) {
        let from = e.from();
        let fid = from.id();
        let to = e.to();
        let tid = to.id();

        if fid == tid {
            panic!("simple: adding self edge");
        }

        if !self.has(from) {
            self.add_node(from.clone_box());
        }
        if !self.has(to) {
            self.add_node(to.clone_box());
        }

        if let Some(holder) = self.edges.get(&fid) {
            self.edges.insert(fid, holder.set(tid, e.clone_box()));
        }
        if let Some(holder) = self.edges.get(&tid) {
            self.edges.insert(tid, holder.set(fid, e));
        }
    }

    pub fn remove_edge(&mut self, e: &dyn Edge) {
        let from = e.from();
        let to = e.to();
        let fid = from.id();
        let tid = to.id();

        if !self.nodes.contains_key(&fid) {
            return;
        }
        if !self.nodes.contains_key(&tid) {
            return;
        }

        if let Some(holder) = self.edges.get(&fid) {
            self.edges.insert(fid, holder.delete(tid));
        }
        if let Some(holder) = self.edges.get(&tid) {
            self.edges.insert(tid, holder.delete(fid));
        }
    }

    pub fn node(&self, id: i64) -> Option<&dyn Node> {
        self.nodes.get(&id).map(|n| &**n)
    }

    pub fn has(&self, n: &dyn Node) -> bool {
        self.nodes.contains_key(&n.id())
    }

    pub fn nodes(&self) -> Vec<Box<dyn Node>> {
        self.nodes.values().map(|n| n.clone_box()).collect()
    }

    pub fn edges(&self) -> Vec<Box<dyn Edge>> {
        let mut edges = Vec::new();
        let mut seen: HashSet<(i64, i64)> = HashSet::new();

        for holder in self.edges.values() {
            holder.visit(|neighbor, e| {
                let uid = e.from().id();
                let vid = e.to().id();
                if seen.contains(&(uid, vid)) {
                    return;
                }
                seen.insert((uid, vid));
                seen.insert((vid, uid));
                edges.push(e.clone_box());
            });
        }

        edges
    }

    pub fn from(&self, n: &dyn Node) -> Vec<Box<dyn Node>> {
        if !self.has(n) {
            return Vec::new();
        }

        let mut nodes = Vec::new();
        if let Some(holder) = self.edges.get(&n.id()) {
            holder.visit(|neighbor, _edge| {
                if let Some(node) = self.nodes.get(&neighbor) {
                    nodes.push(node.clone_box());
                }
            });
        }

        nodes
    }

    pub fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool {
        if let Some(holder) = self.edges.get(&x.id()) {
            holder.get(y.id()).is_some()
        } else {
            false
        }
    }

    pub fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<Box<dyn Edge>> {
        self.edge_between(u, v)
    }

    pub fn edge_between(&self, x: &dyn Node, y: &dyn Node) -> Option<Box<dyn Edge>> {
        if !self.has(x) {
            return None;
        }

        if let Some(holder) = self.edges.get(&x.id()) {
            holder.get(y.id())
        } else {
            None
        }
    }

    pub fn weight(&self, x: &dyn Node, y: &dyn Node) -> (f64, bool) {
        let xid = x.id();
        let yid = y.id();

        if xid == yid {
            return (self.self_weight, true);
        }

        if let Some(holder) = self.edges.get(&xid) {
            if let Some(e) = holder.get(yid) {
                return (e.weight(), true);
            }
        }

        (self.absent, false)
    }

    pub fn degree(&self, n: &dyn Node) -> usize {
        if !self.nodes.contains_key(&n.id()) {
            return 0;
        }

        self.edges.get(&n.id()).map_or(0, |holder| holder.len())
    }
}

impl Graph for UndirectedGraph {
    fn has(&self, node: &dyn Node) -> bool {
        self.has(node)
    }

    fn nodes(&self) -> Vec<Box<dyn Node>> {
        self.nodes()
    }

    fn from(&self, node: &dyn Node) -> Vec<Box<dyn Node>> {
        self.from(node)
    }

    fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool {
        self.has_edge_between(x, y)
    }

    fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<Box<dyn Edge>> {
        self.edge(u, v)
    }
}
