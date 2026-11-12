# OEKAP 脈絡地圖與名詞表 v1.0

> 文件編號：OEKAP-CONTEXT-001｜版本：v1.0｜日期：2026-11-12｜狀態：Active（設計／盤點，非驗證）
> 擁有者：Product＋Engineering｜審閱：架構／QA（獨立檢核）
> 目的：為「企業知識與流程智慧平台（OEKAP）」建立**單一脈絡與名詞 SSOT**，供後續設計、盤點與接線契約引用。
> 本文件**不改變**任何 Phase 判定：Phase A **未核准**、Phase B **阻塞**（見 [phase gates](../quality/phase-gates.md)）。
> 證據基線（實測，不得放大）：`ekpi-platform` workspace `cargo test --locked` **534 passed / 0 failed**（R21，2026-11-11，canonical exit 0）；67 評測案；28 檢索案（20 答／8 拒）；19 active＋1 draft 規則；7 文件／10 版本；KEM 20 claims（confidence 0.79）。

---

## 1. 主題定義（一句話）

把「**文件版本＋條款錨點＋生效規則＋流程證據＋審批紀錄**」綁成**一條可重播、可稽核、證據不足即拒答或升級**的決策鏈——這是本平台有別於「文件搜尋」或「流程圖工具」的核心。

## 2. 脈絡地圖（資產與邊界）

| 資產 | 角色 | 現況（實證） | 權威文件 |
|---|---|---|---|
| `ekpi-platform/` | **判定核心原型**（local deterministic；本 repo） | JSONL hash 鏈帳本；PBKDF2-HMAC-SHA256 本地 session；`api.rs` 未接線佔位；多模組 library-only | [product brief](../product/README.md)、[architecture](../architecture/README.md)、[phase gates](../quality/phase-gates.md) |
| `oekap/` | **目標企業全端**（部署級；獨立 crate 樹） | Rust 2024＋Axum 0.8＋Askama 0.16＋PostgreSQL 18（SQLx 0.9 編譯期、`.sqlx`、`migrations/`）；`apps/web`＋`crates/{domain,application,adapters}`；Zero-JS／CSP `script-src 'none'`／ASVS 5.0／WCAG 2.2 AA | [oekap README](../../../oekap/README.md) |
| `knowledge/`（KEM） | **主張／證據層** | `index.md` 20 條 published claims（Fact／Decision／Invariant；confidence 0.79；schema 1）；`claims/`、`log.md`（103 筆 append-only）、`console/` | [KEM index](../../../knowledge/index.md) |
| `../Knowledge/`（跨專案 KB） | **合規事實權威**（唯讀；ADR-0004 單向引用） | 目錄：`adr/`、`compliance/`、`governance/`、`sources/`、`traceability/` 等；`compliance/` 現僅 `iso-27001/` | [KB index](../../../../Knowledge/README.md)、[knowledge-index](../knowledge-index.md) |
| 支援線：`Wellfit/` | 批次與外部證據產生者 | MTRIX／Shopee／bankpay 冪等帳本＋SHA-256 投入產出（獨立 repo） | [Wellfit AGENTS](../../../Wellfit/AGENTS.md) |
| 支援線：根 `platform/`＋`core/`＋`web/` | FlapsErp RPA 支援（**非本平台**） | Axum 0.7／SQLite／zero-JS | [根 AGENTS](../../../AGENTS.md) |
| 治理 | 閘門與追溯 | Phase A/B 矩陣、traceability register、assurance、C-13 五級判定 | [register](../traceability/register.md) |

> 邊界：`../Knowledge/` 對實作者**唯讀**（ADR-0004）；`ekpi-platform` 與 `oekap` 不得互相比較或互相量測；根 workspace 平台線與本平台無契約關係。

## 3. 名詞表（單一定義；文件一律引用本表）

