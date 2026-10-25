# 規則模型 v0.1.1 設計說明

> 配套：`rule-schema-v0.1.1.schema.json`（機讀版）。v0.1.1 = v0.1 **僅加一件事**：
> `sequencing.loop_policy`（循環控制）。其餘欄位、生命週期、LegalRuleML 對照均沿 v0.1
> （見 `rule-schema-v0.1.md` §1–§5，仍有效）。

## 1. 為什麼需要 v0.1.1

v0.1 的 `sequencing` 只表示線性前置（`required_before` / `forbidden_after` / `deadline`）。
種子規則 **RULE-CAPA-012**（CAPA 矯正驗證循環）暴露缺口：上限外的回退是
「升級人工」而非「違規」，且引用的不是同一位條款——

- 重試次數 `≥ failures_cap` 時：`escalate`，引用「超限條款」；測試案例 CAP-001 引 `§6.2`。
- 重試次數 `< failures_cap` 時：合法回退 `comply`，引用「首輪條款」；CAP-002 引 `§6.1`。

若把超限行為硬編碼在引擎，違反工作區「閾值皆為可設定參數」公理；把兩種引用寫死在
引擎則丟失了條款溯源。因此升版加入結構化 `loop_policy`。

## 2. 新增欄位：`sequencing.loop_policy`

| 子欄位 | 型別 | 必填 | 說明 |
| --- | --- | --- | --- |
| `failures_cap` | integer ≥ 1 | ✔ | 觸發升級前的最大允許失敗次數（測試案例綁 `loop.retries` 欄位） |
| `on_exceeded` | escalate / block / warn | ✔ | 達上限時的處置（MVP 引擎僅支援 `escalate`，其值待 R2-3 擴充） |
| `doc_ref` | string | ✔ | 超限行為的條款引用（§ 形式），判定時逐字輸出 |
| `first_iteration_ref` | string（可 null） | 可 | 首輪合法回退的條款引用；null 時上限內判定僅輸出規則引用 |

使用規則：

1. `loop_policy` 連同 `rule_id@rule_version` 一起綁定文件版本；案件引用時產出
   `{doc_id}@{doc_ver} §{sec}`（超限用 `doc_ref`、合規用 `first_iteration_ref` 或主條款）。
2. 判定輸入的 `loop.retries`（案件事件層）與 `failures_cap` 比較；
   `retries >= cap` → 依 `on_exceeded` 輸出並引用 `doc_ref`；`retries < cap` → 走一般合規路徑。
3. `failures_cap` 的修改需走規則生命週期重審（v0.1 §2），不是配置熱改。

## 3. 案例對照（語義驗收）

| 案件 | loop.retries | failures_cap | 期望判定 | 引用 |
| --- | --- | --- | --- | --- |
| CAP-001 | 3 | 3 | escalate | SOP-CAPA@1.4 §6.2（`doc_ref`） |
| CAP-002 | 1 | 3 | comply | SOP-CAPA@1.4 §6.1（`first_iteration_ref`） |

## 4. 版本記錄

| 版本 | 日期 | 修改 |
| --- | --- | --- |
| 0.1 | 2026-10-25 | 初版（依 Addendum A1 訂定） |
| 0.1.1 | 2026-10-25 | +`sequencing.loop_policy`（RULE-CAPA-012 需要；CAP-001/CAP-002 語義） |
