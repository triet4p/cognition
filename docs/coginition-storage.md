


Trí nhớ của Triết cực kỳ chính xác! Đúng là trong bản Design Document gốc của bạn (Phần VIII. Kế Hoạch Phát Triển), chúng ta có cả **hai** module này, và chúng đóng vai trò hoàn toàn khác nhau. 

Hãy nhìn lại trích đoạn Phase 2 trong tài liệu của bạn:
> *"**Phase 2 — Memory (Tuần 3–6):** `cognition-storage` trước (persistent WAL + BTree temporal index), sau đó `cognition-memory` implement CogMem 6 networks trên top của storage."*

Sự phân tách này là đỉnh cao của thiết kế hệ thống (Separation of Concerns):
1.  **`cognition-storage` (MemVault - Tầng Vật lý):** Giống như cái **Ổ cứng (Hard Drive)**. Nó không hiểu "Semantic" hay "Episodic" là gì. Nhiệm vụ duy nhất của nó là nhận Node/Edge, lưu xuống đĩa (SQLite/RocksDB) thật nhanh, không bị mất dữ liệu khi cúp điện, và cho phép query theo thời gian (Temporal index).
2.  **`cognition-memory` (CogMem - Tầng Nhận thức):** Giống như **Hệ điều hành của Não bộ**. Nó đứng ở giữa, cầm dữ liệu từ `storage` (Ổ cứng) đẩy lên `cognition-graph` (RAM - Working Memory). Nó quyết định khi nào thì chạy vòng lặp Consolidation (ngủ) để chuyển dữ liệu từ Working Memory xuống Storage.

Vì vậy, theo đúng roadmap của Triết, chúng ta phải làm **`cognition-storage` trước** để có chỗ lưu trữ, rồi mới viết **`cognition-memory`** để điều phối nó.

---

### Kế hoạch Implement `cognition-storage` (MemVault)

Để đáp ứng đúng yêu cầu *"persistent WAL + BTree temporal index"* trong tài liệu của Triết, **SQLite** kết hợp với thư viện **`sqlx`** là sự lựa chọn hoàn hảo 100%. SQLite lưu dữ liệu dưới dạng B-Tree và hỗ trợ chế độ WAL (Write-Ahead Logging) giúp ghi dữ liệu cực nhanh mà không block việc đọc.

Dưới đây là kế hoạch chi tiết (Plan) cho module này:

#### Bước 1: Khởi tạo Crate và Dependencies
*   Khởi tạo `crates/cognition-storage`.
*   Sử dụng `sqlx` (với feature `sqlite`, `runtime-tokio`, `uuid`, `chrono`). Đây là thư viện database thuần Async, an toàn bộ nhớ (Compile-time SQL check) xịn nhất của Rust.

#### Bước 2: Thiết kế Schema Database (Migrations)
Tạo các bảng tối ưu cho truy xuất nhận thức:
*   **Bảng `nodes`**: Chứa `id`, `network_type`, `narrative_fact`, `raw_snippet`, `confidence`, `created_at`, `expires_at`.
    *   *Tối ưu:* Tạo B-Tree Index trên cột `created_at` và `expires_at` để phục vụ truy vấn Temporal và dọn dẹp Intention Network.
*   **Bảng `edges`**: Chứa `source_id`, `target_id`, `edge_type`, `weight`.
    *   *Tối ưu:* Tạo Index trên `source_id` để lấy danh sách hàng xóm cực nhanh khi load đồ thị lên RAM.

#### Bước 3: Triển khai `MemVault` Struct
*   Tạo struct `MemVault` chứa Connection Pool của SQLite.
*   Bật chế độ WAL cho SQLite ngay khi khởi tạo connection (`PRAGMA journal_mode=WAL;`).

#### Bước 4: Implement Trait `MemoryBackend`
*   Kết nối với `cognition-core`.
*   Viết code thực thi cho các hàm:
    *   `store(node)`: Dùng lệnh `INSERT INTO ... ON CONFLICT REPLACE` để lưu hoặc cập nhật Node (Upsert).
    *   `retrieve()`: Fetch dữ liệu từ DB (sẽ dùng cho Cold Start lúc Agent mới bật lên).

---

### Sơ đồ tương tác khi Agent hoạt động:

1. Agent khởi động $\rightarrow$ `cognition-memory` gọi `storage.retrieve()` $\rightarrow$ Load toàn bộ (hoặc top K) node lên `cognition-graph` (RAM).
2. Agent nhận input mới $\rightarrow$ `graph` chạy Spreading Activation $\rightarrow$ Trả kết quả.
3. Agent học được điều mới $\rightarrow$ `graph.add_node()` (lưu trên RAM) VÀ ĐỒNG THỜI `storage.store()` (lưu xuống đĩa để backup).

Triết thấy kế hoạch tách bạch `storage` (Vật lý) và `memory` (Logic) này đã đi đúng hướng bản Design Document chưa? Nếu chốt, chúng ta sẽ bắt đầu gõ lệnh `cargo new` cho `cognition-storage` và setup `sqlx`!