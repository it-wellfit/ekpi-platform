# EKPI Platform — 可追溯・可驗證的企業知識與流程智慧平台

**Evidence-Grounded Enterprise Knowledge & Process Intelligence Platform**

本資料夾是 Master Prompt（企業知識智慧中樞）執行 **Round 1（研究與技術選型）** 的產出，
同時是交給任何開發型 Agent（Claude Code / Cursor / OpenCode / Gemini Code）的專案母體。

**堆疊定案（2026-10-25 創辦人指示）**：Rust 全端主軸 + CSS-first 零 JS 展示層；
Mojo 列為前沿方法軌道（research-only）；Python 降為參考實作與選配 sidecar。
詳見 `docs/tech-selection-round1.md` §0。

## Documentation and current status

The canonical documentation index is [docs/README.md](docs/README.md). It names the owner and update trigger for each topic, and points to the product, architecture, backend, frontend, contract, quality, operations, decision, review and traceability documents. Do not treat a summary or a historical report as the current source of truth.

**Phase status (2026-10-26):** Phase A (0 → 1 MVP) is **not approved**; Phase B (production-grade) is **blocked**. The repository is a tested local prototype, not a production service. It has no approved deployment target, enterprise identity, tenant isolation, off-host backup, operational telemetry or named operations owner. See [phase gates](docs/quality/phase-gates.md) and [production readiness](docs/operations/production-readiness.md).

## Development workflow and CI

This repository uses a work-type-specific engineering workflow, GitHub issue/spec templates, risk-based validation, and Linux/Windows CI. Start with [AGENTS.md](AGENTS.md) and [the engineering workflow](docs/development/README.md). The exact commands, risk gates, existing-feature [regression checklist](docs/development/regression-checklist.md), and release/retro checks are in [quality gates](docs/development/quality-gates.md) and [release guidance](docs/development/release-and-retro.md).

The two Cargo crates have separate manifests and lockfiles. Run checks from each crate directory. CI calls the same fixed entry point contributors run locally (`scripts/check.sh` / `scripts/check.ps1`) on Ubuntu and Windows, plus a separate security job. Do not treat a README test count as current evidence unless those commands were run for the revision being delivered.

## 目錄結構

```
ekpi-platform/
├── README.md                                ← 本文件
├── AGENTS.md                                ← Agent 開發規範與路由
├── GLOSSARY.md                              ← 專案領域詞彙索引
├── rust-toolchain.toml                      ← 固定 Rust 工具鏈
├── .github/                                 ← CI、Issue／PR 範本與 Dependabot
├── docs/
│   ├── README.md                            ← canonical documentation map（owner／更新條件）
│   ├── product/                              ← 產品問題、角色、MVP 範圍與非目標
│   ├── architecture/                         ← 系統結構、模組邊界、資料流、威脅模型
│   ├── backend/                              ← 服務面、持久化、授權邊界、已知缺口
│   ├── frontend/                             ← 頁面、表單、互動狀態、無障礙
│   ├── contracts/                            ← CLI、web routes、資料格式、LegalRuleML 契約
│   ├── quality/                              ← 測試策略、Phase A/B 門檻、AI 評測政策
│   ├── operations/                           ← 安裝、設定、備份、事故、生產就緒度
│   ├── decisions/                            ← 決策索引與 ADR 範本（ADR 本體在 adr/）
│   ├── reviews/                              ← 審查、回顧與改善登錄
│   ├── traceability/                         ← ID 規則與需求→證據登錄
│   ├── development/                          ← 工作路徑、範本、回歸清單、品質門檻、發布與移交
│   ├── agents/                               ← GitHub issue、triage、領域知識與社群技能評估
│   ├── adr/                                  ← 工程決策記錄
│   ├── prompt/master-prompt-addendum-v2.md  ← Master Prompt v1 的 12 條訂正（交給開發 Agent 時必併讀）
│   ├── tech-selection-round1.md             ← §0 堆疊翻轉 + 技術整合矩陣／MVP／驗收門檻／分期計畫（§6 已登記 Round 2／Round 3 實施狀態）
│   └── governance/                           ← 治理需求、探索工作簿、試點手冊與驗收矩陣
├── design/
│   ├── rule-schema-v0.1.md                  ← 規則模型說明 v0.1（deontic、生命週期、LegalRuleML 對照；尾注指向 v0.1.1）
│   ├── rule-schema-v0.1.schema.json         ← 規則模型 JSON Schema（v0.1）
│   ├── rule-schema-v0.1.1.md                ← v0.1.1 差異說明（+sequencing.loop_policy，CAP-001/CAP-002 語義）
│   ├── rule-schema-v0.1.1.schema.json       ← 規則模型 JSON Schema（v0.1.1，機讀）
│   └── process-verification-selection.md    ← 流程驗證層選型：Petri/XES/OCEL + 品質關卡 + 拒答校準
├── platform/                                ← 確定性核心（Round 2 交付物 C/D 的 MVP 引擎＋Round 3 匯出／治理／覆蓋率）
│   ├── Cargo.toml / src/                    ← docs/rules/process/retrieval/snapshot/engine/validate/params/export/coverage_report，`#![forbid(unsafe_code)]`、clippy -D warnings 零警告
│   ├── data/                                ← docs.json（5 文件 8 版本，含注入測試欄位）；rules.json（20 紀錄＝19 active＋1 draft_ai 投毒測試）；processes.json（3 流程模型）
│   ├── config/engine-params.json     ← 參數治理檔（`--params` 顯式載入或預設路徑自動載入；任一不符即 fail-closed 拒執行）
│   ├── tests/integration.rs                 ← 端到端整合測試（真實 data + 種子案例）
│   ├── report_out/                          ← `--out` 產物（report.json/md、engine_snapshots.json、engine_hard_gates.json）
│   └── out/                                 ← R3 旗標產物（report.json/md/html、coverage_report.md、lrml/legalruleml_v1_compact.xml）
├── research/
│   ├── papers.csv                           ← 論文登錄表（27 條引用，2026-10-25 全數核實）
│   ├── standards.csv                        ← 標準與工具登錄表（11 條，含 Mojo research-lane）
│   └── db-schema.md                         ← 登錄表欄位定義與更新作業流程
└── eval/                                    ← 評測 harness（Rust crate = 規範性實作，lib+bin 可複用）
    ├── Cargo.toml / src/                    ← `#![forbid(unsafe_code)]`，clippy -D warnings 零警告
    ├── cases/seed_cases.jsonl               ← 9 條語言中立種子案例（含 3 條「應拒答」）
    ├── configs/ablations.json               ← A–G 七組消融配置
    ├── report_out/                          ← 本機實測產物（report.json / report.md）
    └── reference-python/                    ← Python 版（降級為參考實作，NOT RUN）
