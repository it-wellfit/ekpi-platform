# OEKAP 自由 SOTA 前沿評估 v1.0

> 文件編號：OEKAP-SOTA-001｜日期：2026-11-12｜狀態：Active（評估；採納決定見各表）
> 目的：**窮盡**至今先進技術與先進理論之評估（限「自由」＝免費／開放／可本地決定性執行），逐項決定 採納（ADOPTED）／延後（LATER）／觀察（WATCH）／拒絕（REJECTED）／未知（UNKNOWN）。
> 方法：一手文獻＋獨立第三方基準（2024–2026）；「官方宣稱≠獨立證據」；無法查證者標 UNKNOWN（不猜測）。
> 關聯：能力矩陣見 [OEKAP 脈絡地圖](oekap-context-map_v1.0.md)；實作 `platform/src/{ranking,conformance}.rs`。

## 0. 評估準則（為何有些 SOTA 不採）

1. **免費與本地**：無付費 API、無授權費、離線可跑；拒絕「需雲端強模型」之步驟（與 zero-paid-ai 原則一致）。
2. **決定性**：同輸入同輸出；統計模型僅在可固定權重/種子時考慮。
3. **證據**：須有獨立基準；單一供應商宣稱不採。
4. **Lean**：依賴成本與延遲須與增益相稱（見 reranker REJECTED 之證據）。

## 1. 檢索層（E）

| 技術 | 理論來源 | 獨立證據（2024–2026） | 決定 |
|---|---|---|---|
| **BM25** | Robertson & Zaragoza 2009 | TREC 2025 RAG track 前段班管線普遍含 BM25；2026 財務 text-and-table 基準：**BM25 在多數指標勝過當代 dense**（text-embedding-3-large） | **ADOPTED**（`ranking.rs`，library） |
| **BM25+（delta）** | Lv & Zhai 2011 | 長文件偏差修正；實作成本≈0 | **ADOPTED**（同模組，delta 參數） |
| **RRF 融合（k=60）** | Cormack et al. 2009 | 2025 復現（BEIR SciFact）：RRF hybrid nDCG@10 **+0.049**（95% CI [0.028, 0.069]）優於 BM25；參數少、穩健 | **ADOPTED**（`ranking.rs`，library） |
| Weighted fusion | — | 可略優於 RRF 但需調 α（同基準 0.711 vs 0.685） | LATER（需調參協定） |
| **Cross-encoder reranker** | — | **兩份 2025 獨立基準：無增益或反降質**（nDCG@5 −11.5%；paired −0.008 不顯著），延遲 +2–15 s/query | **REJECTED（現階段）** |
| 本地 dense embedding（BGE-M3 等） | Chen et al. 2024 (BGE M3) | hybrid 增益主要來自排名多樣性；dense 本身無一致優勢 | LATER（需模型檔＋記憶體評估，破壞 lean） |
| HyDE／multi-query 擴展 | 2026 arXiv 2604.01733 | 對**精確數值/條款查詢**增益有限 | NOT ADOPTED（非必要） |
| Contextual retrieval | 同上 | 一致增益但需生成式模型 | LATER（受免費本地限制） |

## 2. 流程層（P）

| 技術 | 來源 | 現況與證據 | 決定 |
|---|---|---|---|
| Token-replay fitness/precision | Rozinat & van der Aalst 2008 | 已實作（`conformance.rs`，exact rational，12 測試） | **ADOPTED（R23）** |
| **Alignment-based conformance** | IEEE 2024：「alignment 已成為 SOTA」 | 最優診斷（missing/remaining 歸因）；A* 成本高 | LATER（P1；限小模型＋複雜度上限） |
| **OCEL 2.0／object-centric** | van der Aalst CEUR 3730 (2024)；OCEL 2.0 arXiv 2403.01975 | 學界前沿；多物件互動（deficiency/convergence/divergence）；工具生態成長中 | **WATCH**（對映 X3 事件源） |
| Object-centric alignments（CAiSE 2024）／graph-based OCCC（2025） | 同上 | 前沿研究 | WATCH |

## 3. 規則層（R）

| 技術 | 來源 | 決定 |
|---|---|---|
| **規則衝突偵測（參照完整性＋宣告一致性）** | 傳統 expert systems；SAT/SMT 文獻 | 已實作（`rule_conflicts.rs`；RC-01..09 保守結構檢查；真實語料守衛測試） | **ADOPTED（library）** |
| 條件樹可滿足性推理（SAT/SMT full subsumption） | SAT/SMT 文獻 | 成本高於現階段需求；先以結構等值保守檢查 | LATER |
| LegalRuleML 1.0 匯出 | OASIS | ADOPTED（既有 CTR-EXP-001） |

## 4. 治理／評測層（G）

| 技術 | 來源 | 決定 |
|---|---|---|
| Property-based testing | 既有（property 測試組） | ADOPTED（既有） |
| **Mutation testing（cargo-mutants）** | 業界標準；本地免費 | LATER（夜間批次；`mutants.out` 已見於 artifacts） |
| Metamorphic testing | Chen et al. survey | CANDIDATE（搭配硬閘） |

## 5. 數學與理論維度

| 技術 | 適用性 | 決定 |
|---|---|---|
| **精確有理數**（gcd 約分） | 已用於 conformance；分數/門檻用 f64 但僅排序比較 | ADOPTED |
| 形式化方法（TLA+/Alloy） | `formal_verification.rs` 為 stub；成本高收益局部 | LATER（限關鍵不變量清單） |
| Lyapunov／Pareto／控制理論 | 本平台無動態閉環載體；Pareto 思維已用於多目標取捨（證據矩陣） | UNKNOWN（無直接載體，標註不強行套用） |
| 因果推論 | `causal.rs` library-only | LATER（需資料與識別策略） |

## 6. 來源清單（節錄）

- Cormack, Clarke & Buettcher (2009), *Reciprocal Rank Fusion outperforms Condorcet…*, SIGIR.
- Robertson & Zaragoza (2009), *The Probabilistic Relevance Framework: BM25 and Beyond*.
- Lv & Zhai (2011), *Lower-Bounding Term Frequency Normalization* (BM25+), CIKM.
- NIST TREC 2025 RAG Track（2025-08-16 提交群；BM25+RRF+小型開源 reranker 為多數前段班結構）。
- 2025 BEIR SciFact 復現基準（RRF vs rerank paired bootstrap：hybrid +0.049 [0.028,0.069]；rerank −0.008 不顯著）。
- arXiv 2604.01733 (2026), *From BM25 to Corrective RAG*（財務 text-and-table：BM25 勝 dense；hybrid RRF 最佳）。
- IEEE 10580902 (2024), *Quantifying Conformance Between Object-Centric Event Logs and Models*（alignment 為 SOTA）。
- van der Aalst, CEUR-WS Vol-3730 (2024), *Lifting Process Discovery and Conformance Checking*（OCPM 框架）。
- OCEL 2.0 specification, arXiv 2403.01975 (2024)。

## 7. 採納與驗收（本版）

- **本版採納**：BM25／BM25+／RRF（`platform/src/ranking.rs`，7 測試，library-only）。
- **接入服務路徑前置**：28 案固定集門檻重校準＋雙跑一致性證據＋register 登錄（不得直接切換，避免門檻漂移）。
- **依證據拒絕**：cross-encoder reranker（現階段）。

## 8. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-12 | 初版：E/P/R/G/數學五區評估；BM25/BM25+/RRF 採納；reranker 拒絕；alignment/OCEL/mutation 等延後與觀察 |
