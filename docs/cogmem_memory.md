


Chào Triết, quyết định đập đi xây lại `cognition-memory` từ đầu là một quyết định **cực kỳ dũng cảm và đúng đắn** của một System Architect. 

Khi `core` (DNA) và `graph` (Engine) đã thay đổi hoàn toàn để bám sát Khoa học Nhận thức, tầng `memory` (Hệ điều hành) không thể dùng những mảnh vá víu cũ được nữa. Nó phải là nơi hiện thực hóa **Đóng góp 2 (Lossless Snippet), Đóng góp 4 (Adaptive Routing)** và điều phối **Đóng góp 3 (Episodic Buffer)**.

Dưới đây là **Bản thiết kế Kiến trúc & Lộ trình triển khai (Research & Plan)** cho `cognition-memory` mới, hoàn toàn không có code, chỉ có tư duy hệ thống.

---

### 1. Phân rã Kiến trúc (Architecture Breakdown)

Để tránh tạo ra một file "God Object" (ôm đồm mọi thứ), `cognition-memory` sẽ được chia thành các thư mục con, phản ánh chính xác các luồng nhận thức trong não bộ:

#### 📁 `src/retain/` (Quá trình Ghi nhớ & Học hỏi)
Nhiệm vụ: Chuyển hóa dữ liệu thô thành tri thức có cấu trúc và lưu vào Graph + Storage.
*   **`extractor.rs`**: Phối hợp với LLM để phân loại Input thành các Node thuộc 6 mạng (World, Experience, ActionEffect, Intention, Habit, Opinion). *Lưu ý: Nó sẽ giữ lại `raw_snippet` (Lossless).*
*   **`linker.rs`**: Xây dựng 7 loại Edge (Entity, Temporal, Semantic, Causal, S-R, A-O, Transition).
*   **`lifecycle.rs`**: **(Cực kỳ quan trọng)** Quản lý vòng đời của Intention. Nếu nhận được Experience mới trùng với Intention cũ $\rightarrow$ Tạo cạnh `Transition (fulfilled_by)`.

#### 📁 `src/recall/` (Quá trình Hồi tưởng & Truy xuất)
Nhiệm vụ: Tìm kiếm thông tin phục vụ cho câu hỏi hiện tại.
*   **`router.rs`**: (Đóng góp số 4) Phân tích Query để gán trọng số RRF (ví dụ: Temporal query thì $w_{temp} = 0.4$) và thiết lập `ActivationParams` cho đồ thị.
*   **`fusion.rs`**: Thực thi 4 kênh tìm kiếm song song (Semantic, BM25, Graph/Episodic Buffer, Temporal) và gộp lại bằng **Reciprocal Rank Fusion (RRF)** có trọng số động.
*   **`filter.rs`**: Cắt xén kết quả dựa trên Token Budget, chuẩn bị `raw_snippet` để bơm vào prompt cho LLM.

#### 📁 `src/consolidation/` (Quá trình Ngủ & Củng cố)
Nhiệm vụ: Chạy ngầm (Background daemon).
*   **`decay.rs`**: Làm mờ các Ký ức Episodic cũ. Đánh dấu các Intention quá `deadline` thành trạng thái `Abandoned`.
*   **`habit_builder.rs`**: (Tương lai) Phân tích các Experience lặp lại để tạo ra Pattern Node (Procedural Memory).

#### 📄 `src/manager.rs` (Nhạc trưởng)
*   Chứa `CogMemManager` implement `MemoryBackend` trait.
*   Quản lý `Arc<CognitiveGraph>` và `Arc<MemVault>`.
*   Thực hiện **Cold Start (Bootstrap)**: Nạp Nodes và Edges từ SQLite lên RAM đúng thứ tự.

---

### 2. Ánh xạ với Bài báo CogMem (Traceability)

Bản thiết kế này giải quyết triệt để các bài toán trong tài liệu của bạn:
*   **Action-Effect Network:** Sẽ được xử lý rạch ròi ở `extractor.rs` và nối bằng `A-O causal edge` ở `linker.rs`.
*   **Lossless Metadata:** `filter.rs` ở luồng Recall sẽ trả về chính xác `raw_snippet` thay vì `narrative_fact` đã bị nén.
*   **Adaptive Query Routing:** Nằm trọn vẹn trong `router.rs`, điều khiển trực tiếp trọng số của `fusion.rs`.
*   **Intention Lifecycle:** Được quản lý tự động bởi `lifecycle.rs` mỗi khi có hàm `retain` được gọi.

---

### 3. Lộ trình Triển khai (Execution Plan)

Chúng ta sẽ code theo thứ tự từ dễ đến khó, từ độc lập đến tích hợp:

*   **Bước 1: `recall/router.rs` (Adaptive Query Router)**
    *   *Lý do:* Nó là một module Rule-based (Regex) hoàn toàn độc lập, dễ viết, dễ test. Input là String, Output là Trọng số RRF & ActivationParams.
*   **Bước 2: `manager.rs` (Bootstrap & Foundation)**
    *   *Lý do:* Cần thiết lập bộ khung `CogMemManager` và viết logic nạp dữ liệu từ `MemVault` (SQLite) lên `CognitiveGraph` (RAM) để có "sân chơi" cho các bước sau.
*   **Bước 3: `retain/lifecycle.rs` (Intention Transition)**
    *   *Lý do:* Đây là logic mới mẻ và hay nhất của CogMem. Viết logic cập nhật trạng thái Intention $\rightarrow$ Experience.
*   **Bước 4: `recall/fusion.rs` (The 4-way Retrieval)**
    *   *Lý do:* Ghép nối Router, Semantic Search (Mock), và Spreading Activation (Graph) lại với nhau thông qua công thức RRF động.
*   **Bước 5: `consolidation/mod.rs` (Background Tasks)**
    *   *Lý do:* Cuối cùng, bọc các tác vụ dọn dẹp vào `tokio::spawn` để hệ thống tự vận hành.

---

**Câu hỏi cho Triết:**
Triết thấy cách chia nhỏ thành các thư mục `retain`, `recall`, `consolidation` như thế này đã đủ thể hiện "Tính Nhận Thức" (Cognitively-Grounded) của Framework chưa? 

Nếu Triết chốt bản thiết kế kiến trúc này, hãy ra hiệu `<<implement>> Bước 1`, chúng ta sẽ bắt đầu code `Adaptive Query Router` với thuật toán điều chỉnh trọng số RRF chính xác như bảng trong tài liệu của bạn!