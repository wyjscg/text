struct NamedVertex {
    name: String,
    namespace: String,
    id: i64,
    vertex_type: VertexType,
}

impl NamedVertex {
    fn new(vertex_type: VertexType, namespace: String, name: String, id: i64) -> Self {
        NamedVertex {
            vertex_type,
            name,
            namespace,
            id,
        }
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn to_string(&self) -> String {
        if self.namespace.is_empty() {
            format!("{}:{}", vertex_types_str(self.vertex_type), self.name)
        } else {
            format!(
                "{}:{}/{}",
                vertex_types_str(self.vertex_type),
                self.namespace,
                self.name
            )
        }
    }
}

struct DestinationEdge {
    f: Box<dyn Node>,
    t: Box<dyn Node>,
    destination: Box<dyn Node>,
}

impl DestinationEdge {
    fn new(from: Box<dyn Node>, to: Box<dyn Node>, destination: Box<dyn Node>) -> Self {
        DestinationEdge {
            f: from,
            t: to,
            destination,
        }
    }

    fn from(&self) -> &dyn Node {
        &*self.f
    }

    fn to(&self) -> &dyn Node {
        &*self.t
    }

    fn weight(&self) -> f64 {
        0.0
    }

    fn destination_id(&self) -> i64 {
        self.destination.id()
    }
}
