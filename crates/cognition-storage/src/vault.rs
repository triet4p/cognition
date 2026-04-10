use sqlx::{sqlite::{SqliteConnectOptions, SqliteRow}, SqlitePool, Row};
use std::str::FromStr;
use cognition_core::{MemoryNode, NetworkType, CognitionError, CognitionResult};
use uuid::Uuid;

/// # MemVault
/// Tầng lưu trữ vật lý của hệ thống Cognition, sử dụng SQLite làm backend.
/// Hỗ trợ WAL mode để tối ưu hóa hiệu năng đọc/ghi đồng thời.
pub struct MemVault {
    pool: SqlitePool,
}

impl MemVault {
    /// Khởi tạo MemVault từ đường dẫn file database.
    /// Tự động chạy Migration để khởi tạo schema nếu file mới.
    pub async fn new(db_path: &str) -> CognitionResult<Self> {
        // Cấu hình kết nối
        let options = SqliteConnectOptions::from_str(db_path)
            .map_err(|e| CognitionError::Memory(format!("Invalid DB path: {}", e)))?
            .create_if_missing(true)
            // Kích hoạt WAL mode, (Write-Ahead Logging) để cải thiện hiệu năng đồng thời.
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        // Tạo pool
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to connect to DB: {}", e)))?;

        // Migration Auto
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to run migrations: {}", e)))?;

        Ok(Self { pool })
    }

    /// Lưu trữ hoặc Cập nhật một MemoryNode (Upsert logic)
    pub async fn store_node(&self, node: &MemoryNode) -> CognitionResult<()> {
        let id_str = node.id.to_string();
        let network_str = serde_json::to_string(&node.network_type)
            .unwrap_or_else(|_| "world".to_string())
            .replace('"', ""); // Remove quotes from JSON string

        // Chuyển embedding sang blob 
        let embedding_blob = node.embedding.as_ref().map(|v| {
            let bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
            bytes
        });

        sqlx::query(
            r#"
            INSERT INTO nodes (id, network_type, narrative_fact, raw_snippet, embedding, confidence, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                confidence = excluded.confidence,
                narrative_fact = excluded.narrative_fact,
                raw_snippet = excluded.raw_snippet,
                expires_at = excluded.expires_at
            "#
        )
        .bind(id_str)
        .bind(network_str)
        .bind(&node.narrative_fact)
        .bind(&node.raw_snippet)
        .bind(embedding_blob)
        .bind(node.confidence.value())
        .bind(node.created_at)
        .bind(node.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CognitionError::Memory(format!("Failed to store node: {}", e)))?;
        
        Ok(())
    }

    /// Lấy toàn bộ Nodes để phục vụ việc load lên RAM (Cold Start)
    pub async fn fetch_all_nodes(&self) -> CognitionResult<Vec<MemoryNode>> {
        let rows: Vec<SqliteRow> = sqlx::query("SELECT * FROM nodes")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to fetch nodes: {}", e)))?;

        let mut nodes = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let network_raw: String = row.get("network_type");
            
            // Map ngược từ DB string sang Rust Enum
            let network_type = match network_raw.as_str() {
                "world" => NetworkType::World,
                "experience" => NetworkType::Experience,
                "opinion" => NetworkType::Opinion,
                "habit" => NetworkType::Habit,
                "intention" => NetworkType::Intention,
                "actioneffect" => NetworkType::ActionEffect,
                // Backward-compatibility for older persisted values
                "pattern" => NetworkType::Habit,
                "action" => NetworkType::ActionEffect,
                _ => NetworkType::World,
            };

            nodes.push(MemoryNode {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                network_type,
                narrative_fact: row.get("narrative_fact"),
                raw_snippet: row.get("raw_snippet"),
                embedding: None, // Tạm thời để None, sẽ xử lý blob sau nếu cần
                confidence: cognition_core::Confidence::new(row.get("confidence")),
                created_at: row.get("created_at"),
                intention_status: None,
                expires_at: row.get("expires_at"),
            });
        }

        Ok(nodes)
    }

