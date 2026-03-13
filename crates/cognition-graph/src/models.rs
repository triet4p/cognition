use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Temporal,     // Liên kết thời gian (xảy ra gần nhau)
    Semantic,     // Liên kết ngữ nghĩa (độ tương đồng vector)
    Entity,       // Liên kết thực thể (cùng nhắc đến 1 người/vật)
    Causal,       // Liên kết nhân quả (Precondition -> Action -> Outcome)
    Hierarchical, // Liên kết phân cấp (Schema: Abstract -> Specific)
}

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