# 規則模型 v0.1 設計說明

> 配套：`rule-schema-v0.1.schema.json`（機讀版）、Master Prompt v1 §3.4 + Addendum A1–A2。
> 本 schema 定義「規範與知識模型」的最小可行規則表示。v0.1 是設計基準，
> 非 OASIS 原生格式；匯出目標為 LegalRuleML v1.0（見 §4）。

## 1. 定位與邊界

- 本 schema 表示**規範**（什麼該做/禁止/允許、條件、例外、證據要求），不表示**流程**（步驟怎麼走）。
  流程歸 BPMN/Petri net；兩者透過 `sequencing.required_before` 的 activity 引用串接。
- 規則來源必須是逐字引文（`source.verbatim_text`），並綁定明確的文件版本；
  LLM 只能在 `provenance.origin = ai_extracted` 的草稿上工作，**不得**直接生成 `active` 規則。

## 2. 生命週期（review.status 狀態機）

```
draft_ai ──(領域專家核)──▶ expert_validated ──(生效登記)──▶ active ──(改版/撤銷)──▶ retired
    │                         ▲                                              
    └──(退回修改)──  expert_returned ──────────────────────────────────────────
```

- 判定引擎**只讀取 `active`**（及其規定時間內有效的 supersedes 鏈）。
- `draft_ai / expert_returned` 對判定引擎不可見 → 這是「規則投毒防線」的機制（A8）。
- 每次轉移必須寫 `rule_review_log`（reviewer、日期、理由）。

## 3. 核心欄位摘要（詳列見 JSON Schema）

| 群組 | 欄位 | 說明 |
| --- | --- | --- |
| 識別 | `rule_id`, `rule_version` | 版本化主鍵；supersedes/superseded_by 構成時間鏈 |
| 語氣 | `deontic.modality` | obligation / prohibition / permission / recommendation |
| 語氣 | `deontic.strength` | mandatory / recommended / optional |
| 主體 | `subject` | 實體類型 + actor_role + 適用範圍（process/asset/org_unit/jurisdiction） |
| 條件 | `condition` | 遞迴條件樹（all/any/not/predicate eq ne gt lt gte lte） |
| 時序 | `sequencing` | required_before（活動引用）；deadline；on_violation |
| 例外 | `exceptions` | overridden_by / overrides / conflicts_with / exception_of（rule_id 陣列） |
| 時間 | `temporal` | effective_from/to + supersedes/superseded_by |
| 證據 | `evidence_requirements[]` | 每一種必要證據：evidence_type、min_count、metadata 條件（角色/時間窗/文件版本）、`missing_evidence_result` |
| 判定 | `decision` | method（deterministic_rule_check/dmn_table/shacl_validation/petri_reachability/human_review）+ `missing_evidence_result`（insufficient_evidence / escalate / partial_pass_note）|
| 來源 | `source` | document_id + document_version + clause_refs（section/page/paragraph）+ verbatim_text + selectors |
| 產證 | `provenance` | origin（authoritative_text/derived_requirement/ai_extracted/human_draft）+ extraction（model_id/confidence/時間）+ review |
| 驗證 | `validation` | test_case_refs（指向 `eval` 案例）+ 上次執行結果 |

## 4. LegalRuleML v1.0 對照表（匯出指引）

> LegalRuleML Core Specification v1.0 已是 OASIS 正式標準（OASIS Standard，2021-08-30）；
> 授權版：<https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.html>
> 例庫：<https://github.com/oasis-tcs/legalruleml>
> ⚠️ 本表為**設計級對照**：v1.0 陳述構件的確切命名（Prescriptive Statements 的 Obligation/Prohibition/Permission、
> Substitution/Override/Repeal/Association 等）在首次匯出實作時，需逐一對照官方 Part 1B 核對，
> 不得憑本表直接寫 XML。

| 本 schema | LegalRuleML 概念 | 備註 |
| --- | --- | --- |
| `deontic.modality` | Obligation / Prohibition / Permission 陳述類型 | 加入 strength（mandatory/recommended/optional）的映射需在匯出層定義 |
| `condition` | 內部條件（if/then then-side 之 if 部分） | 對應其構成元素的邏輯部分 |
| `exceptions / overridden_by` | Substitution（例外替代）與/或 Override（優先權）關係 | 兩者的確切語義差異於匯出前對照官方定義後固定 |
| `temporal.supersedes / retired` | Repeal / Association（法律效力變更類陳述） | 廢止/取代鏈記錄 |
| `subject.scope.jurisdiction` | Jurisdiction（管轄/適用範圍上下文） | |
| `source.clause_refs / verbatim_text` | 對應法律文本的 URI 引用（Sources） | |
| `provenance` | Assoc（關聯）+ 自訂 metadata | LegalRuleML 不定義主管治理欄位，保留於本 schema |

**採用現成 OASIS 標準而非自創格式的理由**：外部審查者可用工具校驗語義；
自訂 JSONL 之外的正式匯出，使規則庫可與法律知識工程生態互通。

## 5. 與其他構件的分工

| 問題 | 由誰回答 | 用什麼 |
| --- | --- | --- |
| 這一步之前必須完成什麼 | 規則引擎 | `sequencing.required_before` |
| 這個轉移是否合法 | 流程模型 | BPMN/Petri（PM4Py conformance） |
| 兩條規則誰優先 | 衝突處理器 | `overrides` 鏈 + 優先序 key；無解→escalate |
| 這份證據足夠嗎 | 證據比對器 | `evidence_requirements` |
| 圖資料結構合規嗎 | SHACL | KG 約束（非時序/非因果） |

## 6. 版本記錄

| 版本 | 日期 | 修改 |
| --- | --- | --- |
| 0.1 | 2026-10-25 | 初版（依 Addendum A1 訂定） |
| 0.1.1 | 2026-10-25 | +`sequencing.loop_policy`（RULE-CAPA-012 需要；見 `rule-schema-v0.1.1.md`，機讀版 `rule-schema-v0.1.1.schema.json`） |

> **尾注（2026-10-25）**：v0.1 的規則層已以 Rust crate `ekp_platform/src/rules.rs` 落地——
> 非 active 規則載入即拒收（投毒防線）、v0.1.1 的 `loop_policy` 已接入引擎真機驗證
> （CAP-001 escalate §6.2 ／ CAP-002 comply §6.1，`ekp_platform --selftest` 49 assertions PASS）。
> 本檔 §1–§5 為 v0.1.1 的上位語義來源，仍有效。
