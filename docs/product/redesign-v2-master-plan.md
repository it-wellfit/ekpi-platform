# EKPI Platform 產品再設計與精工總計畫 v2.0

- **狀態：** Active（設計基準）；實作以 TDD 切片推進，每片完成須過 canonical gate
- **Owner：** Product + Engineering + QA
- **最後審閱：** 2026-11-06
- **上位文件：** [requirements](../governance/02-requirements-v0.1.md)、[functional spec](../governance/05-functional-spec-v1.md)、[data and state model](../governance/06-data-and-state-model-v1.md)、[acceptance matrix](../governance/04-acceptance-test-matrix.md)
- **配套交付：** [品質標準與驗收準則](../quality/quality-standards-v2.md)、[測試驗證計畫](../quality/test-verification-plan-v2.md)、[交付清單](../operations/delivery-checklist-v2.md)
- **裁決：** 本文件與 canonical 規格衝突時，以 governance 規格為準；本文件僅定義「如何達標」。

---

## 1. 現況基線（已驗證事實，2026-11-06）

| 層 | 現況 | 證據 |
|---|---|---|
| 建置 | Workspace：`apps/web`、`crates/{domain,application,adapters}`、`eval`、`platform`（edition 2024、resolver 3） | 根 `Cargo.toml` |
| 確定性核心 | 19 條 active 規則、7 份版本化文件、3 個流程；引擎判定、快照重放、LegalRuleML 匯出 | `platform/src/engine.rs` 等 36 模組；selftest 49 |
| 品質門檻 | `8ba4e0c`：canonical ALL CHECKS PASSED；445+ 測試；bundle 還原演練 PASS | `docs/traceability/register.md`、`backup-log.md` |
| 治理工作台 | 本機 session 認證（PBKDF2）、RBAC 三角色、CSRF double-submit（零 JS 相容）、append-only 稽核鏈、readiness/metrics、結構化錯誤、限流與逾時 | `platform/src/governance_web.rs`；25 route 測試 |
| 進行中（peer WIP，未提交） | approval workflow、channels、workflow domain、postgres adapters、ADR-0002/0003、`migrations/20261106000000_approval_workflow.sql` | 工作樹未提交檔案；lockfile 尚未解析完成（`--locked` 失敗） |
| 最薄弱層 | `apps/web`（Askama 呈現層）僅 1 個模板 + 60 行零 JS 測試——產品主要頁面系統尚未建立在這一層 | `apps/web/src/` |

**誠實聲明**：本計畫不宣稱任何未驗證能力；A1（真實試點）維持 `Not met`；Phase B 維持 Blocked（DEC-12…DEC-20）。

---

## 2. 十二工藝維度 → 可量測工程標準

| 維度 | 落地標準（可驗收） |
|---|---|
| 精緻 | 每個互動元件七狀態齊備（Default/Hover/Focus/Disabled/Loading/Empty/Error）並逐頁審計；`:focus-visible` 焦點環、過渡僅 transform/opacity、`prefers-reduced-motion` 生效 |
| 精美 | 樣式僅來自 Design Token——**本儲存庫**的 token 源為 `apps/web/static/css/design-system.css` 與 `platform/assets/governance.css`（根工作區的 `web/assets/design-system.css` 不屬於本儲存庫，先前誤引已更正）；字級/間距/色彩皆取 scale 變數，模板與資產**零裸寫色碼**（掃描 0 內聯值）；九頁面共用同一視覺語言（字階、間距節奏、元件外觀） |
| 精密 | 契約先行：規則庫 schema、評測集、匯出格式（LegalRuleML/JSON）皆 schema 校驗；對外行為以版本化契約（CTR-*）鎖定；快照重放逐位元組一致 |
| 精良 | `#![forbid(unsafe_code)]`、clippy `-D warnings` 零警告、fmt 乾淨、測試 100% 通過且總數不退步；canonical gate exit 0 |
| 精巧 | 零 JS 達成全部互動：CSRF＝cookie＋隱藏欄位 double-submit、導航＝`:target`/`:has()`、回饋＝`:hover/:focus-visible/:user-invalid`、表單＝POST+PRG；以 e2e 證明「無 JS 仍可用」（零 JS 是憲章鐵律，不屬美學維度） |
| 精雕 | 文件僅追加式修改（歷史證據不可刪）；重複實作消除（單一隨機源、單一 cookie 解析器）；每切片留回顧筆記；缺陷逐條登錄 IMP-* 追蹤至閉環 |
| 精準 | 每個對外數字可從具名工件推導（`engine_snapshots.json`/`engine_hard_gates.json`/測試輸出）；引擎決定論（同輸入同輸出）；證據列綁「指令＋修訂＋結果」；UNKNOWN ≠ PASS ≠ FAIL |
| 精湛 | 修復以完整正確設計為單位（不做半套補丁）；已完成切片無遺留 TODO/FIXME；錯誤處理全路徑結構化；狀態機以型別表達而非散落的字串比對 |
| 精品 | 發布門檻：H-1…H-6 全 OK、28-case 檢索 gate 全過、67-case 語料回歸、bundle 還原演練 PASS；僅發布已驗證宣稱 |
| 精益 | 每切片「紅→綠→證據→回顧」；缺陷 IMP-* 閉環率追蹤；門檻只升不降（放寬需 ADR＋贊助人核准） |
| 精研 | 效能先量測後優化（基線 JSON 與優化後並列）；技術選型先有研究筆記與 ADR（含備選方案與後果） |
| 精選 | 新依賴需審查（crates.io 健康/授權/維護紀錄）＋`cargo audit`/`cargo deny` 乾淨；SBOM 每次發布重產生；工具/技能僅用已驗證者 |

