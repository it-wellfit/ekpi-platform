# ISO/SOP/CAPA/內控/稽核系統技術研究框架

> 截至 2026 年 10 月 25 日可核實的公開資料為限。
> 本框架不宣稱已遍歷所有研究；所有技術主張均附來源與驗證狀態。

## 1. 問題重新定義

### 1.1 核心問題

本框架回答的不是「如何讓 LLM 更準確地回答文件問題」，而是：

> **如何建立一套可重現、可稽核、可替換的企業知識與流程智慧平台，使得每個稽核結論都能追溯至原始文件條款、版本與事件紀錄，且在證據不足時正確拒答？**

### 1.2 問題分解

| 子問題 | 說明 | 對應層 |
|--------|------|--------|
| Q1 | 如何從非結構化/半結構化文件中還原可機器查詢的結構、條款、版本與座標？ | Layer 1 |
| Q2 | 如何表示跨文件的規範要求、流程模型、事件紀錄與證據主張？ | Layer 2 |
| Q3 | 如何在不同任務類型下選擇最佳檢索策略，並保留原文引用？ | Layer 3 |
| Q4 | 如何將規範要求轉化為可機器檢查的規則，並區分規則符合、流程順序與因果？ | Layer 4 |
| Q5 | 如何驗證實際作業是否符合流程模型，輸出可追溯的合規判定？ | Layer 5 |
| Q6 | 如何管理證據鏈、版本固定、不確定性與人工覆核？ | Layer 6 |

### 1.3 關鍵區分（A11 核心原則）

| 關係類型 | 定義 | 負責模組 | 驗證方式 |
|----------|------|----------|----------|
| **規範依賴** (Normative Dependency) | 必須先取得核准才能執行 | 規則引擎 | 規則匹配 + 前置條件檢查 |
| **流程順序** (Process Order) | A 活動完成後才允許 B 開始 | 流程驗證器 | Token replay / conformance checking |
| **真實因果** (True Causation) | 某項控制措施是否真的降低了缺失發生率 | 因果分析模組 | 因果圖 + 識別策略 + 混淆因素分析 |

**禁止**將三者混為一談。順序 ≠ 規則符合 ≠ 因果。

---

## 2. 六層技術架構

### Layer 1: 文件感知與結構還原

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| 版面感知解析 (Docling/MinerU 類) | PDF → 結構化 Markdown/JSON；保留表格、圖表、公式 | OCR 錯誤率；複雜版面仍不穩定 | 成熟（開源） |
| PageIndex 樹導覽 | 文件樹 + 推理式導覽；適合長文件精確定位 | 依賴文件已有清晰結構；官方基準成績需獨立重現 | 候選（官方自報） |
| RAPTOR 階層摘要 | 內容聚類生成多粒度摘要樹；跨章節多跳問答 | 摘要可能遺失細節；不保留原文座標 | 研究（ICLR 2024） |
| Psi-RAG 跨文件樹 | 跨文件多跳 QA；F1 較 RAPTOR +25.9% | 自報數字；對順序敏感查詢支援度待確認 | 研究（ICML 2026） |
| SERAG 結構熵樹 | 多視角圖 + 結構熵最小化 | 對條件分支支援度待確認 | 研究（ICML 2026） |
| 長上下文 LLM | 全文比較、版本差異分析 | 成本高；不可作為稽核證據（無座標） | 成熟（工程） |
| OCR + 座標 | 掃描文件數位化；保留 bbox | OCR 錯誤率 1-5%；需人工校正 | 成熟 |

**MVP 選擇**：版面感知解析 + 文件樹 + 條款錨點。PageIndex 作為可插拔後端（R3）。

### Layer 2: 知識表示與索引

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| 向量索引 (pgvector) | 語意相似度檢索 | 無法處理精確匹配；版本/權限過濾需額外 SQL | 成熟 |
| 全文檢索 (Tantivy/FTS) | 關鍵字精確匹配；BM25 排序 | 對同義詞/語意變體不敏感 | 成熟 |
| 知識圖譜 (Typed KG) | 跨文件關係查詢；多跳推理 | LLM 抽取的關係不可自動視為事實 | 成熟（建模）/ 研究（自動抽取） |
| GraphRAG (MS) | 跨文件多跳查詢；社群摘要 | 摘要層輸出不得作為稽核證據；抽取成本高 | 候選（repo 存在） |
| GraphRAG-Bench | 決定何時用圖的證據基礎 | 非稽核場景；可借用資料集 | 成熟（基準） |
| 文件樹 (PageIndex) | 長文件精確定位 | 依賴文件已有清晰結構 | 候選 |

