# 流程與事件驗證層：技術選型與設計

> 配套：Master Prompt v1 §3.5/§5/§七 + Addendum A3/A4/A5/A6。
> 本文件把「流程驗證」從能力清單落成可實作的技術栈。

## 1. 三層模型（不得混寫）

| 層 | 表示 | 回答的問題 |
| --- | --- | --- |
| 規範流程 | BPMN（撰寫）＋ DMN（決策表）+ 必要規則 | 「規定該怎麼走」 |
| 系統授權流程 | Petri net / 工作流（由 BPMN 轉譯） | 「此刻允許什麼轉移」 |
| 實際事件 | XES / OCEL 日誌（observed / assumed / unconfirmed 三態） | 「實際發生了什麼（及其證據）」 |

審計判定 = 1+2 與 3 的**符合性檢查**，絕不以生成模型的自我敘述替代。

## 2. 工具鏈選型（全部已核實存在）

| 元件 | 選擇 | 核實來源 | 用法 |
| --- | --- | --- | --- |
| 執行/驗證核心 | **PM4Py**（開源 process mining） | `pm4py.org`（官方站；2026-10-25 verified） | BPMN↔Petri 轉換、token-based replay、alignments conformance、soundness 檢查。BPMN 匯入等 API 細節於 R2-3 實作時以執行驗證 |
| 撰寫層 | BPMN 2.0 + DMN | OMG 官方頁（v2.0.2 / v1.7 beta 已核實；採用 DMN 前確認最新正式版） | 業務人員可讀可修；引擎僅消費轉譯產物 |
| 日誌格式 | **XES**（IEEE 1849-2023）為主 | standards.ieee.org（2016 頁標 superseded；2023 版 DOI 10.1109/IEEESTD.2023.10267858） | 事件日誌匯入/匯出骨架 |
| 多實體日誌 | **OCEL 2.0** | ocel-standard.org（2026-10-25 verified） | 一案件同時牽涉申請、文件、表單、簽核多實體時使用 |
| 規範陳述 | LegalRuleML v1.0 | OASIS docs（OASIS Standard 2021-08-30 verified） | 規則字典匯出/外部審查 |

**選擇理由**：conformance checking 是此問題學界正解（Process Science 2025 綜述已登錄 `ai-conformance-survey-2025`）；
PM4Py 是該領域事實上的開源基座；BPMN/DMN 保證業務可維護；XES/OCEL 保證日志格式不鎖死於任何供應商。

**堆疊更新（2026-10-25，tech-selection §0）**：平台主軸改為 Rust 全端。MVP 的 conformance v0
採 **Rust 原生 token-based replay + 可達性檢查**（足以支撐硬門檻 H-1/H-2 的禁改放行審核）；
PM4Py 降為選配 sidecar，僅於 R3 作 alignments 對照 oracle；兩者以雜湊過的 XES/OCEL 工件交换，
比對差異登錄為報告附件而非直接混入判定。

## 3. 事件日誌品質關卡（GR-1…GR-6，導入後、判定前）

| 關卡 | 檢查 | 未過處置 |
| --- | --- | --- |
| GR-1 | 實體解析：每筆事件能否無歧義歸屬案件（case_id ↔ 文件 ↔ 表單） | 標記待解析案件，禁止判定 |
| GR-2 | 亂序：時間戳順序違反 process 片段因果 | 列為資料品質缺失；不判定「流程違規」 |
| GR-3 | 重複事件偵測（冪等鍵） | 去重報告 + 原始保留 |
| GR-4 | 時鐘偏差：跨系統時間來源與 offset 登記 | 以 event「時間來源」欄位校正 |
| GR-5 | 完整性：流程模型中該產生日志的活動是否都有事件類型 | 產出缺口報告（不視為違規證據） |
| GR-6 | 角色映射：執行者 ↔ role 目錄 | 標記 unconfirmed |

GR 未過關的日誌只能進「資料品質通道」，不能產生合規判定。

## 4. 符合性檢查輸出分類

| 類別 | 定義 | 必附 |
| --- | --- | --- |
| 符合 | 觀察路徑在授權流程內且必要證據在位 | 路徑 + 證據引用 |
| 偏差 | 觀察到禁止性轉移 / 缺必要活動 / 順序錯誤 | 違反的模型元素 + 證據引用 |
| 缺失 | 預期活動無 observed 事件 | 缺口（不是違規：**缺紀錄 ≠ 未發生**） |
| 無法確認 | 證據不足以重建相應片段 | unconfirmed 說明 + 升級建議 |

## 5. 循環控制設計

每個循環節點（如 CAPA 的 REWORK↔VERIFY）必須定義：

- 進入/退出條件（以 predicate 表述）。
- 重試上限（次數或時間檔，設定參數非硬編碼）。
- 超限行為：`escalate`（人力決策），**絕不**無限回退。
- 冪等守門：同一 retry token 的重複確認不再觸發副作用。

## 6. Decision Snapshot（Addendum A6）

```json
{
  "decision_id": "D-2026-1042",
  "inputs": {
    "case_id": "CHG-0087",
    "doc_versions": { "SOP-CHANGE": "3.2", "QM-01": "2.1" },
    "rules": ["RULE-CHG-001@2.0", "RULE-EX-007@1.1"],
    "retrieved_chunks_sha256": ["..."],
    "events_sha256": "log_excerpt_hash",
    "model_id": "...", "prompt_template_sha256": "...",
    "config_sha256": "..."
  },
  "llm_artifacts": { "raw_answers_sha256": ["..."] },
  "verdict": "insufficient_evidence",
  "computed_by": "deterministic_engine_v1"
}
```

重放規則：仅重執行確定性路徑（規則引擎 + conformance + 證據比對），LLM 輸出以雜湊比對當時紀錄，
**不重叫模型**。兩次重放 must 位元組一致（排除時間戳欄位）。

## 7. 拒答與風險–覆蓋校準（A5）

- 輸出型別：`comply / violate / insufficient_evidence / escalate`。
- 校準指標（接 `eval/ekp_eval/metrics.py`）：
  1. 決定性案件（必須給判定者）上的**無故拒答率**；
  2. 缺證案件上的**正確拒絕率**；
  3. risk–coverage 曲線：依 confidence 排序，計算各覆蓋率下的誤差；
  4. 成本矩陣：漏報權重 W_FN > 誤報權重 W_FP（業務聲明，登錄版本）。
- 閾值治理：`abstain_threshold` 為設定參數，修改需登記版本與核准人。

## 8. 反事實模擬（v1 §四之 2）的邊界

- 「若跳過簽核，哪些後續不再合法」→ Petri net 可達性 / 視阻擋分析（程序問題，可判定）。
- 「若跳過簽核，缺陷率是否上升」→ 因果問題：需假設宣告、資料、識別策略；不足則 `cannot_identify`。
- 平台 UI 必須把兩類答案分開陳列，不得混在單一結論區。

## 9. 版本記錄

| 版本 | 日期 | 修改 |
| --- | --- | --- |
| 1.0 | 2026-10-25 | 初版（Addendum A3/A4/A5/A6 落地） |
