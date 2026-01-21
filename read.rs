use std::collections::{HashMap, HashSet};
use crate::graph::{Node, Edge};
use crate::sparse::Sparse;
use crate::edge_holder::{EdgeHolder, SliceEdgeHolder};

const MAX_INT: i64 = i64::MAX;

pub struct UndirectedGraph {
    nodes: HashMap<i64, Box<dyn Node>>,
    edges: HashMap<i64, Box<dyn EdgeHolder>>,
    self_weight: f64,
    absent: f64,
    free_ids: Sparse,
    used_ids: Sparse,
}

impl UndirectedGraph {
    pub fn new(self_weight: f64, absent: f64) -> Self {
        UndirectedGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            self_weight,
            absent,
            free_ids: Sparse::new(),
            used_ids: Sparse::new(),
        }
    }

    pub fn new_node_id(&mut self) -> i64 {
        if self.nodes.is_empty() {
            return 0;
        }
        if self.nodes.len() == MAX_INT as usize {
            panic!("simple: cannot allocate node: no slot");
        }

        let mut id: i64 = 0;
        if self.free_ids.len() != 0 && self.free_ids.take_min(&mut id) {
            return id;
        }
        
        id = self.used_ids.max();
        if id < MAX_INT {
            return id + 1;
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
            edge_holder.visit(|neighbor, _edge| {
                if let Some(neighbor_holder) = self.edges.remove(&neighbor) {
                    self.edges.insert(neighbor, neighbor_holder.delete(id));
                }
            });
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

        if !self.has(from.as_ref()) {
            self.add_node(from);
        }
        if !self.has(to.as_ref()) {
            self.add_node(to);
        }

        if let Some(fid_holder) = self.edges.remove(&fid) {
            self.edges.insert(fid, fid_holder.set(tid, e.clone_edge()));
        }
        if let Some(tid_holder) = self.edges.remove(&tid) {
            self.edges.insert(tid, tid_holder.set(fid, e));
        }
    }

    pub fn remove_edge(&mut self, e: &dyn Edge) {
        let from_id = e.from().id();
        let to_id = e.to().id();
        
        if !self.nodes.contains_key(&from_id) {
            return;
        }
        if !self.nodes.contains_key(&to_id) {
            return;
        }

        if let Some(from_holder) = self.edges.remove(&from_id) {
            self.edges.insert(from_id, from_holder.delete(to_id));
        }
        if let Some(to_holder) = self.edges.remove(&to_id) {
            self.edges.insert(to_id, to_holder.delete(from_id));
        }
    }

    pub fn node(&self, id: i64) -> Option<&dyn Node> {
        self.nodes.get(&id).map(|n| n.as_ref())
    }

    pub fn has(&self, n: &dyn Node) -> bool {
        self.nodes.contains_key(&n.id())
    }

    pub fn nodes(&self) -> Vec<&dyn Node> {
        self.nodes.values().map(|n| n.as_ref()).collect()
    }

    pub fn edges(&self) -> Vec<Box<dyn Edge>> {
        let mut edges = Vec::new();
        let mut seen: HashSet<(i64, i64)> = HashSet::new();

        for edge_holder in self.edges.values() {
            edge_holder.visit(|_neighbor, edge| {
                let uid = edge.from().id();
                let vid = edge.to().id();
                
                if seen.contains(&(uid, vid)) {
                    return;
                }
                
                seen.insert((uid, vid));
                seen.insert((vid, uid));
                edges.push(edge);
            });
        }

        edges
    }

    pub fn from(&self, n: &dyn Node) -> Vec<&dyn Node> {
        let id = n.id();
        if !self.has(n) {
            return Vec::new();
        }

        let mut nodes = Vec::new();
        if let Some(edge_holder) = self.edges.get(&id) {
            let len = edge_holder.len();
            nodes.reserve(len);
            
            edge_holder.visit(|neighbor, _edge| {
                if let Some(node) = self.nodes.get(&neighbor) {
                    nodes.push(node.as_ref());
                }
            });
        }

        nodes
    }

    pub fn has_edge_between(&self, x: &dyn Node, y: &dyn Node) -> bool {
        if let Some(edges) = self.edges.get(&x.id()) {
            return edges.get(y.id()).is_some();
        }
        false
    }

    pub fn edge(&self, u: &dyn Node, v: &dyn Node) -> Option<Box<dyn Edge>> {
        self.edge_between(u, v)
    }

    pub fn edge_between(&self, x: &dyn Node, y: &dyn Node) -> Option<Box<dyn Edge>> {
        if !self.has(x) {
            return None;
        }

        self.edges.get(&x.id()).and_then(|holder| holder.get(y.id()))
    }

    pub fn weight(&self, x: &dyn Node, y: &dyn Node) -> (f64, bool) {
        let xid = x.id();
        let yid = y.id();
        
        if xid == yid {
            return (self.self_weight, true);
        }
        
        if let Some(edges) = self.edges.get(&xid) {
            if let Some(e) = edges.get(yid) {
                return (e.weight(), true);
            }
        }
        
        (self.absent, false)
    }

    pub fn degree(&self, n: &dyn Node) -> usize {
        let id = n.id();
        if !self.nodes.contains_key(&id) {
            return 0;
        }

        self.edges.get(&id).map_or(0, |e| e.len())
    }
}