| 名詞 | 定義 | 指向 |
|---|---|---|
| 文件（document） | 具版本與 SHA-256 的來源投遞；平台只存**參照**不搬內容 | `platform/src/docs.rs` |
| 文件版本（version） | 同一文件的可消費版本；過期／歧義版本 → `insufficient_evidence` | 同上 |
| 條款錨點（clause anchor） | 版本內可解析的定位（條號／段落），引用必帶 | 同上 |
| 規則（rule） | 具 schema、生效狀態（active／draft）與優先序的判準；draft 不可消費 | `platform/src/rules.rs`、`design/rule-schema-v0.1.1.schema.json` |
| 主張（claim） | KEM 的知識單元（Fact／Decision／Invariant）＋confidence | [KEM index](../../../knowledge/index.md) |
| 證據（evidence） | 可回溯到來源錨點的片段；不足／衝突／未登錄 → 拒答或升級 | `platform/src/evidence.rs`、`retrieval.rs` |
| 流程模型（process model） | 活動、依賴、禁止配對、期限、迴圈的宣告 | `platform/src/process.rs`、`platform/data/processes.json` |
| 流程事件（process event） | 實際發生的活動序列（可來自外部批次） | 同上（`xes_import` 為 library-only） |
| 快照（snapshot） | 決定性、可重播的判定結果＋引用證據 | `platform/src/snapshot.rs` |
| 判定（verdict） | `comply`／`violate`／`insufficient_evidence`／`escalate` 之一 | `platform/src/engine.rs` |

## 4. 六面能力矩陣（盤點）

| 面 | 現況（模組／證據） | 目標能力 | 缺口（誠實標註） | 驗證方式 |
|---|---|---|---|---|
| **K 知識層** | 7 文件／10 版本＋條款錨點＋SHA-256；KEM 20 claims | 版本→條款→主張→證據可解析鏈 | `knowledge_graph`／`ocr`／`multimodal` **library-only**；SoA／93 控制**未定位（UNKNOWN）**；KEM 與判定層未接線 | 錨點解析測試；claim→citation 正／負例 |
| **R 規則層** | 19 active＋1 draft；schema fail-closed；LegalRuleML 匯出 | 規則版本化、衝突偵測、生效／失效邊界 | 規則變更遷移與衝突檢測未實作 | `--declared-rules-schema`；draft 不可消費測試 |
| **P 流程層** | replay：禁止配對／期限／迴圈；`violate`／`insufficient_evidence`／`escalate`；**conformance 指標（fitness／precision，exact rational）已實作** | BPMN 2.0／OCEL 匯入＋conformance（fitness／precision） | `bpmn`／`xes_import` **library-only**；`conformance.rs` 為 library-only（未接 CLI／報告）；BPMN 往返測試待建 | 種子＋邊界案例；BPMN 往返測試（待建）；`cargo test -p ekp_platform conformance`（8 測試） |
| **E 證據層** | lexical retrieval（threshold 0.42；28 案；4 角色案）；引用或拒答；hash 鏈帳本；**BM25／BM25+／RRF 已實作**（`ranking.rs`，library-only；[SOTA 附錄](oekap-sota-frontier_v1.0.md)：reranker 依獨立基準證據**拒絕**） | 混合檢索（BM25＋現行 cosine 經 RRF 融合）＋角色／租戶邊界 | `ranking.rs` 未接線（接線需 28 案**門檻重校準**）；`retrieval_v2`／`fulltext`／`vector_search` 未接線；角色為 library 證據非強制 | `--eval` 硬閘 H-1..H-6；28 案不退化；`cargo test -p ekp_platform ranking`（7 測試） |
| **D 決策層** | 決定性快照＋重播；decisions export v2；審批工作流（7 態稽核、冪等、PG-A 已實作） | 跨資產決策鏈：claim→evidence→decision→approval 可重播 | 跨資產（KEM↔判定↔批次）未接線 | snapshot 重播一致；export 結構驗證 |
| **G 治理／評測層** | eval harness（67 案；eval 自檢 12；self-test 49；硬閘 H-1..H-6）；traceability；phase gates | 需求→測試→證據→判定全鏈可查 | A1 真實試點未做；B 系（B1/B4/B5/B6/B8）需外部決策 | register 完整性；閘門證據標準 |

## 5. 缺口與優先序