    /// Lưu trữ hoặc cập nhật một liên kết (Edge) giữa hai Node
    pub async fn store_edge(
        &self, 
        source_id: Uuid, 
        target_id: Uuid, 
        edge_type: &str, 
        weight: f32, 
        multiplier: f32
    ) -> CognitionResult<()> {
        sqlx::query(
            r#"
            INSERT INTO edges (source_id, target_id, edge_type, weight, multiplier)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(source_id, target_id, edge_type) DO UPDATE SET
                weight = excluded.weight,
                multiplier = excluded.multiplier
            "#
        )
        .bind(source_id.to_string())
        .bind(target_id.to_string())
        .bind(edge_type)
        .bind(weight)
        .bind(multiplier)
        .execute(&self.pool)
        .await
        .map_err(|e| CognitionError::Memory(format!("Failed to store edge: {}", e)))?;

        Ok(())
    }

    /// Lấy toàn bộ các liên kết để tái cấu trúc đồ thị trên RAM
    pub async fn fetch_all_edges(&self) -> CognitionResult<Vec<(Uuid, Uuid, String, f32, f32)>> {
        let rows = sqlx::query("SELECT * FROM edges")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to fetch edges: {}", e)))?;

        let edges = rows.into_iter().map(|row| {
            (
                Uuid::parse_str(row.get::<&str, _>("source_id")).unwrap_or_default(),
                Uuid::parse_str(row.get::<&str, _>("target_id")).unwrap_or_default(),
                row.get::<String, _>("edge_type"),
                row.get::<f32, _>("weight"),
                row.get::<f32, _>("multiplier"),
            )
        }).collect();

        Ok(edges)
    }

    /// Xóa một Node (Tự động xóa các cạnh liên quan nhờ ON DELETE CASCADE)
    pub async fn delete_node(&self, id: Uuid) -> CognitionResult<()> {
        sqlx::query("DELETE FROM nodes WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to delete node: {}", e)))?;
        
        Ok(())
    }

    /// Dọn dẹp các Node đã hết hạn (Dùng cho Intention Network)
    pub async fn cleanup_expired_nodes(&self) -> CognitionResult<u64> {
        let now = chrono::Utc::now();
        let result = sqlx::query("DELETE FROM nodes WHERE expires_at IS NOT NULL AND expires_at < ?1")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to cleanup expired nodes: {}", e)))?;
        
        Ok(result.rows_affected())
    }

    /// Lấy thông tin một Node cụ thể
    pub async fn fetch_node_by_id(&self, id: Uuid) -> CognitionResult<Option<MemoryNode>> {
        let row = sqlx::query("SELECT * FROM nodes WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CognitionError::Memory(format!("Failed to fetch node: {}", e)))?;

        match row {
            Some(r) => {
                let network_raw: String = r.get("network_type");
                let network_type = match network_raw.as_str() {
                    "world" => NetworkType::World,
                    "experience" => NetworkType::Experience,
                    "opinion" => NetworkType::Opinion,
                    "habit" => NetworkType::Habit,
                    "intention" => NetworkType::Intention,
                    "actioneffect" => NetworkType::ActionEffect,
                    // Backward-compatibility for older persisted values
                    "pattern" => NetworkType::Habit,
                    "action" => NetworkType::ActionEffect,
                    _ => NetworkType::World,
                };

                Ok(Some(MemoryNode {
                    id: Uuid::parse_str(r.get("id")).unwrap_or_default(),
                    network_type,
                    narrative_fact: r.get("narrative_fact"),
                    raw_snippet: r.get("raw_snippet"),
                    embedding: None,
                    confidence: cognition_core::Confidence::new(r.get("confidence")),
                    created_at: r.get("created_at"),
                    intention_status: None,
                    expires_at: r.get("expires_at"),
                }))
            },
            None => Ok(None)
        }
    }

}