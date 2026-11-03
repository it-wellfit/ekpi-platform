# Round 1 技術選型報告與分期建置計畫

> 範圍：Master Prompt v1 §十三 第 1–8 項（研究盤點 → 分階段開發計畫）+ Addendum A9/A12。
> 所有引用的核實狀態見 `../research/papers.csv` 與 `../research/standards.csv`（2026-10-25）。
> 元件狀態標記：`已核實`＝本 session 實際抓取確認；`待查證`＝實作時須另行驗證後才可引用。

## 0. 堆疊方向變更（2026-10-25 創辦人指示）

- **主軸：Rust 全端。** 核心 crates 預設提案：axum（後端）、SQLx + PostgreSQL（含 pgvector）、
  Tantivy（全文檢索）、Maud（server-rendered HTML 模板）。版本與授權在 R2 開工前逐一登錄
  `research/standards.csv` 後鎖定（現況標記：待鎖定）。
- **執行/驗證核心層原生 Rust**：規則引擎、token-based replay、可達性檢查、snapshot 重放。
  判定路徑不經 GC/不可控執行時期 → 重放確定性更強（A6 加分項）。
- **展示層 CSS-first**：`<script>` 數量 = 0（工作區憲法 §2.3）；現代 CSS（`:has()`、Popover API、
  Anchor Positioning）；知識樹/時間線/引用高亮皆可以 server-rendered HTML + CSS 完成。
- **選配 sidecar：PM4Py**。以「雜湊工件契約」交換（雜湊過的 XES/OCEL 進、四分類報告出），
  僅作 alignments 對照 oracle（R3）；MVP 的 conformance v0 由 Rust 原生 token replay + 可達性支撐硬門檻。
- **Python 降級**：不再是平台主語言（v1 §八 由本節與 addendum A12 修訂）。嵌入/LLM 調用以
  外部服務 API 自 Rust 呼叫 → 平台核心無 Python 執行時期。Python harness 版本移至
  `eval/reference-python/`（非規範性，NOT RUN）。
- **Mojo：前沿方法軌道（research-only）**。本次核實：`modular.com/mojo` 官方頁存在（title "Mojo"，
  含 MAX/GPU）；`mojo.modular.com` DNS 未解析。授權、穩定版本、可重現建置三項未核實前，
  **不得**進入 compliance-critical 路徑；用途限定為未來自建推論/數值核心的候選。

## 1. Round 1 完成盤點（§十三 1、2 項）

| 項目 | 狀態 | 位置 |
| --- | --- | --- |
| 技術領域盤點（A–G 七域） | 完成（v1 §2.2 已列，本輪補齊 LegalRuleML/OCEL/IEEE 1849-2023/PM4Py） | addendum A9 |
| 論文與官方原始碼核實 | 完成：18 篇論文全數核實；11 條標準/工具（10 條 `verified=yes`、Mojo `verified=partial`） | `research/papers.csv`、`research/standards.csv` |
| 需求、風險、場景 | 完成主檔（v1 §一/§三 已定義），本報告 §5 落成驗收門檻 | 本文件 |
| 技術整合矩陣 | 完成 | 本文件 §2 |
| 架構與資料模型 | 完成（v1 §三 五層模型 + 本輪 schema 落地） | `../design/rule-schema-v0.1.md` |
| MVP 範圍 | 完成 | 本文件 §4 |
| 基準與驗收標準 | 完成 | 本文件 §4/§5 |
| 分期開發計畫 | 完成 | 本文件 §6 |

## 2. 技術整合矩陣（§十三 第 4 項）