> 可否決的 Q-* 指標見[品質標準](../quality/quality-standards-v2.md)（本表為維度→標準映射；Q 表為執行門檻）。

---

## 3. 改進方案

### 3.1 架構（目標與差距）

```
apps/web (Askama, 零 JS 呈現層)          ← 3.3 主要投資
   ↓ 呼叫
crates/application (use cases, ports)    ← WIP: approval 使用案例補齊
   ↓ 實作
crates/domain (entities, policies)       ← WIP: workflow/channels 聚合根
   ↑
crates/adapters (jsonl_ledger, postgres, identity, audit, …)
─────────────────────────────────────────
platform crate（確定性核心，已精工）      ← 引擎/規則/流程/檢索/快照
eval crate（消融與指標）                  ← 已精工
```

依賴方向：Presentation → Application → Domain；Adapters 僅實作 ports。目前以模組邊界人工審查把關；自動化依賴方向檢查尚未存在，列為後續切片候選——**不得宣稱已有自動化強制**。

**架構決策（依 ADR-0002/0003 方向收斂）：**
- AD-1 整合邊界單一化：所有外部系統（電商、ERP、文件庫）經 `application/ports.rs` 定義的單一 trait 面，adapter 只做轉換與重試；ADR-0002/0003 為準（peer 進行中，收斂後落地）。
- AD-2 儲存雙軌：本機 JSONL 帳鏈（已驗證）為試點真相；PostgreSQL（`migrations/` + `adapters/postgres`）為企業目標態，經同一 port 切換，行為由共同整合測試鎖定。
- AD-3 呈現零 JS 不變：憲章 §2.3；任何需要互動的元件一律以 CSS/HTML 原生能力實現。

### 3.2 模組層級設計（缺口 → 目標）

| 模組 | 現況 | 目標（本計畫內） | 驗收 |
|---|---|---|---|
| apps/web 頁面系統 | 1 模板 | 9 頁面系統（見 3.3） | e2e 每頁 1 測試 + 七狀態審計 |
| approval workflow | WIP（domain+application+postgres+migration） | 完成：建立→複核→核准→封存，全程入帳鏈 | 整合測試 6 條（含拒絕/逾時/越權） |
| channels/integration | WIP | 完成 ADR-0003 邊界：連接器失敗顯示「未確認」，不偽造同步 | 模擬故障注入 4 條 |
| engine 檢索 | 詞彙 bigram | 量測後決定倒排索引（P-01），保持結果順序確定性 | 前/後 benchmark 並列 |
| 匯出 | LegalRuleML 1.0 compact | 增加 JSON decision export（v2 契約） | schema 校驗 + 重放一致 |
| 觀測 | /metrics 聚合 + JSON 存取日誌 | +SLO 文件與警示門檻草案（本機可算） | `docs/operations/slo-draft.md` |

### 3.3 介面與互動設計（零 JS、CSS-first）

**頁面清單（apps/web + governance_web 共用設計語言）：**

| 頁 | 主要任務 | 關鍵狀態要求 |
|---|---|---|
| `/` Dashboard | 總覽專案/待辦/門檻狀態 | Loading（骨架）、Empty（無專案）、Error（帳本鎖） |
| `/projects` | 建立/篩選專案 | PRG、`aria-live` 結果、分頁 |
| `/evidence` | 證據登錄與審查 | 表單驗證 `:user-invalid`、審查紀錄時間軸 |
| `/workflows` | 流程版本與發布 | 版本 diff、發布需 Approver |
| `/controls` | 品質個案/CAPA | 狀態機按鈕僅顯示合法轉移 |
| `/lifecycle` | 12 階段 5 關卡 | 關卡未達＝停留，不得繞過 |
| `/audit` | 稽核事件鏈 | 雜湊鏈驗證徽章（通過/中斷） |
| `/login` | 本機登入 | CSRF 欄位、通用錯誤、鎖定不揭露 |
| `/ready`+`/metrics` | 營運探針 | 公開、無敏感資料 |

**互動規範：** 分區切換 `:target`；導航高亮 `:has()`；回饋僅 CSS（`:hover/:focus-visible/:user-invalid`）；`prefers-reduced-motion` 支援；表單一律 POST+PRG；錯誤訊息含「問題＋可能原因＋處理方式」；介面文字繁體中文（台灣情境）。

### 3.4 實施切片（順序、每片以 canonical gate 收口）

| # | 切片 | 內容 | Gate |
|---|---|---|---|
| S1 | 完成進行中整合層 | 補齊 approval/channels 紅測試→實作→綠；lockfile 收斂 | workspace 全綠 + check.ps1 |
| S2 | apps/web 頁面系統 | 9 頁面模板 + 路由 + e2e | e2e 全綠 + 零 JS 掃描 |
| S3 | UI 七狀態審計 | 逐頁補狀態 + a11y 檢核清單 | 審計表全勾 |
| S4 | 匯出 v2 + SLO 草案 | JSON export + slo-draft | schema + 重放一致 |
| S5 | 效能（量測後） | 倒排索引或緩存，僅當 p95 超標 | 前/後 benchmark |
| S6 | 證據收口 | bundle、backup-log、register、phase-gates 更新 | gate.ps1 -IncludeSmoke |

---

## 4. 誠實邊界與外部決策

- A1 真實試點（DEC-01）與 Phase B（DEC-12…DEC-20）為外部決策；本計畫產出可執行環境與規格，不代答完成宣稱。
- 企業 SSO/MFA、對外網路、K8s、訊息佇列仍屬 Phase B；本機原型不得描述為具備這些能力。
