use std::collections::HashMap;

impl Graph {
    fn get_or_create_vertex_locked(
        &mut self,
        vertex_type: VertexType,
        namespace: String,
        name: String,
    ) -> &mut NamedVertex {
        if self.get_vertex_rlocked(&vertex_type, &namespace, &name).is_some() {
            return self.vertices
                .get_mut(&vertex_type)
                .unwrap()
                .get_mut(&namespace)
                .unwrap()
                .get_mut(&name)
                .unwrap();
        }
        self.create_vertex_locked(vertex_type, namespace, name)
    }

    fn get_vertex_rlocked(
        &self,
        vertex_type: &VertexType,
        namespace: &str,
        name: &str,
    ) -> Option<&NamedVertex> {
        self.vertices
            .get(vertex_type)?
            .get(namespace)?
            .get(name)
    }

    fn create_vertex_locked(
        &mut self,
        vertex_type: VertexType,
        namespace: String,
        name: String,
    ) -> &mut NamedVertex {
        let typed_vertices = self.vertices
            .entry(vertex_type.clone())
            .or_insert_with(HashMap::new);

        let namespaced_vertices = typed_vertices
            .entry(namespace.clone())
            .or_insert_with(HashMap::new);

        let node_id = self.graph.new_node_id();
        let vertex = NamedVertex::new(vertex_type.clone(), namespace.clone(), name.clone(), node_id);
        
        self.graph.add_node(Box::new(vertex.clone()));
        namespaced_vertices.insert(name.clone(), vertex);
        
        namespaced_vertices.get_mut(&name).unwrap()
    }

    fn delete_vertex_locked(
        &mut self,
        vertex_type: VertexType,
        namespace: String,
        name: String,
    ) {
        let vertex = match self.get_vertex_rlocked(&vertex_type, &namespace, &name) {
            Some(v) => v.clone(),
            None => return,
        };

        let mut neighbors_to_remove = Vec::new();
        let mut edges_to_remove_from_indexes = Vec::new();

        self.graph.visit_from(&vertex, |neighbor| {
            if self.graph.degree(neighbor) == 1 {
                neighbors_to_remove.push(neighbor.clone());
            }
            true
        });

        self.graph.visit_to(&vertex, |neighbor| {
            if self.graph.degree(neighbor) == 1 {
                neighbors_to_remove.push(neighbor.clone());
            } else {
                if let Some(edge) = self.graph.edge_between(&vertex, neighbor) {
                    edges_to_remove_from_indexes.push(edge.clone());
                }
            }
            true
        });

        self.remove_vertex_locked(&vertex);

        for neighbor in neighbors_to_remove {
            if let Some(named_vertex) = neighbor.downcast_ref::<NamedVertex>() {
                self.remove_vertex_locked(named_vertex);
            }
        }

        for edge in edges_to_remove_from_indexes {
            self.remove_edge_from_destination_index_locked(&edge);
        }
    }
}
