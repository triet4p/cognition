# Cognition
## A Cognitively-Grounded, High-Performance Agent Framework

> *"Build agents that think, remember, act, and communicate — like humans do."*

**Lê Minh Triết**
*Tháng 3, 2026 — Draft v1.0*

---

## Lời Mở Đầu

Trong vài năm qua, các agent framework như LangChain, AutoGen, và CrewAI đã giúp hàng triệu developer xây dựng AI agents. Nhưng tất cả chúng đều có chung một vấn đề căn bản: chúng được xây dựng như *công cụ gọi API*, không phải như *hệ thống có trí tuệ*.

Một agent thực sự không chỉ cần biết "gọi tool nào tiếp theo." Nó cần nhớ những gì đã xảy ra hôm qua, nhận ra pattern lặp lại từ hàng trăm tương tác trước, có khả năng lên kế hoạch cho tương lai, và giao tiếp với các agent khác như một thành viên trong một tổ chức.

Nói cách khác — một agent thực sự cần được thiết kế giống như cách con người hoạt động.

**Cognition** là câu trả lời cho câu hỏi đó.

---

## I. Bức Tranh Lớn

### Tại sao các framework hiện tại chưa đủ?

Nhìn vào bất kỳ agent framework phổ biến nào, bạn sẽ thấy một mô hình quen thuộc:

```
User Input → LLM → Tool Call → LLM → Output
```

Đây là một vòng lặp đơn giản. Không có memory thực sự — chỉ có conversation buffer được nhét vào context window. Không có học hỏi liên tục — mỗi session bắt đầu từ đầu. Không có reasoning có cấu trúc — LLM tự quyết định mọi thứ dựa trên prompt. Không có parallelism thực sự — Python GIL đảm bảo điều đó.

Hậu quả thực tế:
- **Agents quên mọi thứ** giữa các session, dù đã "nói chuyện" hàng trăm lần
- **Latency không ổn định** — garbage collector của Python gây spike bất ngờ
- **Không scale được** khi có nhiều agents cần giao tiếp với nhau real-time
- **Memory là vector store** — tốt cho similarity search, nhưng không thể trả lời "Alice là đồng nghiệp của ai?"

### Vision của Cognition

Cognition được thiết kế từ một câu hỏi khác: **nếu chúng ta xây dựng một agent giống như cách não người hoạt động, nó sẽ trông như thế nào?**

Não người không lưu trữ ký ức dưới dạng vector. Nó có nhiều hệ thống memory khác nhau — mỗi hệ thống phục vụ một mục đích khác nhau. Nó consolidate ký ức trong lúc ngủ. Nó nhận ra pattern và biến chúng thành habit. Nó lên kế hoạch cho tương lai. Nó giao tiếp với người khác qua nhiều kênh khác nhau cùng lúc.

Cognition là framework đầu tiên đưa những nguyên lý đó vào thiết kế cốt lõi — không phải như một tính năng bổ sung, mà như nền tảng kiến trúc.

---

## II. Tại Sao Rust?

Đây là câu hỏi hợp lý. Python đang thống trị AI ecosystem. Tại sao lại đi ngược dòng?

### Vấn đề của Python với agents

Python được thiết kế cho scripting và prototyping, không cho hệ thống concurrent phức tạp. Trong bối cảnh agent framework, điều này tạo ra ba vấn đề không thể giải quyết ở tầng application:

**GIL (Global Interpreter Lock)** đảm bảo chỉ một thread Python chạy tại một thời điểm. Một agent cần đồng thời: nhận input từ nhiều nguồn, query memory, gọi nhiều tools, stream LLM response. Với GIL, đây là *giả* concurrency — các tác vụ thực ra vẫn tuần tự.

**Unpredictable latency** từ garbage collector. Trong hệ thống real-time, một GC pause 50ms là không chấp nhận được. Python không cung cấp bất kỳ cơ chế nào để tránh điều này.

**Memory overhead** của Python object model. Một dict với 1000 entries trong Python tốn gấp 8–10x memory so với equivalent data structure trong Rust. Với agents cần giữ large memory stores, đây là vấn đề thực tế.

### Rust giải quyết những điểm trên tại tầng language

