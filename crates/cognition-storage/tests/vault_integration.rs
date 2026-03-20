use cognition_core::{MemoryNode, NetworkType, MemoryEngine};
use cognition_storage::MemVault;
use std::fs;

#[tokio::test]
async fn test_vault_persistence_lifecycle() {
    let db_name = "test_cognition.db";
    let db_path = format!("sqlite:{}", db_name);

    // 1. Khởi tạo Vault (Tự động chạy migration)
    let vault = MemVault::new(&db_path).await.expect("Failed to create vault");

    // 2. Tạo một Ký ức mẫu (Intention Network)
    let fact = "I plan to master Rust by the end of Q2".to_string();
    let node = MemoryNode::new(NetworkType::Intention, fact.clone());
    let node_id = node.id;

    // 3. Lưu vào DB thông qua Trait MemoryEngine
    vault.retain(node).await.expect("Failed to retain node");

    // 4. Thử lấy lại Node bằng ID
    let fetched = vault.fetch_node_by_id(node_id).await.expect("Failed to fetch");
    
    assert!(fetched.is_some());
    let fetched_node = fetched.unwrap();
    assert_eq!(fetched_node.narrative_fact, fact);
    assert_eq!(fetched_node.network_type, NetworkType::Intention);

    // 5. Dọn dẹp file test sau khi xong
    // Lưu ý: Trong thực tế nên dùng tempfile, ở đây ta xóa thủ công cho trực quan
    let _ = fs::remove_file(db_name);
    let _ = fs::remove_file(format!("{}-shm", db_name));
    let _ = fs::remove_file(format!("{}-wal", db_name));
}