```

## 如何執行自檢（已於本機真實驗證）

固定入口（文件連結、高信號機密樣式、格式、Clippy、測試、自檢、schema）：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

更嚴格的 pre-PR／release gate（`-IncludeSmoke` 另加 CLI 產物驗證）：

```powershell
powershell -ExecutionPolicy Bypass -File gate.ps1 -IncludeSmoke
```

逐項命令與 **2026-10-25 真機數據**（不是永久常數；交付修訂版需重跑）：

```powershell
cd eval
cargo run --locked -- --selftest        # 10/10 檢查通過（2026-10-25, rustc 1.99.1）
cargo run --locked -- --out report_out  # 產出 report.json + report.md（含逐案例失敗分析）

cd ..\platform
cargo clippy --locked --all-targets -- -D warnings              # 零警告
cargo test --locked                                             # 170 lib + 2 edge_cases + 3 governance_web + 8 integration = 183 PASS
cargo run --locked --bin ekp_platform -- --selftest             # 49 assertions PASS：9/9 案例判定+引用集合、H-1..H-6、draft 規則拒收
cargo run --locked --bin ekp_platform -- --declared-rules-schema    # 20 records validated clean（19 active＋1 draft_ai）
cargo run --locked --bin ekp_platform -- --export-legalruleml out\lrml  # LegalRuleML v1.0 compact（19 PrescriptiveStatement、51,026 bytes、匯出自檢）
cargo run --locked --bin ekp_platform -- --coverage-report      # 55 案（seed 9＋casebook 46）真機統計
cargo run --locked --bin ekp_platform -- --out out --coverage-report --export-legalruleml out\lrml --html
# R3 組合：engine acc=1.000、hard gates ALL OK；--params config\engine-params.json 可顯式載入治理檔；out/ 產 report.json/md/html＋coverage_report.md
```

## 跨專案治理試點工作台

`platform` 保留原有確定性引擎與 CLI，並新增一個本機、CSS-first 的治理試點介面：

```powershell
cd platform
cargo run --locked --bin governance_web
```

開啟 `http://127.0.0.1:3000`。工作台支援本機專案登錄、Microsoft／Google／其他來源文件連結與版本參照、人工證據審查、29 項品質檢查、流程草稿與負責人發布、風險／變更／缺失／CAPA、12 階段／5 關卡，以及追加式 SHA-256 稽核事件。預設資料檔位於 `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl`，可用 `EKP_GOVERNANCE_DATA` 指定測試或試點資料檔。

本原型只記錄人工提供的來源參照，**不會**登入 Microsoft／Google、複製文件、呼叫 AI 或驗證外部權限。它沒有企業身分驗證，僅可在本機使用非敏感資料，不可直接部署到共用或公開網路。工作台會明確標示未連線與空資料狀態；工作流程 AI 草稿需待資料政策和供應者審查後接入。

治理探索訪談、工具清冊、基線量測、需求及試點驗收材料見 [`docs/governance/README.md`](docs/governance/README.md)。尚未進行真實租戶連接或組織訪談；正式試點需先由業務、IT／資安與流程負責人核准範圍和資料條件。