Rust không có GIL — threads thực sự chạy song song. Rust không có garbage collector — memory được quản lý tại compile time, không có pause. Rust's ownership model đảm bảo thread safety mà không cần runtime checks.

Quan trọng hơn: Rust là ngôn ngữ duy nhất cung cấp **zero-cost abstractions**. Điều này có nghĩa là một `Skill` trait, một `MemoryBackend` trait, tất cả các abstraction layer của framework — compile xuống machine code tương đương viết thẳng bằng C, không có overhead nào.

### Rust không có nghĩa là "chỉ dùng được bởi Rust developers"

Cognition cung cấp Python bindings thông qua PyO3. Một Python developer có thể:

```python
import cognition

agent = cognition.AgentBuilder() \
    .with_cogmem() \
    .with_llm(cognition.Anthropic()) \
    .with_skill(cognition.WebSearch()) \
    .build()

await agent.chat("What did we discuss last week?")
```

Toàn bộ performance của Rust, interface quen thuộc của Python. Cả hai audiences đều được phục vụ.

---

## III. Kiến Trúc Tổng Thể

Cognition được tổ chức thành năm layer độc lập, giao tiếp với nhau qua trait boundaries rõ ràng.

```
╔═══════════════════════════════════════════════════════════╗
║              User / Application Layer                     ║
║         Rust API  ─────────────  Python API               ║
╠═══════════════════════════════════════════════════════════╣
║                  Runtime Layer                            ║
║    AgentBuilder → Agent → EventLoop → Scheduler           ║
╠═══════════╦═══════════════╦═══════════════╦═══════════════╣
║  Memory   ║    Skills     ║     LLM       ║    Comm       ║
║  Layer    ║    Layer      ║    Layer      ║    Layer      ║
╠═══════════╩═══════════════╩═══════════════╩═══════════════╣
║                    Core Layer                             ║
║   MemoryBackend  Skill  LlmClient  Transport  traits      ║
╚═══════════════════════════════════════════════════════════╝
```

Nguyên tắc thiết kế: **Core layer không depend vào bất cứ thứ gì ngoài std**. Mọi crate khác depend vào Core, không depend vào nhau trực tiếp. Điều này đảm bảo mỗi layer có thể được swap ra mà không ảnh hưởng các layer khác.

---

## IV. Năm Layer — Ý Tưởng Chi Tiết

### Layer 1: Core — Ngôn Ngữ Chung

Core là tập hợp các trait và type definitions. Nó không chứa bất kỳ implementation nào — chỉ có *contracts* mà mọi component phải tuân theo.

**Agent trait** định nghĩa vòng lặp cơ bản của một agent: nhận input (*perceive*), suy nghĩ (*think*), hành động (*act*), và học từ kết quả (*reflect*). Đây là cycle cơ bản, giống như sensorimotor loop trong khoa học nhận thức.

**MemoryBackend trait** định nghĩa những gì một memory system phải làm: lưu trữ, truy xuất, consolidate, và decay. Không có gì về *cách* làm — đó là quyết định của từng implementation.

**Skill trait** định nghĩa một đơn vị hành động: có input type, output type, và một manifest mô tả cho LLM biết khi nào nên dùng skill này. Đây là thiết kế trực tiếp lấy cảm hứng từ cách Claude's skills được định nghĩa.

**LlmClient trait** định nghĩa interface đồng nhất cho mọi LLM backend — completion, streaming, tool calling — không leak implementation details của từng provider.

**Transport trait** định nghĩa cách agents giao tiếp với nhau — không quan tâm là WebSocket, gRPC hay protocol khác.

Tại sao quan trọng: nhờ Core layer, một agent được viết với Anthropic backend có thể switch sang Ollama chỉ bằng một dòng config. Một agent đang dùng CogMem memory có thể được tested với SimpleMemory mà không cần thay đổi bất kỳ logic nào.

---

### Layer 2: Memory — Trái Tim của Agent

Đây là layer khác biệt lớn nhất của Cognition so với mọi framework hiện tại.

#### Memory như một hệ sinh thái, không phải một container

Hầu hết agent frameworks đều có một nơi lưu memory — thường là vector store hoặc conversation buffer. Cognition có *nhiều loại memory khác nhau*, mỗi loại phục vụ một mục đích nhận thức cụ thể.

