use std::collections::HashMap;

pub trait EdgeHolder {
    fn visit<F>(&self, visitor: F)
    where
        F: FnMut(i64, &dyn Edge);
    
    fn delete(&self, neighbor: i64) -> Box<dyn EdgeHolder>;
    
    fn set(&self, neighbor: i64, edge: Box<dyn Edge>) -> Box<dyn EdgeHolder>;
    
    fn get(&self, neighbor: i64) -> Option<Box<dyn Edge>>;
    
    fn len(&self) -> usize;
    
    fn get_neighbors(&self) -> Vec<i64>;
}

pub struct SliceEdgeHolder {
    self_id: i64,
    edges: Vec<Box<dyn Edge>>,
}

impl SliceEdgeHolder {
    pub fn new(self_id: i64) -> Self {
        SliceEdgeHolder {
            self_id,
            edges: Vec::new(),
        }
    }
}

impl EdgeHolder for SliceEdgeHolder {
    fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(i64, &dyn Edge),
    {
        for edge in &self.edges {
            if edge.from().id() == self.self_id {
                visitor(edge.to().id(), &**edge);
            } else {
                visitor(edge.from().id(), &**edge);
            }
        }
    }

    fn delete(&self, neighbor: i64) -> Box<dyn EdgeHolder> {
        let mut edges = Vec::new();
        for edge in &self.edges {
            let should_keep = if edge.from().id() == self.self_id {
                edge.to().id() != neighbor
            } else {
                edge.from().id() != neighbor
            };
            
            if should_keep {
                edges.push(edge.clone_box());
            }
        }
        
        Box::new(SliceEdgeHolder {
            self_id: self.self_id,
            edges,
        })
    }

    fn set(&self, neighbor: i64, new_edge: Box<dyn Edge>) -> Box<dyn EdgeHolder> {
        let mut edges = self.edges.clone();
        
        for i in 0..edges.len() {
            let edge = &edges[i];
            let matches = if edge.from().id() == self.self_id {
                edge.to().id() == neighbor
            } else {
                edge.from().id() == neighbor
            };
            
            if matches {
                edges[i] = new_edge;
                return Box::new(SliceEdgeHolder {
                    self_id: self.self_id,
                    edges,
                });
            }
        }

        if edges.len() < 4 {
            edges.push(new_edge);
            return Box::new(SliceEdgeHolder {
                self_id: self.self_id,
                edges,
            });
        }

        let mut map = HashMap::with_capacity(edges.len() + 1);
        for edge in &edges {
            let key = if edge.from().id() == self.self_id {
                edge.to().id()
            } else {
                edge.from().id()
            };
            map.insert(key, edge.clone_box());
        }
        map.insert(neighbor, new_edge);
        Box::new(MapEdgeHolder(map))
    }

    fn get(&self, neighbor: i64) -> Option<Box<dyn Edge>> {
        for edge in &self.edges {
            let matches = if edge.from().id() == self.self_id {
                edge.to().id() == neighbor
            } else {
                edge.from().id() == neighbor
            };
            
            if matches {
                return Some(edge.clone_box());
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.edges.len()
    }

    fn get_neighbors(&self) -> Vec<i64> {
        self.edges
            .iter()
            .map(|edge| {
                if edge.from().id() == self.self_id {
                    edge.to().id()
                } else {
                    edge.from().id()
                }
            })
            .collect()
    }
}

pub struct MapEdgeHolder(HashMap<i64, Box<dyn Edge>>);

impl EdgeHolder for MapEdgeHolder {
    fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(i64, &dyn Edge),
    {
        for (neighbor, edge) in &self.0 {
            visitor(*neighbor, &**edge);
        }
    }

    fn delete(&self, neighbor: i64) -> Box<dyn EdgeHolder> {
        let mut map = self.0.clone();
        map.remove(&neighbor);
        Box::new(MapEdgeHolder(map))
    }

    fn set(&self, neighbor: i64, edge: Box<dyn Edge>) -> Box<dyn EdgeHolder> {
        let mut map = self.0.clone();
        map.insert(neighbor, edge);
        Box::new(MapEdgeHolder(map))
    }

    fn get(&self, neighbor: i64) -> Option<Box<dyn Edge>> {
        self.0.get(&neighbor).map(|e| e.clone_box())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_neighbors(&self) -> Vec<i64> {
        self.0.keys().copied().collect()
    }
}

impl Clone for Vec<Box<dyn Edge>> {
    fn clone(&self) -> Self {
        self.iter().map(|e| e.clone_box()).collect()
    }
}

impl Clone for HashMap<i64, Box<dyn Edge>> {
    fn clone(&self) -> Self {
        self.iter()
            .map(|(k, v)| (*k, v.clone_box()))
            .collect()
    }
}