platform 引擎已接入同一 runner（adapter `G_ekpi_engine`）：
`--out` 產出的報告並列 `G_ekpi_engine` 與三個確定性 stub（oracle / allow_all / refuse_all），
`engine_hard_gates.json` 逐條登記 H-1..H-6 的真機量測值。

自檢以三個確定性 stub 驗證指標與硬門檻管線：
- `stub:oracle`：verdict accuracy 1.0、wrong_allow=0、正確拒答 1.0、escalation 1.0、citation recall 1.0。
- `stub:allow_all`：**必須觸發硬門檻**（violation_as_comply>0）→ 證明漏報門檻真的會擋下壞系統。
- `stub:refuse_all`：definitive 案件無故拒答率 1.0 → 拒答被計量。

真實系統以 `~/src/systems.rs` 的 `System` trait 接入，跑同一 runner 即得 A–G 消融報告。

## 交給開發 Agent 的最小啟動指令

1. 讀 `docs/prompt/master-prompt-addendum-v2.md`（與 Master Prompt v1 併讀；衝突時以 addendum 為準）。
2. 讀 `docs/tech-selection-round1.md`（§0 堆疊定案 → 整合矩陣 → 驗收門檻 → 分期建置）。
3. 執行 `eval` 自檢與報告生成。
4. 依 `tech-selection-round1.md` §6 的「第二輪磚塊順序」開始實作（Rust crates）。

## 品質規則（承自工作區 GEMINI.md 與 Master Prompt v1 §12）

- **絕對反模擬**：探針失敗（DNS/HTTP/TLS）如實登錄，不得偽造（實例：`tf-pm.org` TLS 異常、
  `legalruleml.oasis-open.org` 與 `mojo.modular.com` DNS 未解析 → 改以 IEEE/OASIS docs/OASIS TC/`modular.com` 交叉核實或標 `verified=no/partial`）。
- **證據先於結論**：任何引用進入論文/選型文件前，必須在 `research/` 登錄並標記 `verified_date` 與驗證方法。
- **不做過早固化**：閾值皆為設定參數；僅漏報類硬門檻（誤判合規=0）屬剛性約束。
- **Rust 宣法**：所有 Rust 程式碼第一行 `#![forbid(unsafe_code)]`；`cargo clippy --all-targets -- -D warnings` 零警告。
- **展示層零 JS**：`<script>` 數量 = 0（CSS-first）。
- **PowerShell 邊界**：多指令串接使用 `;`，不使用 `&&`。

## 變更記錄

| 日期 | 變更 | 作者 |
| --- | --- | --- |
| 2026-10-25 | 初版：18 條論文 + 10 條標準核實；規則 schema v0.1；流程選型；評測骨架（Python 版） | OpenCode (GLM) |
| 2026-10-25 | **堆疊翻轉**：Rust 全端主軸 + CSS-first；Mojo 列研究軌道；Python harness 降級參考；**Rust harness `ekp_eval` 真機 PASS（10/10，clippy 零警告）**；standards.csv +Mojo（partial） | OpenCode (GLM) |
| 2026-10-25 | **Round 2 核心落地**：`ekp_platform` 確定性引擎（文件版本樹/active 規則層/token replay/bigram 檢索/abstain/snapshot 重放）真機 PASS（selftest 49 assertions、單元 14/14、整合 6/6、clippy 零警告）；規則 schema 升 v0.1.1（loop_policy）；`G_ekpi_engine` 接入 eval runner，消融報告硬門檻 H-1..H-6 全部 ok；Round 1 文件 4 處缺陷修正（重排序行、流程層行、盤點計數、A4 放行率=0） | OpenCode (GLM) |
| 2026-10-25 | **Round 3 落地**：CLI 五旗標（`--declared-rules-schema`／`--params` 參數治理檔／`--export-legalruleml`／`--coverage-report`／`--html`）；LegalRuleML v1.0 compact 匯出（active 19 紀錄→官方構件，匯出自檢 key/ref）；覆蓋率報告 55 案真機（seed 9＋casebook 46；18 active id 全覆蓋＋2 非 active 引用 id 顯式拒答；casebook 實測 46 題修正）；papers.csv ＋9 條（合計 27 全數核實）；cargo test 71 PASS、selftest 49 assertions、clippy 零警告 | OpenCode (GLM) |
| 2026-10-26 | **全生命週期文件與品質入口**：新增 canonical 文件索引、product／architecture／backend／frontend／contracts／quality／operations／decisions／reviews／traceability 文件；Phase A/B 門檻與生產就緒度（A 未核准、B 受阻）；`scripts/check-docs` 與高信號機密樣式掃描接入 `scripts/check` 與 `gate.ps1`；CI 改為 `validate` matrix + `security`；修正資料庫／WORM／tenant／monitoring／restore 等過度宣稱；`.env.example`、`.gitignore`、`.gitattributes` 對齊實際行為 | OpenCode (DeepSeek V4.1) |