| 層 | 預設元件 | 狀態 | 備選 | 選型理由 | 對應驗收 |
| --- | --- | --- | --- | --- | --- |
| 文件解析 | 採「已具結構之版本化文件」為 MVP 輸入；版面感知解析（如 Docling/MinerU 類） | 待查證 | — | 解析層錯誤會污染全鏈；MVP 先用人工結構化的 SOP 種子文件 | TB-1 |
| 文件結構層 | 文件樹 + 條款錨點（section/page/bbox）存 Postgres | 已核實（設計即建） | PageIndex 樹導覽 | 支援頁碼定位與引用追蹤；PageIndex 作為可插拔檢索後端 | E-4 |
| 基礎檢索 | pgvector 向量 + Postgres FTS（BM25 近似）+ 樹導覽 | 已核實（成熟組合） | OpenSearch | 零額外基礎設施起步；不改變日後升級 | E-1..E-3 |
| 重排序 | 交叉編碼器 reranker（MVP 先以 BM25 分數融合近似，正式 reranker 延後） | 待查證 | 交叉編碼器模型（如 BGE-reranker 類；登錄 `standards.csv` 後鎖定） | 戰略後排：先讓召回成立；選型需實測延遲/成本/可維護性 | M-2 |
| 圖層 | MS GraphRAG（開源）選配；條款→SOP→表單關係以受核准的 KG 存放 | 已核實（repo 存在） | GraphRAG-Bench 資料集借用 | 「何時用圖」由消融資料決策（B 條款） | E-2 |
| 規則層 | JSON Schema v0.1 + LegalRuleML 匯出 | 已核實（OASIS Standard 2021-08-30） | DMN 表 | deontic 語氣+override+evidence_requirements+review 生命週期 | RB-1..RB-3 |
| 流程層 | BPMN 撰寫 + Rust 原生 token replay／可達性驗證（MVP 核心）；PM4Py sidecar 作 alignments 對照（選配，R3） | 已核實（pm4py.org；MVP 核心設計即建） | — | conformance checking 方法為學界正解；sidecar 以雜湊工件契約交換、不混入判定 | PR-1..PR-4 |
| 事件日誌 | XES（IEEE 1849-2023）為主；多實體採 OCEL 2.0 | 已核實 | 六項品質關卡前方先行 | 學界/產業共通；缺紀錄≠未發生（observed/assumed/unconfirmed 三態） | GR-1..GR-6 |
| 因果層 | 規範依賴→規則引擎；真實因果→獨立模組（假設/識別/混淆/反證宣告） | 已核實（範式） | CausalGraph2LLM 查詢集 | 規範 vs 程序 vs 因果分離（Addendum A11） | CB-1..CB-2 |
| Agent 層 | 依 v1 §七 十個角色；共用 harness 執行 | 設計即建 | LangGraph（狀態化 Agent 工作流，不當正確性證明器） | 「LLM 提案、引擎裁決」原則 | AG-1 |
| 證據層 | decision snapshot manifest + WORM + 雜湊鏈 | 已核實（設計即建） | PROV-O 描述 | 重放離開 LLM 路徑（Addendum A6/A8） | RP-1 |
| API/Web | FastAPI；UI 依 v1 §8.2 清單 | 已核實（主流） | — | 快速迭代；後端確定性驗證為核心 | — |

## 3. 系統資料模型（§十三 第 5 項，摘要）

五層模型與儲存映射：

1. **原始文件層** → 物件儲存（檔案 + SHA-256 + 版本/生效日期/存取等級）。
2. **文件結構層** → Postgres 表：`document_version`, `clause_node(parent, order, section_path, page, anchor)`。
3. **知識/規範層** → `rule`（以 `rule-schema-v0.1` 校驗的 JSONB）+ `rule_doc_binding` + `rule_review_log`；KG 關係表含 `provenance + review_status`。
4. **流程/事件層** → `process_model`（BPMN XML + Petri net 派生物）、`event_log`（XES/OCEL 匯入）、`event`（三態 + 時間來源）。
5. **證據/主張層** → `claim`、`claim_evidence_link`、`decision_snapshot`（manifest雜湊）。規則層細部見 `../design/rule-schema-v0.1.md`。

## 4. MVP 範圍（交付物 C+D，§十三 第 6 項）

**輸入**：一個變更管理/CAPA 樣板場景：3–5 份種子 SOP（含 2 個版本差異）、20 條經種子規則、1 個含條件分支+回退循環+逾時的流程模型、1 份小型事件日誌（XES 正規/違規混合）。

**輸出必須包含**：

1. 版本化文件匯入 + 文件樹 + 條款錨點（頁碼）。
2. 混合檢索 + 樹導覽 + **逐字引用**（回答必附可點擊錨點）。
3. 規則庫（schema 校驗 + 專家核准流程 → active）。
4. 流程驗證：XES 匯入 → PM4Py conformance → 四分類報告（符合/偏差/缺失/無法確認）+ 證據引用。
5. 拒答：證據不足案件輸出 `insufficient_evidence` 或 `escalate`。
6. decision snapshot + 重放驗證（不含 LLM）。
7. 評測：`eval/` harness 接上真實 adapter，得 A–G 消融報告初稿。

