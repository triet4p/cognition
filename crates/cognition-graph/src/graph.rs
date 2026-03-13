use std::collections::HashMap;
use tokio::sync::RwLock;
use petgraph::graph::{DiGraph, NodeIndex};
use cognition_core::{CognitionResult, CognitionError, MemoryNode, NodeId};
use crate::models::EdgeData;

/// # CognitiveGraph
/// 
/// Hệ thống đồ thị hợp nhất (Unified Knowledge Graph) quản lý 6 mạng nhận thức.
/// Sử dụng `tokio::sync::RwLock` để cho phép nhiều Agent cùng đọc (Recall) 
/// nhưng chỉ 1 Agent được ghi (Retain/Consolidate) tại một thời điểm.
pub struct CognitiveGraph {
    /// Đồ thị có hướng (Directed Graph) từ petgraph.
    /// Chứa MemoryNode ở các đỉnh và EdgeData ở các cạnh.
    inner: RwLock<DiGraph<MemoryNode, EdgeData>>,
    
    /// Bảng tra cứu nhanh (Lookup Table):
    /// petgraph sử dụng `NodeIndex` (index mảng) để quản lý đỉnh cho nhanh.
    /// Nhưng hệ thống của ta dùng `NodeId` (UUID). 
    /// Ta cần HashMap này để map UUID sang NodeIndex với chi phí O(1).
    node_indices: RwLock<HashMap<NodeId, NodeIndex>>,
}

impl CognitiveGraph {
    /// Tạo một CognitiveGraph mới, ban đầu rỗng.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(DiGraph::new()),
            node_indices: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a new MemoryNode to the graph.
    /// Returns an error if the node already exists.
    pub async fn add_node(&self, node: MemoryNode) -> CognitionResult<()> {
        let id = node.id;

        // 1. Acquire write lock on indices
        let mut indices = self.node_indices.write().await;
        if indices.contains_key(&id) {
            return Err(CognitionError::Logic(format!("Node with id {} already exists", id)));
        }

        // 2. Acquire write lock on graph
        let mut graph = self.inner.write().await;
        let idx = graph.add_node(node);
        indices.insert(id, idx); 

        // Locks are automatically dropped here when variables go out of scope
        Ok(())
    }

    /// Adds a directed edge between two existing nodes.
    pub async fn add_edge(&self, source: NodeId, target: NodeId, edge: EdgeData) -> CognitionResult<()> {
        // IMPORTANT DEADLOCK PREVENTION:
        // We acquire the read lock, extract the data we need (copying the NodeIndex),
        // and then EXPLICITLY drop the read lock before acquiring the write lock.
        let (src_idx, tgt_idx) = {
            let indices = self.node_indices.read().await;

            let s = indices.get(&source)
                .ok_or_else(|| CognitionError::Logic(format!("Source node {} not found", source)))?;
            let t = indices.get(&target)
                .ok_or_else(|| CognitionError::Logic(format!("Target node {} not found", target)))?;
            (*s, *t) // Copy the NodeIndex values out
        }; // Read lock on `node_indices` is dropped here!

        // Now it is perfectly safe to acquire the write lock on the graph
        let mut graph = self.inner.write().await;
        graph.add_edge(src_idx, tgt_idx, edge);

        Ok(())
    }

    /// Retrieves a clone of a MemoryNode by its UUID.
    pub async fn get_node(&self, id: NodeId) -> CognitionResult<MemoryNode> {
        let idx = {
            let indices = self.node_indices.read().await;
            *indices.get(&id)
                .ok_or_else(|| CognitionError::Logic(format!("Node {} not found", id)))?
        };

        let graph = self.inner.read().await;
        let node = graph.node_weight(idx)
            .ok_or_else(|| CognitionError::Memory(format!("Graph corrupted: NodeIndex missing for {}", id)))?;
        
        Ok(node.clone())
    }

    /// Returns the total number of nodes in the graph
    pub async fn node_count(&self) -> usize {
        let graph = self.inner.read().await;
        graph.node_count()
    }
}

// Implement Default cho chuẩn Rust
impl Default for CognitiveGraph {
    fn default() -> Self {
        Self::new()
    }
}