**MVP 選擇**：pgvector + Postgres FTS + 文件樹。KG 以受核准的關係表存放。

### Layer 3: 檢索與證據組合

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| 混合檢索 (Hybrid) | 通用檢索；平衡召回與精確 | RRF 參數需調校 | 成熟 |
| Reranker (Cross-encoder) | 提升 Top-k 精確度 | 延遲增加 2-5x | 成熟（工程） |
| Self-RAG | 自我反思式檢索 | 反思次數 ≠ 正確性 | 研究（ACL 2024） |
| Corrective RAG | 檢索品質評估 + 補充檢索 | 評估器可能出錯 | 研究（arXiv 2024） |
| Adaptive RAG | 依查詢複雜度選擇策略 | 複雜度分類器需訓練 | 研究（arXiv 2024） |
| Agentic RAG | 多步推理 + 工具調用 | 無限制對話風險；需停止條件 | 研究（2024-2025） |
| Parent-Child Retrieval | 子塊檢索 + 父塊上下文 | 父塊可能過長 | 成熟（工程模式） |

**MVP 選擇**：混合檢索（bigram cosine → pgvector + FTS）+ 樹導覽 + snippet citations。

### Layer 4: 推理與規則驗證

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| JSON Schema 規則 | 結構化規則定義 | 表達能力有限 | 成熟 |
| LegalRuleML | 法規/標準的機器可讀表示 | 學習曲線；工具生態有限 | 標準（OASIS 2021） |
| DMN | 決策表；條件分支 | 複雜邏輯需 FEEL | 成熟（OMG） |
| Datalog/Prolog | 規則推理；遞迴查詢 | 效能；除錯困難 | 成熟 |
| 神經符號 AI | LLM 理解 + 符號推理 | LLM 輸出不確定；需驗證層 | 研究（2024-2025） |
| CausalGraph2LLM | 因果圖 + LLM 查詢生成 | 因果識別需足夠資料 | 研究（2025） |

**MVP 選擇**：JSON Schema v0.1 + LegalRuleML 匯出。因果模組獨立（A11）。

### Layer 5: 流程執行與合規驗證

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| BPMN 2.0 | 業務流程建模 | 複雜流程難以維護 | 成熟（OMG） |
| Petri Net | 形式化流程模型；token replay | 模型爆炸問題 | 成熟 |
| 狀態機 (FSM) | 簡單狀態轉移 | 無法處理平行/複雜同步 | 成熟 |
| Process Mining (PM4Py) | 從事件日誌發現流程模型 | 依賴日誌品質 | 成熟（PM4Py） |
| XES/OCEL | 事件日誌標準格式 | 資料品質問題 | 標準（IEEE 1849-2023） |
| 形式化方法 (TLA+/Alloy) | 關鍵流程的數學驗證 | 學習曲線極高 | 成熟（學術） |
| Model Checking | 狀態空間窮舉驗證 | 狀態爆炸 | 成熟（學術） |

**MVP 選擇**：Petri Net token replay（Rust 原生）+ 四分類 conformance。PM4Py sidecar 作 alignments（R3）。

### Layer 6: 證據與治理

| 技術 | 適用情境 | 限制 | 成熟度 |
|------|----------|------|--------|
| Decision Snapshot (manifest hash) | 決策重放；byte-identical 驗證 | 需所有輸入固定 | 設計即建（本專案） |
| WORM 儲存 | 證據不可篡改 | 儲存成本；需 hash chain | 成熟（工程模式） |
| PROV-O | 來源追溯描述 | 複雜度；工具生態有限 | W3C 標準 |
| NIST AI RMF | AI 治理框架 | 非技術標準 | 成熟（框架） |
| OWASP LLM Top 10 | LLM 安全防護 | 針對 LLM 應用 | 成熟（指南） |

**MVP 選擇**：Decision snapshot + manifest hash + WORM hash chain。

---

## 3. 技術比較矩陣

### 檢索技術比較

| 技術 | Recall@k | Precision@k | 延遲 | 成本 | 引用正確率 | 版本感知 | 適合稽核 |
|------|----------|-------------|------|------|-----------|----------|----------|
| 傳統 RAG (向量) | 中 | 中 | 低 | 低 | 低（無座標） | 否 | 否 |
| 混合檢索 (向量+BM25) | 高 | 中高 | 中 | 低 | 中 | 部分 | 部分 |
| PageIndex 樹導覽 | 高（長文件） | 高 | 中 | 中 | 高 | 是 | 是 |
| GraphRAG | 高（多跳） | 中 | 高 | 高 | 中 | 部分 | 部分 |
| RAPTOR | 中高 | 中 | 中 | 中 | 低（摘要無座標） | 否 | 否 |
| Agentic RAG | 高 | 高 | 高 | 高 | 高 | 是 | 是（需驗證） |