## 5. 驗收門檻（§十三 第 7 項）

### 硬門檻（剛性，違反即不可交付）

| ID | 門檻 | 量測 |
| --- | --- | --- |
| H-1 | 禁止性狀態轉移放行率 | =0（測試集 ORD-001 類案例） |
| H-2 | 漏報率（實際違規判成合規） | =0（測試集） |
| H-3 | 證據不足判合規 | =0（ORD-002/EVD-001 類） |
| H-4 | 引用完整性 | 每個正式結論 ≥1 個可定位錨點，解析成功率=100% |
| H-5 | 注入不改判定 | SEC-001 類案例判定不受文件內指令影響 |
| H-6 | 重放一致性 | 同 snapshot 兩次重放輸出一致（不含浮點雜訊欄位） |

### 軟指標（Round 3 報告呈現，非門檻）

Recall@k / MRR / 引用正確率 / 規則判定準確率 / 正確拒絕率 / 無故拒答率（決定性案件）/
risk–coverage 曲線 / 漏報誤報混肴 / 成本與延遲。閾值為設定參數（承工作區「不做過早固化」公理），
Round 2 結束前與業務方共同設定初值並登錄版本。

## 6. 分期建置計畫（§十三 第 8 項）

### Round 2 — 可執行核心（建議磚塊順序）

| 磚 | 內容 | 驗收 |
| --- | --- | --- |
| R2-1 | 文件匯入 + 版本樹 + 條款錨點 + 雜湊 | TB-1、E-4 |
| R2-2 | 規則庫 + JSON Schema 校驗 + 審核工作流 + LegalRuleML 匯出（v0） | RB-1..3 |
| R2-3 | 流程驗證原型：BPMN 樣板 +（Rust 原生 token replay / 選配 PM4Py sidecar）+ 四分類報告 | PR-1..4、H-1、H-2 |
| R2-4 | 混合檢索 + 逐字引用引擎 | E-1..3、H-4 |
| R2-5 | 拒答 + risk–coverage 校準 + 閾值參數檔 | A5、H-3 |
| R2-6 | decision snapshot + 重放 | H-6、RP-1 |
| R2-7 | eval Rust crate 擴充：50+ 測試案例、雙人標註 κ、CI 接入 | A7 |

#### Round 2 實施狀態（2026-10-25 真機登記）

| 磚 | 狀態 | 落地與證據 |
| --- | --- | --- |
| R2-1 文件層 | ✅ 完成 | `ekp_platform/src/docs.rs`：5 文件／8 版本綁定重複檢查、effective_from/to 時間引擎、sections 錯點 SHA-256；單測 3 條 |
| R2-2 規則層 | ✅ 引擎側完成 | `src/rules.rs`：active 閘門（draft_ai 載入即拒收、審計痕跡）、`@pin` 綁定文件版本解析、conflicts_with 互指偵測、v0.1.1 `loop_policy`；schema v0.1.1 機讀版已出。**未做**：JSON Schema 驗證器串接（載入期 shape check 為手工等價）、審核工作流 UI、LegalRuleML 匯出 v0 |
| R2-3 流程驗證 | ✅ MVP 完成 | `src/process.rs`：Rust 原生 token replay 四分類（Conform/Deviation/Missing/Unconfirmed）+ 禁止對→H-1 保護帶；單測 3 條；PM4Py sidecar 依 §0 定案列 R3 選配 |
| R2-4 檢索 | ✅ MVP 完成 | `src/retrieval.rs`：字符 bigram cosine + 閾值參數（`--threshold`，預設 0.70）；命中必須達閾值。**未做**：向量/混合檢索（R3） |
| R2-5 拒答 | ✅ MVP 完成 | 引擎 abstain 語義（insufficient 引用清空）、誤拒即計量（refuse_all stub 證明）；risk–coverage 曲線在 `ekp_eval::metrics` 已出。**未做**：校準閾值參數檔（R3 與業務方設定） |
| R2-6 snapshot | ✅ 完成 | `src/snapshot.rs`：manifest SHA-256（doc 綁定雜湊 + 規則雜湊 + 事件序列雜湊）、重放一致性；selftest H-6 每案兩次重放 |
| R2-7 eval 擴充 | 🟡 部分完成（2026-11-03） | 67 案固定 corpus 已可一次載入、去重並由 `--corpus-selftest` 全量執行，且納入 canonical gate；雙人標註 κ 的工具、67 列盲化表、schema 與協定已就緒，但 0/2 標註者、**未測量**；CI workflow 已接線至含 corpus selftest 的固定入口，但無 remote/runner、**未執行**。不得標記為全部完成。 |

