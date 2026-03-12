Chào Triết, đây là bước thiết lập **kết cấu hạ tầng (Infrastructure Setup)** cho `cognition-core`. Chúng ta sẽ làm đúng theo triết lý: dùng công cụ để quản lý, cấu trúc module phân cấp rõ ràng để chuẩn bị cho việc mở rộng sau này.

### Bước 1: Khởi tạo kết cấu thư mục

Tại thư mục gốc của dự án `cognition`, chúng ta sẽ tạo các file trống để định hình cấu trúc module cho `core`.

```powershell
# Chuyển vào thư mục core
cd crates/cognition-core

# Tạo các file module dựa trên thiết kế đã thống nhất
# Lưu ý: Trên Windows PowerShell dùng 'ni' (New-Item), trên Linux/macOS dùng 'touch'
ni src/config.rs, src/logging.rs, src/error.rs, src/types.rs, src/models.rs, src/traits.rs
```

**Cấu trúc dự kiến sau khi tạo:**
```text
cognition-core/
├── Cargo.toml
└── src/
    ├── lib.rs        # Điểm tập trung, export các module dưới đây
    ├── config.rs     # Cấu hình hệ thống (YAML/Env)
    ├── logging.rs    # Thiết lập tracing/observability
    ├── error.rs      # Định nghĩa Enum lỗi dùng chung
    ├── types.rs      # Các kiểu nguyên tử (UUID, Timestamp, Salience)
    ├── models.rs     # Các Schema liên kết (MemoryNode, v.v.)
    └── traits.rs     # Các Interface (LlmProvider, Storage, v.v.)
```

---

### Bước 2: Thêm các thư viện "xương sống" (Dependencies)

Chúng ta sẽ dùng `cargo add` để thêm các thư viện chuẩn công nghiệp. Triết hãy chạy các lệnh này tại thư mục gốc của workspace:

```powershell
# 1. Serialization/Deserialization (Cực kỳ quan trọng)
cargo add -p cognition-core serde --features derive
cargo add -p cognition-core serde_json

# 2. Quản lý thời gian và định danh
cargo add -p cognition-core chrono --features serde
cargo add -p cognition-core uuid --features v4,serde

# 3. Quản lý lỗi chuyên nghiệp (Chương 9 nâng cao)
cargo add -p cognition-core thiserror

# 4. Logging & Chẩn đoán hệ thống (Observability)
cargo add -p cognition-core tracing
cargo add -p cognition-core tracing-subscriber --features env-filter

# 5. Đọc file cấu hình
cargo add -p cognition-core config

# 6. Hỗ trợ Async Trait (Cho đến khi Rust hỗ trợ hoàn toàn dyn async)
cargo add -p cognition-core async-trait
```

---

### Bước 3: Đăng ký Module vào `lib.rs`

Bây giờ, Triết mở file `crates/cognition-core/src/lib.rs` và khai báo các module đã tạo để Compiler nhận diện chúng. Đây là file duy nhất chúng ta "đụng code" lúc này để chốt structure.

```rust
// crates/cognition-core/src/lib.rs

pub mod config;
pub mod error;
pub mod logging;
pub mod models;
pub mod traits;
pub mod types;

// Re-export các kiểu dữ liệu quan trọng để các module khác 
// có thể dùng ngắn gọn: cognition_core::MemoryNode
pub use error::CognitionError;
pub use models::*;
pub use traits::*;
pub use types::*;
```

---

### Giải thích về lựa chọn công cụ:

1.  **`tracing` thay vì `log`**: Trong các hệ thống async (Tokio) và đa luồng, `tracing` mạnh hơn vì nó cho phép theo dõi "context". Ví dụ: Triết có thể biết một dòng log được in ra bởi Agent nào, trong Session nào.
2.  **`thiserror`**: Giúp Triết tạo ra các lỗi có ý nghĩa. Thay vì trả về `String`, Triết sẽ trả về `CognitionError::StorageFull`, giúp module gọi nó biết cách xử lý (ví dụ: thực hiện dọn dẹp bộ nhớ).
3.  **`config`**: Thư viện này rất mạnh, nó cho phép Triết gộp cấu hình từ file `default.toml`, sau đó ghi đè bằng `production.json`, và cuối cùng là ghi đè bằng biến môi trường (Environment Variables).

---

### Kiểm tra trạng thái

Triết hãy chạy lệnh sau tại thư mục gốc:

```powershell
cargo check
```

Nếu kết quả là `Finished`, Triết đã hoàn thành việc xây dựng **"Bộ khung DNA"** cho Cognition. 

**Bước tiếp theo (Research):** Triết muốn chúng ta bắt đầu định nghĩa nội dung cho file nào trước? 
*   `types.rs` & `models.rs`: Định nghĩa hình hài của tri thức (MemoryNode).
*   `traits.rs`: Định nghĩa các bản hợp đồng hành động.
*   `error.rs`: Thiết lập hệ thống chẩn đoán lỗi.

Tôi khuyên nên bắt đầu từ `types.rs` và `models.rs` vì dữ liệu là trọng tâm của CogMem. Triết thấy sao?