Mỗi loại memory có đặc điểm riêng về cách lưu, cách retrieve, và cách decay. Cùng một thông tin có thể tồn tại ở nhiều dạng trong nhiều hệ thống memory, giống như cách não người xử lý thông tin.

#### CogMem: Default Memory Architecture

Cognition ship với CogMem làm default — một kiến trúc memory dựa trên khoa học nhận thức. Chi tiết về CogMem sẽ được trình bày ở phần V.

#### Pluggable Memory

CogMem không phải lựa chọn duy nhất. Cognition định nghĩa `MemoryBackend` trait — bất kỳ ai cũng có thể implement memory architecture của riêng mình:

- **SimpleMemory** — in-memory, không persistent, tốt cho prototyping
- **CogMemBackend** — default, cognitively grounded, với optional persistence
- **VectorMemory** — traditional vector store cho những ai muốn
- **Custom** — implement trait, plug vào framework

#### Memory Consolidation

Một tính năng ít ai nghĩ đến: memory consolidation. Trong não người, ký ức được củng cố trong lúc ngủ — thông tin từ working memory được transfer sang long-term memory, patterns được extracted.

Cognition simulate điều này: agent runtime có một consolidation cycle chạy định kỳ. Working memory items có salience thấp được đẩy xuống episodic store. Patterns lặp lại được extracted thành procedural memory. Đây không phải metaphor — đây là mechanism thực sự trong architecture.

---

### Layer 3: Skills — Cách Agent Hành Động

#### Skill là gì?

Một Skill là một đơn vị hành động có thể tái sử dụng. Nó có:
- **Input type** rõ ràng — không phải "bất kỳ JSON nào"
- **Output type** rõ ràng — LLM biết chính xác nó sẽ nhận được gì
- **Manifest** — mô tả bằng ngôn ngữ tự nhiên cho LLM biết khi nào dùng skill này, giống như cách Claude's skills được mô tả trong markdown
- **Safety level** — sandboxed (WASM), trusted (native), system (full access)

#### Implement Skills theo chuẩn của Anthropic
Tạo ra các thư mục skills chứa các file SKILL.md, references, scripts như Claude => Dễ bảo trì và phát triển, tiến hóa các workflow


#### Skill Composition

Skills không chỉ được gọi độc lập. Cognition cho phép compose skills thành pipelines — output của skill này là input của skill tiếp theo. Compiler kiểm tra type compatibility tại compile time, không phải runtime. Điều này loại bỏ toàn bộ một class of bugs thường gặp trong Python agent frameworks.

#### WASM Sandboxing

Skills không tin tưởng — ví dụ code execution từ user — chạy trong WASM sandbox. Chúng không có access vào filesystem, network, hay system resources trừ khi được explicitly granted. Đây là tính năng security mà không có Python framework nào cung cấp được ở mức này.

---

### Layer 4: LLM — Bộ Não Có Thể Thay Thế

LLM layer cung cấp unified interface cho mọi provider lớn:

- **Anthropic** — Claude family, với native tool_use support
- **OpenAI** — GPT family và compatible APIs
- **Google** — Gemini family
- **OpenRouter** — access nhiều models qua một API
- **Ollama** — local models, không cần internet, privacy-first

Agent không biết và không cần biết nó đang dùng provider nào. Điều này có nghĩa là:

- Switch từ Anthropic sang Ollama để testing local: một dòng config
- A/B test hai models trên cùng một agent logic: không cần thay đổi code
- Fallback tự động khi provider có downtime: built-in

#### Streaming First

Mọi LLM interaction trong Cognition đều được thiết kế với streaming là default, không phải exception. Token stream được xử lý as an async Rust stream — low latency, backpressure-aware, không allocate unnecessarily.

#### Structured Output và Tool Calling

Cognition tự động map từ Skill manifests sang tool definitions của từng provider. Developer định nghĩa skill một lần — framework lo phần dịch sang format của Anthropic, OpenAI, hay Google.

---

### Layer 5: Communication — Agent như Thành Viên Xã Hội

Đây là layer ít được chú ý nhất trong các framework hiện tại, nhưng lại là điều phân biệt một agent đơn lẻ với một *hệ thống* agents.

#### Multi-Protocol