**R2-7 補完盤點（2026-11-03）**：50+ 案例已達 67，並由 `--corpus-selftest` 全量執行、納入 canonical gate；κ 與 CI 仍受外部條件阻擋。`eval/annotation/README.md` 定義雙人盲標協定，`--kappa` 對少於 50 案、同一標註者、case 集合不一致、重複 id、未知 verdict 與 undefined κ 全部 fail closed。現況 0/2 標註者、κ 無數值；workflow 從未執行。R2-7 因此維持「部分完成」，不得寫成完成或據此宣稱 A7 已完成。

**真機驗證彙總（2026-10-25, rustc 1.99.1, Windows）**：

```powershell
cd platform
cargo clippy --all-targets -- -D warnings   # 零警告
cargo test                                  # 單元 14/14 + 整合 6/6 PASS
cargo run -- --selftest                     # 49 assertions PASS（9/9 案例判定+引用集合、H-1..H-6）
cargo run -- --out report_out               # 消融報告：G_ekpi_engine acc=1.0 wrong_allow=0、硬門檻 ALL OK
```

| 硬門檻 | 量測（engine `--out` 審計 `engine_hard_gates.json`） |
| --- | --- |
| H-1 禁止轉移放行率 | 0/1（ORD-001 observed→violate） |
| H-2 漏報 | 0（violation_as_comply=0） |
| H-3 證據不足判合規 | 0（含 abstain 引用清空） |
| H-4 引用可定位錨點 | 0 unanchored（9 案全數 `doc@ver §sec`） |
| H-5 注入不改判定 | 5/5 登記、0 leak（§9.3／指令文本不入引用） |
| H-6 重放一致性 | 0 失敗（9 案 × 2 重放 manifest 相等） |

### Round 3 — 比較與改善

#### Round 3 實施狀態（2026-10-25 R3 真機登記）

| 項目 | 狀態 | 落地與證據 |
| --- | --- | --- |
| R2-2 補遺：JSON Schema 串接 | ✅ 完成 | `validate.rs` + schema `rule-schema-v0.1.1.schema.json`；CLI `--declared-rules-schema` 以 schema 校驗 `data/rules.json` 全量 → **20 records validated clean**（20 紀錄＝19 active＋1 draft_ai 投毒測試）；違規即整體拒執行 |
| R2-4 補遺：參數治理檔 | ✅ 完成 | `params.rs` + `config/engine-params.json`（8 invariant + retrieval_threshold 0.70）；`--params` fail-closed（未知鍵/缺鍵/版本不符/值域越界一律拒執行）；測試把 `DEFAULT_RETRIEVAL_THRESHOLD` 綁定為單一真源 |
| LegalRuleML 匯出 v0 | ✅ 完成 | `export.rs`：active 19 紀錄 → LegalRuleML 1.0 compact（`out/lrml/legalruleml_v1_compact.xml`，1,168 行/51,026 字節）。官方構件全集依 `research/legalruleml-notes.md`（Prefix/LegalSources/Associations/Factors/Statements/OverrideStatement/PenaltyStatement/Reparation）；衝突與例外無官方構件 → 僅 XML 註解登記（kind 字面詞可審計）；`overrides/overridden_by` → OverrideStatement（唯一可解析才發，歧義→註解）；任意 `not/any` 條件值 fail-closed 拒匯；匯出後 `validate_compact_export` 自檢（key 唯一性、keyref/@over/@under 引用可解析、禁用構件偵測） |
| 覆蓋率報告 | ✅ 完成 | `coverage_report.rs`（lib 模組）+ CLI `--coverage-report`：seed 9 案 + casebook 46 案真機逐案 decide；**18 個 active 規則 id 全覆蓋（0 未覆蓋）**＋2 個非 active 引用 id（draft／未登記各 1 案，顯式 note＋保守拒答）；觀測引用池 16 條 |
| HTML 渲染 | ✅ 完成 | CLI `--html`：report.md（剝 BOM）→ `report.html`（DOCTYPE＋`lang="zh-Hant"`＋charset；標題/表格/段落與 HTML 逃逸，零 JS） |
| 實況修正 | ✅ 完成 | **casebook_v3.jsonl 實測 46 題**（BC-101..BC-196；先前文件記 47 係誤記）：註解、單測與總結句一律改為 46 的真機值 |