### 流程驗證技術比較

| 技術 | Conformance Precision | Conformance Recall | 延遲 | 可解釋性 | 適合稽核 |
|------|---------------------|-------------------|------|----------|----------|
| Token Replay (Petri Net) | 高 | 中 | 低 | 高 | 是 |
| PM4Py Alignments | 高 | 高 | 中 | 高 | 是 |
| BPMN Simulation | 中 | 中 | 中 | 中 | 部分 |
| Model Checking | 極高 | 極高 | 高（狀態爆炸） | 極高 | 是（小型模型） |

### 因果推論技術比較

| 技術 | 因果識別能力 | 資料需求 | 可解釋性 | 適合稽核 |
|------|-------------|----------|----------|----------|
| 規範依賴（規則引擎） | 確定性 | 低 | 極高 | 是 |
| 流程順序（token replay） | 確定性 | 低 | 極高 | 是 |
| 時間序列因果 (Granger) | 統計關聯 | 中 | 中 | 部分 |
| 因果圖 (DAG) | 識別（需假設） | 高 | 高 | 是（需專家） |
| 反事實推理 | 識別（需模型） | 極高 | 中 | 部分 |
| CausalGraph2LLM | 生成式因果查詢 | 高 | 中 | 研究 |

---

## 4. 稽核專用評測集

### 評測類別（15 類）

| # | 類別 | 測試案例數 | 指標 |
|---|------|-----------|------|
| 1 | 精確條款檢索 | 5 | Recall@5, Precision@5, MRR |
| 2 | 跨章節/跨文件多跳 | 5 | Recall@5, nDCG@10 |
| 3 | 表格/附錄定位 | 3 | Anchor resolvability |
| 4 | 新舊版本衝突 | 4 | Version selection accuracy |
| 5 | 否定句與例外條款 | 4 | Rule verdict accuracy |
| 6 | 前置條件與順序錯誤 | 4 | Conformance F1 |
| 7 | 合法/非法循環 | 3 | Loop policy accuracy |
| 8 | 缺少證據 | 4 | Abstention accuracy |
| 9 | 相互矛盾的來源 | 3 | Conflict detection rate |
| 10 | 因果 vs 相關 | 3 | Causal identification accuracy |
| 11 | 長流程/平行分支 | 3 | Conformance F1 |
| 12 | 文件解析失敗 | 2 | Parse failure handling |
| 13 | 權限不足 | 2 | Permission violation rate |
| 14 | 惡意文件/Prompt Injection | 3 | Injection resistance rate |
| 15 | 無法判定/應升級 | 2 | Escalation accuracy |

### 評測指標（分開計算）

| 指標 | 定義 | 目標 |
|------|------|------|
| **漏判率** (FNR) | 應判定違規但判定為符合的比例 | < 1% |
| **誤報率** (FPR) | 應判定符合但判定為違規的比例 | < 5% |
| **證據完整度** | 每個結論引用的證據覆蓋率 | ≥ 95% |
| **引用正確率** | 引用能精確定位至文件/章節/頁碼 | 100% |
| **版本錯誤率** | 使用了非當前有效版本的比例 | 0% |
| **延遲** (P95) | 單案件 adjudication P95 | < 500ms |
| **成本** | LLM + 檢索 + 儲存成本 | < $0.01/案件 |
| **人工覆核率** | 需人工介入的案件比例 | < 20% |
| **拒答正確率** | 證據不足時正確拒答的比例 | 100% |
| **因果誤判率** | 將規範依賴/流程順序誤判為真實因果的比例 | 0% |

---

## 5. 可重現的基準架構比較實驗

### 5.1 基線比較（7 個配置）

| 配置 | 描述 | 元件 |
|------|------|------|
| A | 傳統 RAG | 向量檢索 + LLM 生成 |
| B | 混合檢索 + 重排序 | 向量 + BM25 + reranker |
| C | PageIndex 樹狀檢索 | 文件樹導覽 + LLM |
| D | GraphRAG | 知識圖譜 + 社群摘要 |
| E | Agentic RAG | 多步推理 + 工具調用 |
| F | 流程模型 + 規則驗證 | Petri net + 規則引擎（無 LLM） |
| G | 整合式平台 | A-F 所有元件整合 |

