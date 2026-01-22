impl Graph {
    /// 删除从 fromType 到指定目标顶点的所有边
    fn delete_edges_locked(
        &self,
        from_type: VertexType,
        to_type: VertexType,
        to_namespace: String,
        to_name: String,
    ) {
        let mut data = self.lock.write().unwrap();
        
        // 获取目标顶点
        let to_vert = match Self::get_vertex_from_data(
            &data,
            to_type,
            &to_namespace,
            &to_name,
        ) {
            Some(v) => v,
            None => return,
        };
        
        // 收集需要删除的邻居和边
        let mut neighbors_to_remove = Vec::new();
        let mut edges_to_remove = Vec::new();
        
        // 访问所有指向 to_vert 的节点
        let to_id = to_vert.id();
        data.graph.visit_to(to_id, |from_node| {
            // 将 from_node 转换为 NamedVertex
            if let Some(from_vert) = data.graph.node(from_node.id()) {
                if let Some(from_named) = from_vert.as_any().downcast_ref::<NamedVertex>() {
                    // 只处理指定类型的顶点
                    if from_named.vertex_type != from_type {
                        return true;
                    }
                    
                    // 如果这个邻居只有一条边（必定是指向我们的），也删除它
                    if data.graph.degree(from_node.id()) == 1 {
                        neighbors_to_remove.push(Arc::clone(from_vert));
                    } else {
                        // 否则只删除边
                        if let Some(edge) = data.graph.edge_between(from_node.id(), to_id) {
                            edges_to_remove.push(edge);
                        }
                    }
                }
            }
            true
        });
        
        // 清理孤立的顶点
        for v in neighbors_to_remove {
            Self::remove_vertex_from_data(&mut data, &v);
        }
        
        // 删除边并更新目标索引
        for edge in edges_to_remove {
            data.graph.remove_edge(edge.from(), edge.to());
            Self::remove_edge_from_destination_index_locked(&mut data, &edge);
        }
    }
    
    /// 从目标索引中移除边的快速路径
    fn remove_edge_from_destination_index_locked(
        data: &mut GraphData,
        edge: &Arc<dyn Edge>,
    ) {
        let from_id = edge.from();
        
        // 对于边数较少的节点不维护索引
        let edge_count = data.graph.degree(from_id);
        if edge_count < data.destination_edge_threshold {
            data.destination_edge_index.remove(&from_id);
            return;
        }
        
        // 如果索引存在，减少 nodeID->destinationID 的引用计数
        if let Some(index) = data.destination_edge_index.get_mut(&from_id) {
            if let Some(dest_edge) = edge.as_any().downcast_ref::<DestinationEdge>() {
                index.decrement(dest_edge.destination_id());
            }
        }
    }
    
    /// 向目标索引添加边的快速路径
    fn add_edge_to_destination_index_locked(
        data: &mut GraphData,
        edge: &Arc<dyn Edge>,
    ) {
        let from_id = edge.from();
        
        // 检查是否已有索引
        if !data.destination_edge_index.contains_key(&from_id) {
            // 没有索引，使用完整的索引计算方法
            Self::recompute_destination_index_locked(data, from_id);
            return;
        }
        
        // 快速添加新边到现有索引
        if let Some(index) = data.destination_edge_index.get_mut(&from_id) {
            if let Some(dest_edge) = edge.as_any().downcast_ref::<DestinationEdge>() {
                index.increment(dest_edge.destination_id());
            }
        }
    }
    
    /// 必须在写锁下调用
    /// 从图和维护的索引中删除指定的顶点
    /// 不会影响邻居顶点的索引
    fn remove_vertex_locked(&self, v: &Arc<dyn Node>) {
        let mut data = self.lock.write().unwrap();
        Self::remove_vertex_from_data(&mut data, v);
    }
    
    /// 内部辅助方法：从 GraphData 中删除顶点
    fn remove_vertex_from_data(data: &mut GraphData, v: &Arc<dyn Node>) {
        let node_id = v.id();
        
        // 从图中删除节点
        data.graph.remove_node(node_id);
        
        // 从目标边索引中删除
        data.destination_edge_index.remove(&node_id);
        
        // 从 vertices map 中删除
        if let Some(named_vertex) = v.as_any().downcast_ref::<NamedVertex>() {
            if let Some(namespace_map) = data.vertices.get_mut(&named_vertex.vertex_type) {
                if let Some(name_map) = namespace_map.get_mut(&named_vertex.namespace) {
                    name_map.remove(&named_vertex.name);
                    
                    // 如果 namespace 下没有节点了，删除整个 namespace
                    if name_map.is_empty() {
                        namespace_map.remove(&named_vertex.namespace);
                    }
                }
            }
        }
    }
    
    /// 重新计算节点的目标索引
    fn recompute_destination_index_locked(data: &mut GraphData, node_id: i32) {
        // 对于边数较少的节点不维护索引
        let edge_count = data.graph.degree(node_id);
        if edge_count < data.destination_edge_threshold {
            data.destination_edge_index.remove(&node_id);
            return;
        }
        
        // 获取或创建索引
        let index = data.destination_edge_index
            .entry(node_id)
            .or_insert_with(IntSet::new);
        
        // 重置索引
        index.reset();
        
        // 填充索引
        data.graph.visit_from(node_id, |dest_node| {
            if let Some(edge) = data.graph.edge_between(node_id, dest_node.id()) {
                if let Some(dest_edge) = edge.as_any().downcast_ref::<DestinationEdge>() {
                    index.increment(dest_edge.destination_id());
                }
            }
            true
        });
    }
}