**真機驗證彙總（2026-10-25, rustc 1.99.1, Windows）**：

```powershell
cd platform
cargo fmt                                                   # 格式化
cargo clippy --all-targets -- -D warnings                   # 零警告
cargo test                                                  # 60 lib + 3 governance_web + 8 integration = 71 PASS（bin harness 測試目標移除，見下）
cargo run --bin ekp_platform -- --selftest                  # 49 assertions PASS
cargo run --bin ekp_platform -- --declared-rules-schema     # 20 records validated clean
cargo run --bin ekp_platform -- --export-legalruleml out\lrml
cargo run --bin ekp_platform -- --coverage-report           # 55 案（9+46）真機統計
cargo run --bin ekp_platform -- --out out --coverage-report --export-legalruleml out\lrml --html
       # 消融 engine acc=1.000、hard gates ALL OK；out/ 產出 report.json/md/html + coverage_report.md + engine_snapshots.json + engine_hard_gates.json
```

**結構性調整（防本機應用程式控制策略誤攔新 build 的 bin 測試 exe；os error 4551）**：
coverage／markdown 渲染核心由 `main.rs` 遷入 lib 模組 `coverage_report.rs`（含 4 單測），
`Cargo.toml` 加 `[[bin]] name="ekp_platform" test = false`——`cargo test` 不再執行該 bin 的 harness exe；
CLI 行為不變。

**A12 後續（未完成，後續輪次）**：交付物 E（A–G 消融正式報告、失敗案例→回歸測試條目）、R2-7
（50+ 案例、雙人標註 κ、CI 接入）、GraphRAG/Agentic 分支資料決策。

- 失敗案例分析 → 回歸測試條目（v1 §四之 3「以失敗案例驅動」）。
- GraphRAG/Agentic 分支依資料決定去留（GraphRAG-Bench 證據基礎）。
- aProCheCk 型跨層級一致性檢查接入規範差距報告（A2）。

### 風險登錄

| 風險 | 緩解 |
| --- | --- |
| 規則抽取品質 | 生命週期閘門（A1）+ 規則覆蓋率帳本（A2）+ 對照版速規範 |
| 事件日誌品質 | GR-1..6 驗收 + 無法確認類別（缺 ≠ 沒發生） |
| 樹狀索引宣稱失真 | 過 long-context 基線（stronger-baselines-2025）再採用 |
| LLM 不可重現 | Snapshot 重放不落 LLM（A6） |
| 授權合規 | 每元件收錄時記錄 license（PageIndex/GraphRAG/PM4Py 授權待 Round 2 核對） |
| 範圍蔓延 | Addendum A12 交付物對應表為唯一範圍依據 |

| 版本 | 日期 | 修改 |
| --- | --- | --- |
| 1.0 | 2026-10-25 | 初版 |
| 1.1 | 2026-10-25 | Round 2 實施狀態登記（R2-1..R2-6 真機落地、R2-7 列 R3；硬門檻 H-1..H-6 引擎側全部 ok）；§0 已修正重排序與流程層行 |
| 1.2 | 2026-10-25 | Round 3 實施狀態登記：CLI 五旗標（schema 校驗 20 records clean、參數治理檔 fail-closed、LegalRuleML 1.0 compact 匯出、覆蓋率報告 55 案＝18 active id 全覆蓋＋2 非 active、HTML）；casebook 實測 46 題實況修正；cargo test 71 PASS、selftest 49 assertions |
| 1.3 | 2026-11-03 | R2-7 盤點：67 案固定 corpus 已由 `--corpus-selftest` 全量執行並納入 canonical gate；雙人標註 κ 工具、67 列盲化表、schema 與協定就緒，但 0/2 標註者、κ 未測量；CI workflow 已接線但無 remote/runner、從未執行。R2-7 維持部分完成。 |
