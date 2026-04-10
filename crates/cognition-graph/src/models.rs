use serde::{Deserialize, Serialize};
pub use cognition_core::EdgeType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub edge_type: EdgeType,
    pub weight: f32, // Độ mạnh của liên kết (0.0 - 1.0)
    pub multiplier: f32, // Hệ số điều chỉnh khi lan truyền thông tin qua cạnh này
}

impl EdgeData {
    pub fn new(edge_type: EdgeType, weight: f32, multiplier: f32) -> Self {
        Self {
            edge_type,
            weight,
            multiplier,
        }
    }
}