Cognition hỗ trợ nhiều giao thức giao tiếp:

**WebSocket** cho real-time bidirectional communication — phù hợp cho agents cần nhận và gửi messages liên tục, interactive applications, và human-agent collaboration.

**gRPC** (qua tonic) cho service-to-service communication giữa agents — strongly typed, efficient binary protocol, built-in load balancing. Phù hợp cho microservices-style multi-agent systems.

**HTTP/SSE** cho streaming responses về phía client — standard web protocol, dễ integrate với mọi frontend.

**MCP Server** — một agent Cognition có thể expose chính nó như một MCP server. Điều này có nghĩa là Claude, Cursor, hay bất kỳ MCP-compatible tool nào đều có thể interact với agent như một tool. Agent của bạn trở thành một MCP server mà người khác có thể dùng.

#### Agent-to-Agent Messaging

Agents giao tiếp qua structured messages với routing rõ ràng: point-to-point, broadcast, hay group messaging. Một orchestrator agent có thể delegate tasks, collect results, và aggregate — tất cả qua cùng một communication abstraction.

#### CommHub

CommHub là router trung tâm: nhận messages từ bất kỳ transport nào, route đến đúng recipient, handle delivery failures. Agent không cần biết đang giao tiếp qua protocol nào — CommHub lo phần đó.

---

## V. CogMem — Bộ Nhớ Lấy Cảm Hứng Từ Khoa Học Nhận Thức

### Bối Cảnh

CogMem là một memory architecture được phát triển như một proposal nghiên cứu, lấy nền tảng từ khoa học nhận thức và neuroscience. Nó không phải là cách engineer thường thiết kế memory system — nó là cách *neuroscientist* hiểu memory hoạt động, được translate thành một hệ thống có thể implement.

Ý tưởng cốt lõi: con người không có một "bộ nhớ" — chúng ta có *nhiều hệ thống memory* chuyên biệt, mỗi cái được tối ưu cho một loại thông tin và một loại truy xuất khác nhau. Một agent được thiết kế giống như vậy sẽ có khả năng nhớ và reasoning mạnh hơn nhiều so với một agent chỉ có vector store.

### Sáu Memory Networks

#### 1. Working Memory Buffer

Tương đương với "bộ nhớ ngắn hạn" trong tâm lý học nhận thức. Working memory là những gì agent đang chú ý *ngay lúc này*: context của conversation hiện tại, results của tool calls vừa thực hiện, intermediate reasoning steps.

Working memory có *giới hạn dung lượng* — giống như con người không thể giữ quá 7±2 items trong đầu cùng lúc. Khi buffer đầy, items có salience thấp nhất bị đẩy xuống Episodic Network. Cơ chế này không phải limitation — nó là *feature* giúp agent focus vào những gì quan trọng.

#### 2. Episodic Network

Lưu trữ *sự kiện* — những gì đã xảy ra, khi nào, trong context nào. Đây là temporal memory: "buổi sáng hôm qua user hỏi về dự án X," "ba ngày trước có lỗi Y xảy ra trong pipeline."

Episodic memory có temporal index — truy vấn theo thời gian là first-class operation, không phải afterthought. Nó cũng có *temporal decay* — ký ức cũ fade dần nếu không được accessed, giống như con người.

Persistence của Episodic Network được backed bởi `cognition-storage` (memvault) khi cần — đảm bảo ký ức survive qua nhiều sessions.

#### 3. Semantic Network

Lưu trữ *kiến thức có cấu trúc* — facts, entities, relationships. Không phải "Alice nói gì hôm qua" (episodic) mà là "Alice là Project Manager tại Acme Corp, responsible cho dự án X, làm việc với Bob."

Semantic Network là một Knowledge Graph: nodes là entities, edges là relations. Điều này cho phép truy vấn mà vector store không thể làm: "Tất cả người liên quan đến dự án X trong vòng 2 bước relationship là ai?" — đây là graph traversal, không phải similarity search.

Semantic Network là integration point của `kgent` — toàn bộ KG engine từ kgent được dùng ở đây.

#### 4. Habit Network (Procedural Memory)

Nền tảng lý thuyết: Squire & Zola-Morgan (1991) — basal ganglia S-R (Stimulus-Response) circuit.

