# Master Prompt v2 Addendum（訂正條款）

> **交付方式**：本文件與使用者提供的《MASTER PROMPT：下一代企業知識、因果推理、流程驗證與自動化智慧平台》v1 併讀。
> 衝突時**本文件優先**。條款已按工作區「規則總數壓縮（≤20）」精神合併為 12 條，
> 每條標明修訂 v1 的哪一節。簽收後任何修改需記錄於本檔案表尾。

| 條款 | 修訂 v1 位置 | 內容 |
| --- | --- | --- |
| **A1 規則層採用基座 schema** | §3.4 | 規則一律以 `design/rule-schema-v0.1.schema.json` 校驗。強制欄位：deontic 語氣（obligation/prohibition/permission/recommendation + strength）、例外關係（`overridden_by / overrides / conflicts_with`）、有效時間、**結構化 `evidence_requirements`**（每一種必要證據的類型、數量、metadata 條件與 `missing_evidence_result`）、`review.status`。目標匯出格式為 LegalRuleML v1.0（OASIS Standard, 2021-08-30），首次匯出前逐項對照官方規格構件名稱。禁止把 LLM 抽取草稿（`draft_ai`）路由進正式判定：只有 `review.status=expert_validated/active` 的規則可被引用。 |
| **A2 規則↔文件版本綁定與覆蓋率** | §3.4 | 每條規則綁定 {document_id, document_version, clause_refs, verbatim_text}；建立覆蓋率帳本（條款 → 規則映射），文件改版時自動產出「尚未映射/失效規則」差距報告；衝突偵測（兩規則矛盾且無 override）自動開缺漏工單，供 SOP 擁有者修文。 |
| **A3 事件層採用 XES/OCEL 與品質關卡** | §3.5 | 事件日誌導入格式：XES（IEEE 1849-2023，2026 年不得再引用 2016 版）為主；一案件牽涉多實體（申請、文件、表單、簽核）時採 OCEL 2.0。導入後先過六項品質關卡（GR-1 實體解析、GR-2 亂序、GR-3 重複、GR-4 時鐘偏差、GR-5 事件完整性、GR-6 角色映射），未過關的日誌**不得**用於任何正式符合性判定。事件保留三態：`observed / assumed / unconfirmed`；缺紀錄 ≠ 未發生。 |
| **A4 conformance checking 落地工具鏈** | §5、§11 交付物 D | 流程語義以 **Petri net 為執行/驗證核心**；MVP 以 **Rust 原生 token-based replay + 可達性檢查** 實作（tech-selection §0 堆疊定案），PM4Py 降為選配 sidecar（僅 R3 作 alignments 對照 oracle，以雜湊工件契約交換）；BPMN 作為撰寫層、DMN 作為決策表層。符合性檢查輸出分四類：符合路徑 / 偏差 / 缺失事件 / 無法確認事件，附證據引用。**硬門檻：引入測試集中的禁止性狀態轉移，放行率 = 0**。交付物 D 以此工具鏈實作，不再以「建立一個範例流程」含糊表述。 |
| **A5 拒答校準為一等公民** | §9 | 「無法判定 / 升級人工」是正式輸出類型，不是失敗計數。評估必含：(1) 決定性案件上的無故拒答率；(2) 證據不足案件上的正確拒絕率；(3) risk–coverage 曲線（以 confidence 排序百分比覆蓋誤差）；(4) 閾值治理：閾值為可設定參數並記錄版本，漏報類錯誤的代價權重 > 誤報，於成本矩陣中顯式聲明。 |
| **A6 可重現性：判定時刻快照** | §8.2、§9 | 每個正式判定存 decision snapshot manifest：文件版本雜湊 + 規則版本 + 檢索所得片段雜湊 + 事件集 + prompt 模板雜湊 + 模型版本 + 配置。**重放驗證不得依賴 LLM 重跑**：重放只重執行確定性路徑並比對雜湊；LLM 輸出以當時紀錄比對。原則：「LLM 提案，確定性引擎裁決」。 |
| **A7 評估統計設計** | §9 | 五組以上基準比較改為**消融式**：共用 harness、共用測試集（`eval/`），以 G（完整系統）為主軸逐項移除構件，不得各建一套。實驗前記錄樣本量目標與置信區間；標註使用雙人標註並報 Cohen's κ；測試集版本明確綁定文件版本（`seed_cases.jsonl` 已內建 `doc_versions` 欄位）；避免用同一標註集同時做調參與報告。 |
| **A8 資安訂正** | §10 | 除 v1 已有內容外：(1) 「規則投毒防線」— AI 抽取規則在核准前對判定引擎不可見（A1 生命週期即為機制）；(2) 證據庫以 WORM 式儲存 + 內容雜湊鏈，寫後不可改；(3) 證據存取控制與隱私層級在匯入時標定；(4) Prompt injection 進入正式測試集（seed case `SEC-001`）。 |
| **A9 研究登錄表採用工作區現成庫** | §2 | `research/papers.csv`（27 條，2026-10-25 全數核實；Round 3 增補 9 條——原 18 條見初版登記）與 `research/standards.csv`（11 條）為現狀基準。新引用必須先登錄、標 `claim_source`（peer_reviewed / author_reported / vendor_reported）與驗證方法（official_page_fetch / crossref_pii / openalex / websearch_cross_check），未能核實者標 `verified=no` 並禁入正式報告。新增必研標準：LegalRuleML、OCEL、IEEE 1849-2023、SHACL、PROV-O（均已在 standards.csv）。 |
| **A10 Agent 修正必須產出新證據** | §7 | 反思/補查每輪結尾驗證：是否新增了可定位的證據片段、修改了主張、或觸發規則重新評估；三者皆否則停止並輸出未解決事項。迭代次數一律記錄，不得作為正確性證據。 |
| **A11 因果與程序分離的落地位置** | §6 | 規範依賴（required_before）由規則引擎與流程模型負責；真實因果主張進入 `design/` 因果模組時必須宣告：因果假設清單、資料來源、識別策略、混淆因素與反證搜尋結果；資料不足以識別時輸出 `cannot_identify`，禁止生成確定性因果結論。CausalGraph2LLM 型查詢集為因果理解的獨立回歸測試。 |
| **A12 交付物對應表** | §11 | 已於 Round 1 產出：交付物 A 部分（research/）+ 選型與驗收（docs/）。**Round 2 已完成（2026-10-25 真機登記）**：交付物 C（MVP 引擎）與 D（流程驗證原型）以 Rust crate `ekp_platform` 落地——版本化文件+條款錨點+SHA-256（R2-1）、active 規則層+`@pin` 綁定+衝突偵測+v0.1.1 `loop_policy`（R2-2，schema `rule-schema-v0.1.1.schema.json`）、Rust 原生 token replay 四分類+H-1 保護帶（R2-3）、bigram 檢索+閾值參數（R2-4）、abstain 拒答語義（R2-5）、decision snapshot manifest+重放一致性（R2-6）；`ekp_platform --selftest` 49 assertions PASS、`cargo test` 單元 14/14 + 整合 6/6、`--out` 消融報告硬門檻 H-1..H-6 全部 ok（詳 tech-selection §6「Round 2 實施狀態」）。**Round 3 已完成（2026-10-25 真機登記）**：JSON Schema 驗證器串接（`--declared-rules-schema`：20 records validated clean，20 紀錄＝19 active＋1 draft_ai）、參數治理檔（`--params` fail-closed：不變參數任一不符即拒執行）、LegalRuleML v0 匯出（`--export-legalruleml`：active 19 紀錄→官方構件 compact，匯出後自檢 key/ref）、覆蓋率報告（`--coverage-report`：55 案真機＝seed 9＋casebook 46，18 active id 全覆蓋＋2 非 active 引用 id 顯式拒答；casebook 實測 46 題）、HTML 渲染（`--html`：零 JS）；`cargo test` 60 lib＋3 governance_web＋8 integration＝71 PASS、selftest 49 assertions、clippy -D warnings 零警告（詳 tech-selection §6「Round 3 實施狀態」）。後續：交付物 E（A–G 消融正式報告）、R2-7 擴充（50+ 案例、雙人 κ、CI）、GraphRAG/Agentic 分支依資料決定去留。評測 harness 為 Rust crate `ekp_eval`（規範性；2026-10-25 於 rustc 1.99.1 真機 PASS 10/10、clippy -D warnings 零警告），Python 版列為 `eval/reference-python/` 參考實作；v1 §八「Python 作為主語言」修訂為 **Rust 全端主軸 + CSS-first 展示層**（tech-selection §0）。交付物 B、F、G 的規格以 `tech-selection-round1.md` §4–§6 為準。 |

