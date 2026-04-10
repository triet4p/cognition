# CogMem: Khung Bộ Nhớ Hội Thoại Dài Hạn Có Nền Tảng Khoa Học Nhận Thức

---

## Mục Lục

- [CogMem: Khung Bộ Nhớ Hội Thoại Dài Hạn Có Nền Tảng Khoa Học Nhận Thức](#cogmem-khung-bộ-nhớ-hội-thoại-dài-hạn-có-nền-tảng-khoa-học-nhận-thức)
  - [Mục Lục](#mục-lục)
  - [1. Tổng Quan CogMem](#1-tổng-quan-cogmem)
  - [2. Bối Cảnh: Tại Sao Cần CogMem?](#2-bối-cảnh-tại-sao-cần-cogmem)
    - [2.1 Bài Toán Bộ Nhớ Hội Thoại Dài Hạn](#21-bài-toán-bộ-nhớ-hội-thoại-dài-hạn)
    - [2.2 Benchmark Đánh Giá](#22-benchmark-đánh-giá)
    - [2.3 HINDSIGHT — Baseline Mạnh Nhất Hiện Tại và Điểm Yếu Của Nó](#23-hindsight--baseline-mạnh-nhất-hiện-tại-và-điểm-yếu-của-nó)
  - [3. Kiến Trúc CogMem](#3-kiến-trúc-cogmem)
    - [3.1 Đóng Góp 1 — Cognitively-Grounded Memory Graph (6 Networks + 7 Edge Types)](#31-đóng-góp-1--cognitively-grounded-memory-graph-6-networks--7-edge-types)
      - [Nền tảng: Tại Sao Phân Chia Thành Nhiều Mạng?](#nền-tảng-tại-sao-phân-chia-thành-nhiều-mạng)
      - [6 Mạng Bộ Nhớ](#6-mạng-bộ-nhớ)
      - [Phân Biệt Habit vs. Action-Effect: Phân Ly S-R / A-O](#phân-biệt-habit-vs-action-effect-phân-ly-s-r--a-o)
      - [7 Edge Types và Intention Lifecycle](#7-edge-types-và-intention-lifecycle)
      - [Phân Biệt Action-Effect Network vs. Mem^p](#phân-biệt-action-effect-network-vs-memp)
    - [3.2 Đóng Góp 2 — Lossless Node Metadata (Raw State Snippets)](#32-đóng-góp-2--lossless-node-metadata-raw-state-snippets)
      - [Vấn Đề: Mọi Hệ Thống Hiện Tại Đều Nén Có Mất Mát](#vấn-đề-mọi-hệ-thống-hiện-tại-đều-nén-có-mất-mát)
      - [Giải Pháp: 2-Layer Node Schema](#giải-pháp-2-layer-node-schema)
      - [Cơ Sở Nhận Thức](#cơ-sở-nhận-thức)
    - [3.3 Đóng Góp 3 — Episodic Buffer với SUM Spreading Activation + Cycle Guards](#33-đóng-góp-3--episodic-buffer-với-sum-spreading-activation--cycle-guards)
      - [Vấn Đề: MAX Chỉ Lấy Đường Mạnh Nhất](#vấn-đề-max-chỉ-lấy-đường-mạnh-nhất)
      - [Cơ Sở Nhận Thức: Episodic Buffer](#cơ-sở-nhận-thức-episodic-buffer)
      - [Giải Pháp: Phép SUM Thay Phép MAX](#giải-pháp-phép-sum-thay-phép-max)
      - [Ba Cycle Guards](#ba-cycle-guards)
    - [3.4 Đóng Góp 4 — Adaptive Query Routing](#34-đóng-góp-4--adaptive-query-routing)
      - [Vấn Đề: Equal-Weight RRF Không Phân Biệt Ý Định Truy Vấn](#vấn-đề-equal-weight-rrf-không-phân-biệt-ý-định-truy-vấn)
      - [Giải Pháp: Dynamic Channel Weights theo Query Type](#giải-pháp-dynamic-channel-weights-theo-query-type)
      - [Cơ Sở Nhận Thức](#cơ-sở-nhận-thức-1)
    - [3.5 Đóng Góp 5 — Hierarchical Knowledge Graph (Thăm Dò)](#35-đóng-góp-5--hierarchical-knowledge-graph-thăm-dò)
  - [4. Thiết Kế Thực Nghiệm](#4-thiết-kế-thực-nghiệm)
    - [4.1 Evaluation Strategy](#41-evaluation-strategy)
    - [4.2 Ablation Study](#42-ablation-study)
    - [4.3 Baselines So Sánh](#43-baselines-so-sánh)
  - [5. Các Nghiên Cứu Liên Quan](#5-các-nghiên-cứu-liên-quan)
  - [6. Lộ Trình Thực Hiện](#6-lộ-trình-thực-hiện)

---

## 1. Tổng Quan CogMem

**CogMem** là framework bộ nhớ dài hạn cho tác nhân hội thoại, được xây dựng dựa trên nguyên lý của **khoa học nhận thức (cognitive science)** thay vì thuần túy kỹ thuật. Điểm xuất phát của CogMem là quan sát: não người không lưu trữ ký ức như một danh sách phẳng, mà tổ chức theo **nhiều hệ con bộ nhớ chuyên biệt**, mỗi hệ phục vụ một loại truy vấn khác nhau và có cơ chất thần kinh riêng.

CogMem gồm **4 đóng góp kỹ thuật chính** (và 1 thăm dò), mỗi cái giải quyết một vấn đề cụ thể được đo được trên benchmark:

| # | Đóng góp | Giải quyết vấn đề gì | Target category |
|---|----------|---------------------|-----------------|
| 1 | **Cognitively-Grounded Memory Graph** (6 networks + 7 edge types) | Thiếu bộ nhớ thói quen, nhân quả, tương lai; thiếu transition lifecycle | Causal, Preference, Prospective |
| 2 | **Lossless Node Metadata** (Raw State Snippets) | LLM nén mất thông tin chi tiết | Tất cả — cải thiện recall precision |
| 3 | **Episodic Buffer SUM + Cycle Guards** | MAX propagation bỏ qua bằng chứng đa đường | Multi-hop, Multi-session |
| 4 | **Adaptive Query Routing** | Equal-weight RRF không phân biệt query type | Temporal, Causal |

CogMem **kế thừa toàn bộ** các thành phần của HINDSIGHT (4 networks cũ, 4-way retrieval, RRF fusion, opinion reinforcement, stripped-down CARA reflect) và build on top — không loại bỏ component nào. Điều này đảm bảo fair comparison: mọi performance gap đều do kiến trúc mới, không phải do bỏ đi thứ baseline đã có.

CogMem được đánh giá theo **end-to-end accuracy (LLM-as-judge)** trên stratified subset của LongMemEval-S và LoCoMo, cho phép so sánh trực tiếp với published numbers của HINDSIGHT mà không cần rebuild lại baseline từ đầu.

---

## 2. Bối Cảnh: Tại Sao Cần CogMem?

### 2.1 Bài Toán Bộ Nhớ Hội Thoại Dài Hạn

Tác nhân hội thoại AI hiện tại không nhớ được người dùng qua nhiều phiên. Khi context window bị vượt quá, thông tin cũ biến mất — tác nhân "quên" như thể gặp lần đầu.

Các hệ thống bộ nhớ thế hệ trước cố gắng giải quyết bằng cách lưu raw conversation logs vào vector database và dùng RAG để tra cứu. Vấn đề: logs dài hàng triệu token, không có cấu trúc, và retrieval đơn giản thất bại khi câu hỏi cần suy luận qua nhiều sự kiện (multi-hop) hoặc hỏi về thói quen và nhân quả.

### 2.2 Benchmark Đánh Giá

**LongMemEval-S** (Wu et al., 2025): 500 câu hỏi, mỗi câu thuộc về một cuộc hội thoại dài (~50 phiên, ~115K tokens). Category khó: multi-session, temporal reasoning, knowledge update, preference.

**LoCoMo** (Maharana et al., 2024): 50 cuộc hội thoại, trung bình 304.9 lượt trao đổi. Category khó nhất: multi-hop (cần liên kết ≥2 facts để trả lời).

Cả hai benchmark đều build **1 Knowledge Graph riêng cho mỗi cuộc hội thoại** — không có cross-conversation retrieval. Nhiều questions có thể share cùng một conversation, do đó số KGs cần build **ít hơn đáng kể** so với số questions. Với một stratified subset N questions, số KGs cần build giảm linear theo số conversations được chọn.

**Tại sao dùng end-to-end accuracy thay vì retrieval-only Recall@k?**

Dùng LLM-as-judge cho phép **so sánh trực tiếp với published numbers của HINDSIGHT** mà không cần rebuild baseline. Nếu dùng Recall@k, phải build lại HINDSIGHT graph để có metric này — tốn chi phí build graph 2 lần (1 cho HINDSIGHT, 1 cho CogMem) trong khi end-to-end chỉ cần build graph CogMem. Chi phí evaluation do đó chủ yếu là: build CogMem graph một lần + stripped-down CARA reflect + LLM-as-judge calls trên subset.

### 2.3 HINDSIGHT — Baseline Mạnh Nhất Hiện Tại và Điểm Yếu Của Nó

HINDSIGHT (Latimer et al., 2025) là hệ thống bộ nhớ tác nhân đạt SOTA end-to-end: 91.4% trên LongMemEval với Gemini-3, 89.61% trên LoCoMo. Kiến trúc gồm 4 mạng (World, Experience, Opinion, Observation), pipeline Retain dùng LLM để trích xuất narrative facts theo kiểu **incremental** — mỗi session mới được processed qua: fact extraction → entity resolution → opinion reinforcement → observation regeneration → background merging. Pipeline Recall dùng 4-way parallel retrieval (semantic + BM25 + graph + temporal) kết hợp bằng Reciprocal Rank Fusion và cross-encoder reranking.

**Chi phí LLM trong pipeline HINDSIGHT:**

Pipeline Retain gọi LLM **incremental** — mỗi session mới trigger một chuỗi calls:

| Bước | Model | Số calls | Ghi chú |
|------|-------|----------|---------|
| Fact extraction | GPT-OSS-20B | 1 per chunk | Entity resolution piggyback vào cùng call |
| Opinion reinforcement | GPT-OSS-20B | Tăng theo số opinions tích lũy | Bottleneck — tăng tuyến tính theo số sessions |
| Observation regeneration | GPT-OSS-20B | 1 per entity khi facts thay đổi | Chạy async background |
| Background merging | GPT-OSS-20B | Nhỏ, triggered by bio info | Ít xảy ra nhất |
| Reflect (CARA) | GPT-OSS-20B | 1 per query | Generation + opinion formation |
| LLM-as-judge | GPT-OSS-120B | 1 per question | Evaluation only |

Pipeline Recall hoàn toàn không dùng LLM — HNSW vector search, BM25, spreading activation, và cross-encoder reranker (MiniLM-22M) đều là non-LLM components.

Opinion reinforcement là bottleneck lớn nhất về chi phí trong Retain: số candidates tăng dần theo sessions vì phải so sánh fact mới với toàn bộ opinions đã tích lũy. HINDSIGHT có pre-filter bằng entity overlap và cosine similarity (equation 25 trong paper) nhưng threshold θ không được ghi rõ.

**Spreading activation của HINDSIGHT dùng phép MAX:**

$$A(v, t+1) = \max_{u \in N(v)} \left[ A(u, t) \cdot w(u,v) \cdot \delta \cdot \mu(\ell) \right]$$

**RRF dùng trọng số bằng nhau** cho 4 channel ($w_i = 0.25$):

$$\text{RRF}(d) = \sum_{i \in \{\text{sem, bm25, graph, temp}\}} \frac{0.25}{60 + \text{rank}_i(d)}$$

Dù đạt SOTA, HINDSIGHT vẫn yếu trên các category khó:

| Benchmark | Category | Điểm (OSS-20B) | Nguyên nhân gốc rễ |
|-----------|----------|---------------|-------------------|
| LongMemEval | Multi-session | 79.7% | MAX bỏ qua bằng chứng tích lũy |
| LongMemEval | Temporal | 79.7% | Equal-weight RRF không ưu tiên temporal |
| LongMemEval | Preference | 66.7% | Không có Habit Network |
| LoCoMo | Multi-hop | **64.6%** | MAX chỉ lấy đường mạnh nhất |
| LoCoMo | Temporal | 79.4% | Không có Causal query type |

Ngoài ra, toàn bộ hệ thống hiện tại (kể cả HINDSIGHT) mắc một vấn đề cơ bản hơn: LLM trích xuất facts là quá trình **nén có mất mát không thể đảo ngược**. AMA-Bench (Wang et al., 2025) cho thấy Mem0 chỉ trích xuất được **0 facts** từ trajectory dài 5,022 ký tự.

---

## 3. Kiến Trúc CogMem

### 3.1 Đóng Góp 1 — Cognitively-Grounded Memory Graph (6 Networks + 7 Edge Types)

Đây là đóng góp cốt lõi nhất của CogMem — không chỉ thêm networks mới mà còn định nghĩa lại **cách các memory units liên kết với nhau** thông qua một hệ thống edge types phong phú hơn, phản ánh đúng bản chất của các quan hệ nhận thức giữa các loại ký ức khác nhau.

#### Nền tảng: Tại Sao Phân Chia Thành Nhiều Mạng?

Não người không dùng một vùng duy nhất để lưu mọi loại ký ức. Squire & Zola-Morgan (1991) phân loại bộ nhớ dài hạn thành các hệ con với cơ chất thần kinh khác nhau — tổn thương hippocampus làm mất episodic memory nhưng không ảnh hưởng đến procedural memory; tổn thương basal ganglia làm mất habit memory nhưng không ảnh hưởng semantic memory. Sự phân ly này không phải ngẫu nhiên mà phản ánh các chức năng nhận thức khác nhau cần được tối ưu độc lập.

Trong hệ thống AI, lợi ích thực tiễn: (1) mỗi mạng có schema node riêng phù hợp với loại thông tin, (2) retrieval có thể định hướng đến đúng mạng theo query type, (3) ablation study sạch — có thể bật/tắt từng mạng để đo đóng góp riêng lẻ.

#### 6 Mạng Bộ Nhớ

| Mạng | Cơ sở nhận thức | Neural substrate | Lưu trữ gì | Ví dụ |
|------|----------------|------------------|------------|-------|
| **World** | Semantic Memory (Tulving, 1972) | Temporal-Parietal Cortex | Sự kiện khách quan, kiến thức chung | "DI chuyên về LLM infrastructure" |
| **Experience** | Episodic Memory (Tulving, 1983) | Hippocampus | Sự kiện cá nhân có gắn thời gian-không gian | "User vào làm DI tháng 4/2024" |
| **Opinion** | Belief System / Attitude Memory | Prefrontal Cortex | Nhận xét chủ quan + confidence score | "Python tốt nhất cho ML (0.85)" |
| **Observation** | Entity Summary Layer | — | Tổng hợp khách quan về entities | "Alice: proactive, detail-oriented" |
| **Habit ★** | Habit Memory · S-R (Squire & ZM, 1991) | Basal Ganglia | Mẫu hành vi lặp lại, tự động | "User luôn check email trước standup" |
| **Intention ★** | Prospective Memory (Brandimonte et al., 1996) | Prefrontal Cortex | Kế hoạch và mục tiêu tương lai có deadline | "User định học Rust trước Q3" |
| **Action-Effect ★** | A-O Learning / TEC (Hommel et al., 2001) | Prefrontal + Premotor Cortex | Bộ ba nhân quả Precondition→Action→Outcome | "Lọc shop rating cao → giao hàng nhanh hơn" |

> ★ = Mạng bộ nhớ mới, không có trong HINDSIGHT hay bất kỳ KG-based memory system nào trước đây.

#### Phân Biệt Habit vs. Action-Effect: Phân Ly S-R / A-O

Dickinson & Balleine (1994) thiết lập **devaluation test** để phân biệt S-R và A-O: nếu outcome mất giá trị, hành động A-O bị ức chế ngay — trong khi hành động S-R (thói quen) **tiếp tục** vì nó không phụ thuộc vào giá trị của outcome.

| Chiều | Habit Network (S-R) | Action-Effect Network (A-O) |
|-------|--------------------|-----------------------------|
| Neural circuit | Basal Ganglia (dorsolateral striatum) | Prefrontal + Premotor Cortex |
| Cơ chế | Kích thích → phản ứng **tự động**, không cần ý thức | Hành động được **chủ động chọn** vì tạo ra kết quả mong muốn |
| Devaluation | Thói quen **tồn tại** kể cả khi kết quả mất giá trị | Hành động **bị ức chế** nếu kết quả mất giá trị |
| Frequency | Frequency-based — lặp lại nhiều lần mới hình thành | Instance-based — một lần quan sát đủ để ghi nhận |
| Downstream use | Personalization, preference retrieval | Causal inference, "tại sao user làm X?" |

**Ví dụ phân biệt:**
- *"User luôn check email trước standup"* → **Habit** (S-R): pattern tần suất, không cần biết lý do
- *"Lọc shop rating cao → giao hàng nhanh hơn"* → **Action-Effect** (A-O): quan hệ nhân quả tường minh, có thể bị ức chế nếu không cần giao nhanh nữa

**Schema node Intention:**
```json
{
  "network_type": "intention",
  "goal": "Học Rust đủ để viết inference server",
  "deadline": "2025-Q3",
  "status": "planning",
  "priority": 0.8,
  "fulfilled_at": null,
  "linked_experience": null
}
```

**Schema node Action-Effect:**
```json
{
  "network_type": "action_effect",
  "precondition": "Embedding latency > 100ms",
  "action": "Switch sang int8 quantization",
  "outcome": "Latency giảm 75% (180ms → 45ms)",
  "confidence": 0.92,
  "observation_count": 1,
  "devalue_sensitive": true
}
```

#### 7 Edge Types và Intention Lifecycle

CogMem định nghĩa 7 loại edge, trong đó có 3 loại mới so với HINDSIGHT:

| Edge type | Hướng | Capture gì | Ví dụ |
|-----------|-------|-----------|-------|
| **Entity** | Bidirectional | Cùng entity được nhắc | Alice → mọi node về Alice |
| **Temporal** | Directed | Thứ tự thời gian | e_join → e_promo (9 tháng) |
| **Semantic** | Undirected | Tương đồng ngữ nghĩa (cosine ≥ θ) | ML Engineer ↔ AI Team |
| **Causal** | Directed | Opinion/belief shape hành động | o_python → w_proj |
| **S-R link ★** | Directed | Habit reinforces/contributes to Observation | h_email → obs_work |
| **A-O causal ★** | Directed | Precondition→Action→Outcome triple | ae_quant → e_fix |
| **Transition ★** | Directed + typed | State change của concept theo thời gian, across networks | i_rust → e_rust_done |

**Transition edge** là đóng góp mới nhất, được lấy cảm hứng từ causality edges của AMA-Agent (Wang et al., 2025) — họ dùng directed edges để capture state transitions trong environment ($s_t \xrightarrow{action} s_{t+1}$); CogMem dùng tương tự để capture lifecycle transitions của memory units across networks.

**Typed transition edges và Intention lifecycle:**

Trong incremental graph building, một sự kiện có thể là dự định ở thời điểm $t_0$ nhưng trở thành experience ở $t_1$. Phải giữ lại cả hai vì chúng có giá trị khác nhau khi trả lời câu hỏi:

- *"Alice có biết Rust không?"* → cần Experience node (đã học xong)
- *"Alice học Rust vì lý do gì?"* → cần Intention node gốc (motivation)
- *"Alice mất bao lâu để học Rust?"* → cần cả hai để tính delta thời gian

Các typed transitions:

```
fulfilled_by    : Intention → Experience   (kế hoạch được thực hiện)
abandoned       : Intention → null         (status update only, node giữ nguyên)
triggered       : Experience → Intention   (review với Minh → plan thêm cache)
enabled_by      : Intention → Intention    (i_rust enables i_paper)
revised_to      : Opinion → Opinion        (confidence shift lớn)
contradicted_by : World → World            (fact bị update bởi fact mới)
```

**Cơ sở nhận thức cho Transition edge:** Brandimonte et al. (1996) mô tả prospective memory discharge — intention khi fulfilled không biến mất mà được re-encoded như autobiographical event trong episodic memory. Transition edge model chính xác cơ chế này.

**Incremental update rule cho Intention node:**

Khi retain session mới phát hiện event là fulfillment của existing intention:
1. Intention node: `status → fulfilled`, `fulfilled_at → timestamp`, `linked_experience → new_exp_id`
2. Tạo Experience node mới với timestamp thực tế
3. Tạo Transition edge `i_rust ──fulfilled_by──▶ e_rust_done`

Khi phát hiện abandonment:
1. Intention node: `status → abandoned` — không xóa node, giữ để biết motivation lịch sử
2. Không tạo Experience node

**Tại sao thiết kế này đúng hơn so với HINDSIGHT:**

HINDSIGHT không có Intention network nên không phân biệt được "Alice đang plan học Rust" (future intent) với "Alice đã học Rust" (past experience) — cả hai bị lưu lẫn vào Experience hoặc World, mất temporal semantics của prospective planning.

#### Phân Biệt Action-Effect Network vs. Mem^p

Mem^p (Sun et al., 2025) cũng lưu "procedural memory" nhưng ở cấp độ hoàn toàn khác:

| Chiều | Action-Effect Network (CogMem) | Mem^p |
|-------|-------------------------------|-------|
| Granularity | **Action-level** A-O triples từ NL dialogue | **Task-level** scripts từ agent trajectory logs |
| Extraction source | Hội thoại người-AI tự nhiên | JSON, HTML, tool call logs của agent |
| Goal | Trả lời "tại sao user làm X?" trong personal memory | Lặp lại task thành công trong môi trường agent mới |

Hai đóng góp orthogonal — không chồng chéo, không thay thế nhau.

---

### 3.2 Đóng Góp 2 — Lossless Node Metadata (Raw State Snippets)

#### Vấn Đề: Mọi Hệ Thống Hiện Tại Đều Nén Có Mất Mát

Tất cả memory systems hiện tại — kể cả HINDSIGHT — dùng LLM để nén hội thoại thành narrative facts trước khi lưu. Đây là quá trình **lossy và không thể đảo ngược**:

```
Hội thoại gốc:
"Hôm nay mình vừa nghỉ việc ở VCCorp rồi. Buồn lắm vì 
team cũ rất thân. Mình nhận offer từ DI với mức lương cao 
hơn 40%. Sẽ bắt đầu từ 1/4. Vị trí là ML Engineer..."

LLM trích xuất thành narrative fact:
→ "User chuyển từ VCCorp sang DI tháng 4/2024, vị trí ML Engineer"
→ Mất: cảm xúc ("buồn"), lý do gắn bó team cũ, con số lương cụ thể (40%)
```

Hậu quả: khi user hỏi "Mức lương khi chuyển sang DI cao hơn bao nhiêu?", hệ thống truy xuất đúng node nhưng không có đủ thông tin để trả lời.

#### Giải Pháp: 2-Layer Node Schema

CogMem thêm field **`raw_snippet`** — văn bản nguồn gốc lossless — đi kèm mỗi node:

```json
{
  "id": "node-uuid",
  "network_type": "experience",
  "narrative_fact": "User chuyển từ VCCorp sang DI tháng 4/2024",
  "embedding": [...],
  "raw_snippet": "Hôm nay mình vừa nghỉ việc ở VCCorp rồi. Buồn lắm vì team cũ rất thân. Mình nhận offer từ DI với mức lương cao hơn 40%...",
  "timestamp": "2024-03-12T09:30:00Z",
  "confidence": 0.92,
  "expiry": null
}
```

**Hai layer phục vụ hai mục tiêu khác nhau:**
- `narrative_fact` + `embedding`: dùng cho **retrieval** — semantic search, BM25, graph traversal cần vector nhỏ gọn
- `raw_snippet`: inject vào context window **sau khi** node được retrieve, để LLM generation có đủ chi tiết nguyên bản

Tách biệt này quan trọng vì hai mục tiêu có yêu cầu mâu thuẫn: retrieval cần compression (vector ngắn = precision cao), generation cần verbatim (text đầy đủ = recall không mất mát).

#### Cơ Sở Nhận Thức

Fuzzy Trace Theory (Brainerd & Reyna, 2004) mô tả con người lưu đồng thời hai dạng biểu diễn:
- **Gist**: tóm tắt ngữ nghĩa — dùng để nhận dạng nhanh
- **Verbatim**: chi tiết nguyên bản — dùng khi cần recall chính xác

`narrative_fact` ≈ gist · `raw_snippet` ≈ verbatim.

---

### 3.3 Đóng Góp 3 — Episodic Buffer với SUM Spreading Activation + Cycle Guards

#### Vấn Đề: MAX Chỉ Lấy Đường Mạnh Nhất

Spreading activation của HINDSIGHT dùng phép MAX:

$$A(v, t+1) = \max_{u \in N(v)} \left[ A(u, t) \cdot w(u,v) \cdot \delta \cdot \mu(\ell) \right]$$

Phép MAX chỉ truyền năng lượng theo đường **mạnh nhất duy nhất**. Nếu có 3 nguồn bằng chứng yếu cùng trỏ về một node, MAX vẫn chỉ lấy một — hai nguồn còn lại bị bỏ qua hoàn toàn. Đây là lý do multi-hop thất bại.

#### Cơ Sở Nhận Thức: Episodic Buffer

Mô hình Bộ Nhớ Làm Việc của Baddeley (2000) giới thiệu **Episodic Buffer** — không gian làm việc tích hợp tạm thời liên kết thông tin từ nhiều hệ con bộ nhớ dài hạn thành các episode mạch lạc. Episodic Buffer **tích lũy bằng chứng từ nhiều nguồn**, không phải chọn một nguồn mạnh nhất.

#### Giải Pháp: Phép SUM Thay Phép MAX

$$A(v, t+1) = \text{clip}\left[ A(v,t) + \delta \cdot \sum_{u \in N(v)} \left[ A(u,t) \cdot w(u,v) \cdot \mu(\text{edge}) \cdot \text{refractory}(u) \right], \; A_{\max} \right]$$

| Khía cạnh | HINDSIGHT (MAX) | CogMem (SUM) |
|-----------|----------------|--------------|
| Bằng chứng đa đường | Bị loại bỏ — chỉ lấy 1 đường tốt nhất | Được tích lũy — mọi đường đóng góp |
| Yêu cầu entry point | Phải chính xác cao | Chỉ cần xấp xỉ đúng hướng |
| Độ bền multi-hop | Yếu — một liên kết sai phá cả chain | Mạnh — tín hiệu yếu bổ sung nhau |
| Nguy cơ phân kỳ | Không | **Có — cần cycle guards** |

#### Ba Cycle Guards

Phép SUM trên đồ thị có chu trình **sẽ phân kỳ** nếu không có cơ chế kiểm soát. Ba guards giải quyết ba dạng mất ổn định khác nhau:

**Guard 1 — Local Refractory Period** (Hodgkin & Huxley, 1952)

$$\text{refractory}(u) = \begin{cases} 0 & \text{nếu } u \text{ đã kích hoạt ở bước } t \\ 1 & \text{ngược lại} \end{cases}$$

Chặn ping-pong giữa 2 node ($u \leftrightarrow v$).

**Guard 2 — Global Firing Quota** (Baddeley, 2000)

Mỗi node có counter `fire_count`. Khi `fire_count(v) ≥ max_fires` (mặc định = 3), node không nhận thêm activation. Chặn chu trình dài ≥ 3 nodes ($A \to B \to C \to A$).

**Guard 3 — Saturation Threshold** (Wilson-Cowan, 1972)

$$A(v, t+1) = \min\left( A(v, t+1)_{\text{raw}}, \; A_{\max} \right)$$

Chặn score explosion, đảm bảo diversity trong retrieval. `A_max = 2.0`.

**Tại sao cần cả 3:** Guard 1 chặn chu trình 2 node, Guard 2 chặn chu trình dài, Guard 3 chặn score explosion kể cả khi không có chu trình. Thiếu bất kỳ guard nào, thuật toán có thể phân kỳ trên một lớp đồ thị nhất định.

---

### 3.4 Đóng Góp 4 — Adaptive Query Routing

#### Vấn Đề: Equal-Weight RRF Không Phân Biệt Ý Định Truy Vấn

HINDSIGHT dùng trọng số bằng nhau ($w = 0.25$ mỗi channel). Điều này không hợp lý vì các loại câu hỏi khác nhau cần các channel khác nhau:

$$\text{RRF}(d) = \sum_{i \in \{\text{sem, bm25, graph, temp}\}} \frac{0.25}{60 + \text{rank}_i(d)}$$

#### Giải Pháp: Dynamic Channel Weights theo Query Type

Thêm bước **query classification** trước retrieval:

$$\text{RRF}_{\text{adaptive}}(d, q) = \sum_{i} w_i(q) \cdot \frac{1}{60 + \text{rank}_i(d)}, \quad \sum_i w_i(q) = 1$$

| Loại Query | Detection Signal | $w_\text{sem}$ | $w_\text{bm25}$ | $w_\text{graph}$ | $w_\text{temp}$ | Đặc biệt |
|-----------|-----------------|----------------|-----------------|------------------|-----------------|----------|
| **Temporal** | Date/time expressions | 0.20 | 0.20 | 0.20 | **0.40** | — |
| **Entity** | Named entities, proper nouns | 0.20 | 0.30 | **0.40** | 0.10 | — |
| **Multi-hop** | Relational pronouns, indirect ref | 0.15 | 0.10 | **0.50** | 0.25 | — |
| **Causal ★** | "Tại sao / lý do / vì sao" | 0.10 | 0.10 | **0.40** | 0.10 | Ưu tiên Action-Effect nodes + Transition edges |
| **Prospective ★** | "Đang plan / dự định / sắp" | 0.20 | 0.15 | **0.35** | **0.30** | Chỉ query Intention nodes có status=planning |
| **Semantic** | Descriptive, không có entity/time | **0.50** | 0.20 | 0.20 | 0.10 | — |

Hai query types mới so với v2:
- **Causal**: khi phát hiện query hỏi về lý do/nhân quả, ưu tiên Action-Effect nodes và traverse A-O causal edges
- **Prospective**: khi phát hiện query về tương lai/kế hoạch, chỉ query Intention nodes có `status=planning`, kết hợp temporal channel để rank theo deadline gần nhất

#### Cơ Sở Nhận Thức

Attentional selection trong nhận thức: não người tùy theo bản chất nhiệm vụ mà tập trung vào hệ bộ nhớ phù hợp. Câu hỏi "khi nào?" kích hoạt hippocampal temporal circuits; câu hỏi "tại sao?" kích hoạt prefrontal causal reasoning; câu hỏi "sắp làm gì?" kích hoạt prospective memory retrieval.

---

### 3.5 Đóng Góp 5 — Hierarchical Knowledge Graph (Thăm Dò)

> Đây là đóng góp **thăm dò** (exploratory), sẽ được implement sau khi hoàn thành 4 đóng góp chính nếu thời gian cho phép.

Tất cả facts trong HINDSIGHT đều ở cùng một level. Schema Theory (Bartlett, 1932) và Basic Level Categorization (Rosch, 1978) mô tả não người tổ chức kiến thức theo cấu trúc phân cấp:

| Level | Tên | Mô tả | Ví dụ |
|-------|-----|-------|-------|
| 0 | Abstract | Schema cấp cao | "User làm trong ngành AI/tech tại Hà Nội" |
| 1 | Basic (anchor) | Facts cấp trung — đích retrieval chính | "User là ML Engineer tại DI từ 4/2024" |
| 2 | Specific | Facts thực thể cấp thấp | "User join DI team AI tháng 6/2024" |

---

## 4. Thiết Kế Thực Nghiệm

### 4.1 Evaluation Strategy

**Metric:** End-to-end accuracy (LLM-as-judge), phân tích per-category. Cùng judge setup với HINDSIGHT để so sánh trực tiếp với published numbers.

**Lý do chọn end-to-end thay vì Recall@k:**

Dùng LLM-as-judge cho phép so sánh trực tiếp với published HINDSIGHT numbers mà không cần rebuild baseline. Recall@k yêu cầu build lại HINDSIGHT graph (tốn cost build 2 lần), trong khi end-to-end chỉ cần build CogMem graph + simple generation step.

**Evaluation pipeline CogMem:**

1. Build CogMem graph offline một lần per conversation (cached, reused cho tất cả questions)
2. Recall: Adaptive Router → 4-way retrieval → RRF → reranker → top-k nodes
3. Reflect (stripped-down CARA): retrieved nodes + question → LLM generate answer
4. Judge: LLM-as-judge(question, gold answer, generated answer) → binary score

**Stripped-down CARA:** Giữ opinion reinforcement (liên quan trực tiếp đến memory architecture — graph được build incremental nên các thành phần chủ quan phải được cập nhật liên tục). Bỏ behavioral profile (Θ = skepticism, literalism, empathy) và background merging vì orthogonal với contribution của CogMem.

**Subset strategy:**

Do resource constraints, evaluate trên stratified subset — giữ nguyên category distribution của từng benchmark. Số KGs cần build giảm linear theo số conversations được chọn (nhiều questions có thể share cùng conversation).

| Benchmark | Full | Subset | Lý do |
|-----------|------|--------|-------|
| LongMemEval-S | 500 questions | 100–150 | 20–30 per category, đủ significance |
| LoCoMo | 50 conversations | 20–25 | Giữ đủ question types |

**Tất cả experiments dùng cùng SLM local** cho retain pipeline (ví dụ Qwen3-7B) — bao gồm cả HINDSIGHT baseline rebuild. Điều này đảm bảo fair comparison: mọi performance gap đều do kiến trúc, không phải do model scale. Ghi rõ trong thesis rằng đây là controlled comparison, không phải reproduction của published numbers.

### 4.2 Ablation Study

Mỗi thực nghiệm thêm đúng **một component** so với baseline:

| Thực nghiệm | Components | Hypothesis |
|-------------|------------|------------|
| **E1** — Baseline | HINDSIGHT rebuilt với SLM | Điểm tham chiếu |
| **E2** | E1 + Habit Network + S-R links | Cải thiện Preference category |
| **E3** | E1 + Intention Network + Transition edges | Cải thiện Prospective queries |
| **E4** | E1 + Action-Effect Network + A-O causal links | Cải thiện Causal category |
| **E5** | E1 + Adaptive Query Router (incl. Causal + Prospective types) | Cải thiện Temporal + Multi-hop |
| **E6** | E1 + Episodic Buffer SUM + 3 Cycle Guards | Cải thiện Multi-hop + Multi-session |
| **E7 — Full CogMem** | E1 + tất cả | Tổng hợp — so sánh chính |
| **E8** *(Optional)* | E7 + Hierarchical KG | Cải thiện general queries |

### 4.3 Baselines So Sánh

| Hệ thống | Vai trò |
|----------|---------|
| **HINDSIGHT** (rebuilt, SLM) | Baseline chính — controlled comparison |
| **HINDSIGHT** (published numbers) | Reference — upper bound với strong model |
| **HippoRAG2** | Graph-based retrieval SOTA |
| **Mem^p** | Related work, khác paradigm |

---

## 5. Các Nghiên Cứu Liên Quan

**Memory systems:** HINDSIGHT (Latimer et al., 2025) — baseline chính. HippoRAG2 — graph retrieval. Mem0, Zep, MemGPT (Packer et al., 2023) — production và OS-like memory.

**Agentic memory:** Mem^p (Sun et al., 2025) — task-level procedural từ agent trajectory. AMA-Bench / AMA-Agent (Wang et al., 2025) — benchmark và memory system cho agent-centric memory (khác paradigm với CogMem — dialogue-centric). AMA-Agent's Causality Graph là inspiration trực tiếp cho Transition edges của CogMem.

**Cognitive science foundations:** Tulving (1972, 1983) · Squire & Zola-Morgan (1991) · Dickinson & Balleine (1994) · Hommel et al. / TEC (2001) · Brandimonte et al. (1996) · Baddeley (2000) · Hodgkin & Huxley (1952) · Wilson-Cowan (1972) · Bartlett (1932) · Brainerd & Reyna (2004).

---

## 6. Lộ Trình Thực Hiện

| Giai đoạn | Nội dung | Thời gian |
|-----------|----------|-----------|
| **1 — Nền tảng** | Tái tạo HINDSIGHT với SLM local (E1 baseline) · Evaluation pipeline end-to-end · 2-layer node schema · Stratified subset selection | Tuần 1–2 |
| **2 — Core networks** | Habit + Action-Effect + Intention Networks · S-R + A-O causal + Transition edge types · Intention lifecycle rules (fulfilled_by, abandoned, triggered) · Stripped-down CARA retain opinion reinforcement | Tuần 3–5 |
| **3 — Retrieval** | SUM activation + 3 cycle guards · Adaptive Query Router (6 types incl. Causal + Prospective) | Tuần 5–6 |
| **4 — Thực nghiệm** | Chạy E1–E6 ablation · E7 Full CogMem vs. baselines · Phân tích per-category · Error analysis + case studies | Tuần 7–9 |
| **5 — Hoàn thiện** | E8 Hierarchical KG (nếu kịp) · Viết báo cáo · So sánh với HINDSIGHT published reference | Tuần 9–10 |

---

*Tài liệu tổng hợp từ các thảo luận 05/03/2026 – 17/03/2026. Phiên bản: 17/03/2026.*