Habit Network lưu trữ *patterns đã được học*: "khi user hỏi về code errors, thường cần gọi tool search trước khi trả lời," "khi conversation kéo dài hơn 20 turns, consolidate working memory." Đây không phải explicit rules — đây là patterns được extract tự động từ experience.

Điểm khác biệt với Episodic: Episodic nhớ *event cụ thể* ("lần đó tôi làm X và kết quả là Y"), Habit nhớ *pattern chung* ("tình huống loại này thường đòi hỏi action loại kia"). Đây chính xác là sự phân biệt giữa declarative và procedural memory trong neuroscience.

#### 5. Action-Effect Network

Nền tảng lý thuyết: Hommel et al. (2001) Theory of Event Coding; Dickinson & Balleine (1994) Action-Outcome (A-O) dissociation.

Action-Effect Network lưu trữ *causal models*: "action A trong context C dẫn đến outcome O với probability P." Đây là nền tảng cho planning và counterfactual reasoning — agent không chỉ nhớ *đã làm gì* mà còn nhớ *tại sao* và *kết quả như thế nào*.

Điểm này khác Habit Network ở chỗ: Habit là S-R (khi thấy S, làm R — không cần hiểu tại sao), còn Action-Effect là A-O có goal-mediated — agent hiểu relationship nhân quả và có thể reason về nó.

#### 6. Prospective Memory

Lưu trữ *intentions về tương lai*: "nhớ remind user về deadline vào thứ Sáu," "check lại task X sau 2 giờ nếu chưa có response."

Prospective Memory là một feature đặc biệt mà hầu hết agent frameworks bỏ qua hoàn toàn. Một agent có prospective memory có thể *tự chủ động* làm điều gì đó mà không cần được nhắc — đây là một bước quan trọng về phía agents thực sự autonomous.

Agent runtime có một timer/scheduler loop check Prospective Memory và trigger actions khi đúng condition.

### Raw State Snippets — Contribution Mới

Ngoài sáu networks trên, CogMem thêm một mechanism mới không có trong các cognitive science models truyền thống: **Raw State Snippets**.

Raw State Snippets là captures verbatim của trạng thái hệ thống tại một thời điểm — không được processed, không được abstracted. Khi agent cần debug tại sao nó đã làm một quyết định nào đó, nó có thể access lại exact state vào thời điểm đó.

Đây là contribution thực tiễn quan trọng: trong AI systems, explainability thường bị sacrifice để có performance. Raw State Snippets giải quyết điều đó mà không làm chậm normal operation.

### Consolidation Cycle

Khác với các frameworks khác, CogMem có một *consolidation cycle* chạy nền — tương đương với giấc ngủ của con người về mặt chức năng:

1. Working Memory items có salience thấp được transferred sang Episodic Network
2. Patterns lặp lại trong Episodic được extracted và strengthened trong Habit Network
3. Causal relationships được updated trong Action-Effect Network dựa trên recent outcomes
4. Stale items trong Episodic decay theo temporal decay policy

Consolidation chạy asynchronously — không interrupt agent's normal operation.

### Tại Sao CogMem Là Default, Không Phải Bắt Buộc

CogMem là default vì nó là memory architecture tốt nhất cho general-purpose agents. Nhưng không phải mọi use case đều cần full CogMem:

- Một agent chỉ cần answer questions từ documents — SimpleMemory + vector search là đủ
- Một agent cho edge devices với memory constraints — lightweight custom backend
- Một researcher muốn test một memory architecture mới — implement MemoryBackend trait

Cognition không áp đặt. CogMem là lựa chọn tốt nhất có sẵn ngay từ đầu — không phải lựa chọn duy nhất.

---

## VI. Rust Concepts — Tại Sao Mỗi Tính Năng Cần Rust

Đây không phải là document dạy Rust. Nhưng một câu hỏi quan trọng cần trả lời: tại sao từng tính năng của Cognition *cần* Rust, không chỉ *được hưởng lợi từ* Rust?

### Memory Safety không có Runtime Cost

CogMem có nhiều components chia sẻ data: Semantic Network được accessed bởi cả Working Memory consolidation và Skill execution cùng lúc. Trong Python, điều này yêu cầu locks cẩn thận và vẫn có thể crash runtime. Trong Rust, ownership rules đảm bảo điều này là impossible tại compile time — không phải tại runtime khi user đang dùng.

