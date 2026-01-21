use std::collections::HashMap;

pub trait EdgeHolder {
    fn visit<F>(&self, visitor: F)
    where
        F: FnMut(i32, &dyn Edge);
    
    fn delete(self: Box<Self>, neighbor: i32) -> Box<dyn EdgeHolder>;
    
    fn set(self: Box<Self>, neighbor: i32, edge: Box<dyn Edge>) -> Box<dyn EdgeHolder>;
    
    fn get(&self, neighbor: i32) -> Option<&dyn Edge>;
    
    fn len(&self) -> usize;
}

pub struct SliceEdgeHolder {
    self_id: i32,
    edges: Vec<Box<dyn Edge>>,
}

impl SliceEdgeHolder {
    pub fn new(self_id: i32) -> Self {
        SliceEdgeHolder {
            self_id,
            edges: Vec::new(),
        }
    }
}

impl EdgeHolder for SliceEdgeHolder {
    fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(i32, &dyn Edge),
    {
        for edge in &self.edges {
            if edge.from().id() == self.self_id {
                visitor(edge.to().id(), edge.as_ref());
            } else {
                visitor(edge.from().id(), edge.as_ref());
            }
        }
    }

    fn delete(mut self: Box<Self>, neighbor: i32) -> Box<dyn EdgeHolder> {
        self.edges.retain(|edge| {
            if edge.from().id() == self.self_id {
                edge.to().id() != neighbor
            } else {
                edge.from().id() != neighbor
            }
        });
        self
    }

    fn set(mut self: Box<Self>, neighbor: i32, new_edge: Box<dyn Edge>) -> Box<dyn EdgeHolder> {
        for i in 0..self.edges.len() {
            let edge = &self.edges[i];
            if edge.from().id() == self.self_id {
                if edge.to().id() == neighbor {
                    self.edges[i] = new_edge;
                    return self;
                }
            } else {
                if edge.from().id() == neighbor {
                    self.edges[i] = new_edge;
                    return self;
                }
            }
        }

        if self.edges.len() < 4 {
            self.edges.push(new_edge);
            return self;
        }

        let mut map = HashMap::with_capacity(self.edges.len() + 1);
        for edge in self.edges {
            if edge.from().id() == self.self_id {
                map.insert(edge.to().id(), edge);
            } else {
                map.insert(edge.from().id(), edge);
            }
        }
        map.insert(neighbor, new_edge);
        Box::new(MapEdgeHolder { edges: map })
    }

    fn get(&self, neighbor: i32) -> Option<&dyn Edge> {
        for edge in &self.edges {
            if edge.from().id() == self.self_id {
                if edge.to().id() == neighbor {
                    return Some(edge.as_ref());
                }
            } else {
                if edge.from().id() == neighbor {
                    return Some(edge.as_ref());
                }
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.edges.len()
    }
}

pub struct MapEdgeHolder {
    edges: HashMap<i32, Box<dyn Edge>>,
}

impl MapEdgeHolder {
    pub fn new() -> Self {
        MapEdgeHolder {
            edges: HashMap::new(),
        }
    }
}

impl EdgeHolder for MapEdgeHolder {
    fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(i32, &dyn Edge),
    {
        for (&neighbor, edge) in &self.edges {
            visitor(neighbor, edge.as_ref());
        }
    }

    fn delete(mut self: Box<Self>, neighbor: i32) -> Box<dyn EdgeHolder> {
        self.edges.remove(&neighbor);
        self
    }

    fn set(mut self: Box<Self>, neighbor: i32, edge: Box<dyn Edge>) -> Box<dyn EdgeHolder> {
        self.edges.insert(neighbor, edge);
        self
    }

    fn get(&self, neighbor: i32) -> Option<&dyn Edge> {
        self.edges.get(&neighbor).map(|e| e.as_ref())
    }

    fn len(&self) -> usize {
        self.edges.len()
    }
}