| # | 缺口 | 面向 | 阻斷 | 可驗證性 | 優先 |
|---|---|---|---|---|---|
| 1 | 跨資產決策鏈（KEM↔判定↔批次）未接線 | D／K | Phase A 價值證明 | 高（正／負例可測） | P0 |
| 2 | SoA／93 控制位置未定位 | K／G | 合規評測 | 中（需 KB 定位） | P0（先定位） |
| 3 | 規則變更遷移與衝突檢測 | R | 規則治理 | 高 | P1 |
| 4 | 流程 conformance 指標（BPMN／OCEL） | P | 指標已實作（`conformance.rs`，exact rational、8 測試）；import 未實作 | 高（純演算法、無新依賴） | P1 |
| 5 | 混合檢索：BM25/RRF **已實作**（library）；接線需門檻重校準；dense 延後、reranker 已拒絕 | E | 召回品質 | 高（純演算法；28 案為驗收） | P1 |
| 6 | 角色／租戶強制（非 library） | E／D | 企業邊界 | 低（需 IdP；DEC-12） | BLOCKED |
| 7 | 真實試點（A1） | G | Phase A | — | BLOCKED（DEC-01） |
| 8 | 部署／營運／備份（B1/B4/B6/B7/B8） | G | Phase B | — | BLOCKED（DEC-13/14/16/19/20） |

## 6. 接線契約（X1–X5；只定義，不實作）

| ID | 輸入 | 輸出 | 不變量 | 失敗模式 |
|---|---|---|---|---|
| **X1** 原型→`oekap` 對映 | `ekpi-platform` 模組與契約 | `oekap/crates/{domain,application,adapters}`＋`apps/web` | JSONL→PostgreSQL 18；PBKDF2→Argon2id；金額禁浮點；zero-JS | 對映缺項 → 阻斷該切片（不得部分宣稱） |
| **X2** KEM→決策快照 | claim（含 confidence） | 快照內引用（claim id＋錨點） | 無 claim 即拒答；引用必可解析 | claim 失聯／confidence 過低 → `insufficient_evidence` |
| **X3** 批次→流程事件 | Wellfit ledger／產出 | 流程事件流（活動＋時間＋批次 id） | 事件不可靜默改寫；重跑冪等 | 事件缺漏／重複 → `insufficient_evidence` 或拒收 |
| **X4** 控制集→規則／評測 | 控制項（SoA 定位後） | 規則條款＋評測案 | 每控制至少一案；可重跑 | 控制無對應 → 標 UNKNOWN（不得宣稱覆蓋） |
| **X5** 治理閘門→證據標準 | 需求／變更 | 判定欄位（命令／修訂／結果／五級） | PASS 僅限指定修訂版且退出 0；UNKNOWN≠PASS | 證據不足 → NOT VERIFIED |

## 7. 驗證計畫（可重跑；判定欄位＝命令／修訂／結果）

| 能力 | 命令 | 預期 |
|---|---|---|
| 全鏈（canonical） | `ekpi-platform/scripts/check.ps1` | `CHECK_EXITCODE=0`（ALL CHECKS PASSED） |
| 引擎自檢 | `cargo run --bin ekp_platform -- --selftest` | 49 斷言全過 |
| 規則 schema | `cargo run --bin ekp_platform -- --declared-rules-schema` | 20 筆、fail-closed |
| 檢索／拒答 | `cargo run --bin ekp_platform -- --search`；`--eval --test-set data/eval/search_eval.json` | 28 案：20/20、8/8、0 violations |
| 結構匯出 | `cargo run --bin ekp_platform -- --export-legalruleml <dir>` | root/namespace/語句數結構檢查 |
| 文件連結 | `ekpi-platform/scripts/check-docs.ps1` | 通過（required／links） |
| KEM 索引 | `knowledge/index.md` 產生器（重產） | 無 diff（若可重產） |

> 任何未執行或無法執行者標 **UNKNOWN**；不得將 UNKNOWN 寫成 PASS。

## 8. 邊界與非目標

- 不做：Phase A/B 決策、部署、IdP／SSO／MFA、租戶隔離、AI 模型導入、連接器。
- 不宣稱：production-ready／compliant／certified／validated。
- `api.rs` 永遠不是契約；library-only 模組不得被稱為能力。

## 9. 修訂紀錄

| 版本 | 日期 | 變更 |
|---|---|---|
| v1.0 | 2026-11-12 | 初版：脈絡地圖、名詞表（10）、六面矩陣、缺口優先序、X1–X5 契約、驗證計畫 |