### 5.2 消融實驗

逐一移除 G 的元件，測量 Δ 指標。例如：
- G - Reranker：移除 reranker，測量 Precision@k 下降
- G - GraphRAG：移除圖層，測量多跳 Recall 下降
- G - Causal：移除因果模組，測量因果誤判率變化

### 5.3 實驗控制

- **固定**：資料集版本、模型版本、Prompt 版本、索引版本、參數
- **記錄**：每次執行的 commit hash、環境、時間戳
- **重現**：容器化；seed 設定固定
- **統計**：每配置跑 3 次，報告均值 ± 標準差

---

## 6. 技術成熟度分類

### 已成熟的工程方法（可直接採用）

PostgreSQL + pgvector、Tantivy、BPMN 2.0、Petri Net、XES/OCEL、JSON Schema、LegalRuleML、Rust (unsafe forbidden)、axum、Maud。

### 仍需實證的架構（需實驗驗證）

PageIndex 樹導覽、GraphRAG、Reranker、神經符號 AI、CausalGraph2LLM。

### 研究中的前沿技術（不可直接用於合規路徑）

Mojo、RAPTOR/Psi-RAG/SERAG、Self-RAG/Corrective/Adaptive/Agentic RAG、TLA+/Alloy。

---

## 7. 避免混淆：順序、規則符合、因果推論

### 三層分離架構

```
Layer A: 規範依賴 → 規則引擎 → comply/violate/insufficient/escalate
Layer B: 流程順序 → Token Replay → conform/deviate/missing/unconfirmed
Layer C: 真實因果 → 因果分析模組 → identified/cannot_identify/partially/contradicted
```

### 禁止事項

1. 不得將流程順序等同於真實因果
2. 不得將規則符合等同於因果
3. 不得在資料不足以識別因果時生成確定性因果結論
4. 不得將 LLM 的反思次數作為正確性證據

### 證據鏈保存

原始文件（SHA-256）→ 文件版本 → 條款錨點 → 規則（JSON Schema + LegalRuleML）→ 事件（XES/OCEL 三態）→ 決策（snapshot manifest hash）→ 因果假設（假設/資料/識別/混淆/反證）。

### 版本管理

文件版本、規則版本、索引版本、決策版本。每次變更記錄版本號與核准紀錄。

### 不確定性管理

| 不確定性類型 | 處理方式 | 輸出 |
|-------------|----------|------|
| 證據不足 | 停止判定 | `insufficient_evidence` + 缺失清單 |
| 證據衝突 | 輸出 escalate | `escalate` + 衝突清單 |
| 無法識別因果 | 輸出 cannot_identify | `cannot_identify` + 資料限制 |
| 文件解析失敗 | 標記 parse_failure | `parse_failure` + 錯誤詳情 |
| 權限不足 | 拒絕存取 | `permission_denied` |

---

## 8. 參考文獻（已核實，截至 2026-10-25）

完整文獻清單見 `research/papers.csv`（28 篇）與 `research/standards.csv`（11 條）。

| ID | 標題 | 來源 | 年份 |
|----|------|------|------|
| raptor-2024 | RAPTOR: Recursive Abstractive Processing | ICLR 2024 | 2024 |
| pageindex-2025 | PageIndex: Document Index for Vectorless RAG | GitHub | 2025 |
| graphrag-bench-2026 | When to use Graphs in RAG | ICLR 2026 | 2026 |
| microsoft-graphrag-2024 | Microsoft GraphRAG | official doc | 2024 |
| ares-2024 | ARES: Automated Evaluation Framework | NAACL 2024 | 2024 |
| psi-rag-2026 | Hierarchical Abstract Tree for Cross-Doc RAG | ICML 2026 | 2026 |
| serag-2026 | Token-Free Hierarchical Indexing | ICML 2026 | 2026 |
| stronger-baselines-2025 | Stronger Baselines for RAG with Long-Context LLMs | EMNLP 2025 | 2025 |
| ragas-tool-2024 | RAGAS Evaluation Metrics | official doc | 2024 |

---

## 9. 開放問題

1. PM4Py sidecar 是否為硬依賴或可選？
2. 目標部署環境（Windows/Linux/兩者）？
3. 知識圖譜是否作為獨立 crate？
4. 50+ 案例的最低數量？
5. 是否需要多租戶？
6. 是否需要離線 LLM？