## 訂正理由對照（溯源）

- A1–A2：Master Prompt v1 的 rule 結構為敘述性欄位清單，缺少 deontic 語氣、override 語義與專家核准閘門，無法支撐「稽核級」宣稱；LegalRuleML v1.0 已核實為 OASIS 正式標準。
- A3–A4：v1 的流程驗證能力清單正確但無落地元件；conformance checking 是學界正解（Process Science 綜述已登錄），工具鏈（PM4Py/XES/OCEL）全部真實可查。
- A5–A7：v1 已有「正確拒答率」指標但缺校準機制與統計設計；漏報比誤報危險的原則需要成本矩陣與硬門檻落實。
- A8：v1 §10 已覆蓋注入與權限，本條補齊審計級完整性与投毒防線。
- A9–A12：把 Round 1 已完成的事實登錄進主提示詞，避免開發 Agent 重做研究。

| 版本 | 日期 | 修改 | 核准 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-25 | 初版 12 條 | — |
| 1.1 | 2026-10-25 | A4 交付物 D 表述落實為 Rust 原生 replay 工具鏈（放行率=0）；A12 登記 Round 2 `ekp_platform` 落地與硬門檻 H-1..H-6 量測結果 | — |
| 1.2 | 2026-10-25 | A12 登記 Round 3 完成：Schema 串接／參數治理檔／LegalRuleML 匯出 v0／覆蓋率報告／HTML；casebook 實測 46 題修正；A9 現狀更新為 papers 27 條／standards 11 條；交付物 E 與 R2-7 列後續 | — |