### True Parallelism cho Agent EventLoop

Agent EventLoop cần xử lý đồng thời: nhận messages mới, run skills, stream LLM tokens, run consolidation cycle, trigger prospective memories. Với Python GIL, đây là concurrency giả — chúng thay nhau chạy. Với Rust Tokio, đây là true async parallelism, tận dụng mọi CPU core.

### Zero-Cost Skill Abstraction

`Skill` trait với generic types compile xuống direct function calls — không có vtable lookup trong hot path. Khi một skill được invoked hàng nghìn lần trong một session, sự khác biệt này tích lũy thành con số benchmark đáng kể.

### WASM Safety

Sandboxing WASM plugins yêu cầu careful memory management — bạn cần biết chính xác bytes nào được chia sẻ giữa host và guest. Rust ownership model làm cho điều này safe và verifiable, thay vì dựa vào convention và careful coding.

---

## VII. So Sánh với Ecosystem Hiện Tại

### vs. LangChain / LlamaIndex

LangChain và LlamaIndex là Python-first, không có path nào dẫn đến true parallelism hay predictable latency. Memory là pluggable nhưng không có cognitively-grounded default. Skills (tools) không type-safe. Không có multi-protocol communication.

Cognition không cố thay thế LangChain cho rapid prototyping — LangChain vẫn tốt cho điều đó. Cognition là lựa chọn khi bạn cần *production-grade* agents.

### vs. AutoGen / CrewAI

AutoGen và CrewAI focus vào multi-agent orchestration. Memory vẫn là conversation buffer. Không có cognitively-grounded memory. Communication là HTTP-based, không có WebSocket hay gRPC native.

Cognition có multi-agent support nhưng không là primary focus — nó là consequence tự nhiên của Communication layer tốt.

### vs. Rig (Rust agent framework)

Rig là Rust agent framework đang nổi lên. Nó tốt cho simple LLM applications nhưng không có cognitively-grounded memory, không có MCP support, không có multi-protocol communication. Cognition và Rig cùng space nhưng khác ambition.

### Positioning

```
                    Cognitive Depth
                          ▲
                          │
                    HIGH  │  Cognition ◆
                          │
                          │
                     LOW  │  LangChain ◆  AutoGen ◆  Rig ◆
                          │
                          └──────────────────────────────▶
                              LOW          HIGH
                                     Performance
```

Cognition là framework duy nhất ở góc phần tư cao-cao: cognitively grounded *và* high performance.

---

## VIII. Kế Hoạch Phát Triển

### Workspace Structure

```
cognition/
├── crates/
│   ├── cognition-core/       # traits, types — zero deps
│   ├── cognition-memory/     # CogMem + SimpleMemory
│   ├── cognition-storage/    # persistent store (memvault)
│   ├── cognition-skills/     # Tool, MCP, WASM skills
│   ├── cognition-llm/        # LLM backends
│   ├── cognition-comm/       # WebSocket, gRPC, MCP server
│   ├── cognition-runtime/    # agent event loop, builder
│   └── cognition-py/         # PyO3 Python bindings
├── examples/
└── benches/
```

### Roadmap — 16 Tuần

**Phase 1 — Foundation (Tuần 1–2)**
`cognition-core`: định nghĩa tất cả traits. Đây là phase quan trọng nhất — decisions ở đây ảnh hưởng mọi thứ sau. Không có implementation, chỉ có contracts.

**Phase 2 — Memory (Tuần 3–6)**
`cognition-storage` trước (persistent WAL + BTree temporal index), sau đó `cognition-memory` implement CogMem 6 networks trên top của storage. SimpleMemory implement song song để có gì đó test.

**Phase 3 — Skills (Tuần 7–8)**
`cognition-skills`: Tool skill, MCP client integration, WASM sandbox, SkillRegistry, `define_skill!` macro.

**Phase 4 — LLM (Tuần 9–10)**
`cognition-llm`: Anthropic và Ollama trước (cover cả cloud và local), sau đó OpenAI, Google, OpenRouter. Streaming là priority.

