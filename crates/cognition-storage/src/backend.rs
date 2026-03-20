use async_trait::async_trait;
use cognition_core::{MemoryEngine, MemoryNode, CognitionResult};
use crate::vault::MemVault;

#[async_trait]
impl MemoryEngine for MemVault {
    /// Hiện thực hóa việc lưu trữ Ký ức xuống đĩa
    async fn retain(&self, node: MemoryNode) -> CognitionResult<()> {
        // Gọi hàm store_node đã viết trong vault.rs
        self.store_node(&node).await
    }

    /// Hiện thực hóa việc truy xuất (Hiện tại là fetch từ DB)
    /// Lưu ý: Logic "Recall" thực sự (Spreading Activation) sẽ nằm ở module memory,
    /// module storage chỉ đóng vai trò cung cấp dữ liệu nền (Cold Storage).
    async fn recall(&self, _query: &str, _limit: usize) -> CognitionResult<Vec<MemoryNode>> {
        // Tạm thời trả về toàn bộ node để nạp vào RAM
        self.fetch_all_nodes().await
    }
}