**Phase 5 — Communication (Tuần 11–12)**
`cognition-comm`: WebSocket trước, sau đó gRPC, sau đó MCP server exposure.

**Phase 6 — Runtime (Tuần 13–14)**
`cognition-runtime`: AgentBuilder, EventLoop, consolidation scheduler, prospective memory timer. Đây là lúc mọi thứ kết hợp lại lần đầu tiên.

**Phase 7 — Python + Benchmark (Tuần 15–16)**
`cognition-py`: PyO3 bindings cho Python API. Sau đó benchmark suite so sánh với LangChain và AutoGen trên các scenarios cụ thể.

### Feature Flags

```toml
[features]
default   = ["cogmem"]
cogmem    = ["cognition-storage"]   # CogMem với persistence
simple    = []                      # SimpleMemory, minimal deps
full      = ["cogmem", "grpc", "wasm-skills", "python"]
grpc      = ["tonic"]
wasm-skills = ["wasmtime"]
python    = ["pyo3"]
```

Người dùng chỉ compile những gì họ cần. Binary size tối thiểu khi chỉ cần core functionality.

---

## IX. Tầm Nhìn Xa Hơn

### Cognition như một Research Platform

Framework này không chỉ là engineering project — nó là platform để test các cognitive architecture. Bất kỳ ai muốn implement và compare memory architectures (CogMem vs. ACT-R vs. SOAR-inspired) có thể làm trong cùng một framework, với cùng infrastructure, đảm bảo fair comparison.

### Multi-Agent Society

Khi nhiều Cognition agents giao tiếp với nhau qua Communication layer, chúng ta có thể bắt đầu khám phá *emergent behavior* trong agent societies — điều mà các Python frameworks không thể làm ở scale do performance constraints.

### Edge Deployment

Một Cognition agent với SimpleMemory backend và local Ollama — không cần internet, không cần cloud, chạy trên laptop hay Raspberry Pi. Đây là tương lai của AI deployment mà các Python frameworks không thể đến được vì dependency overhead và memory usage.

---

## X. Kết Luận

Cognition không được xây dựng để thay thế LangChain cho những người đang happy với Python prototyping. Nó được xây dựng cho những người đang hỏi câu hỏi khác:

*Làm thế nào để build agents không chỉ gọi tools, mà thực sự nhớ, học, lên kế hoạch, và giao tiếp — như những thực thể có trí tuệ thực sự?*

Câu trả lời bắt đầu từ khoa học nhận thức — CogMem cung cấp blueprint. Và nó được hiện thực hóa trong Rust — ngôn ngữ duy nhất có thể deliver cả safety, performance, và expressiveness cần thiết để làm điều đó đúng cách.

Đây là framework mà chúng ta muốn tồn tại. Vì vậy chúng ta sẽ build nó.

---

*Cognition — v1.0 Design Document*
*Lê Minh Triết, Tháng 3 2026*

---

## Phụ Lục: Glossary

| Thuật ngữ | Định nghĩa trong Cognition |
|---|---|
| **Agent** | Một entity autonomous có memory, skills, và khả năng giao tiếp |
| **Skill** | Một đơn vị hành động có input/output types rõ ràng và manifest cho LLM |
| **MemoryBackend** | Interface cho bất kỳ memory architecture nào (CogMem, Simple, custom) |
| **CogMem** | Cognitively-grounded memory architecture với 6 specialized networks |
| **Working Memory** | Short-term, attention-gated buffer — những gì agent đang focus |
| **Episodic Network** | Temporal event store — những gì đã xảy ra và khi nào |
| **Semantic Network** | Knowledge Graph — entities và relationships |
| **Habit Network** | Procedural patterns — S-R associations learned from experience |
| **Action-Effect Network** | Causal models — A-O relationships cho planning |
| **Prospective Memory** | Future intentions — những gì agent cần làm sau này |
| **Raw State Snippets** | Verbatim system state captures cho explainability |
| **Consolidation** | Background process transfer và strengthen memories |
| **Transport** | Protocol-agnostic communication interface (WS, gRPC, HTTP) |
| **MCP** | Model Context Protocol — standard cho tool/agent interoperability |
| **WASM Sandbox** | Isolated execution environment cho untrusted skills |
| **CommHub** | Central message router hỗ trợ nhiều transports đồng thời |