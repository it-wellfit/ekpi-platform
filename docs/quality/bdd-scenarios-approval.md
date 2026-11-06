# 核准工作流 BDD 場景規格 v1.0

- **狀態：** Active（S2 規格基準；實作以場景名稱對應測試，缺一不收）
- **Owner：** Product + QA + Engineering
- **最後審閱：** 2026-11-06
- **方法：** SDD（本規格先於實作）＋ BDD（每場景 = 一個具名測試）＋ DDD（聚合行為在 `crates/domain/src/workflow.rs`）＋ ADD（依賴方向 web→application→domain←adapters）
- **配套：** [再設計總計畫](../product/redesign-v2-master-plan.md)、[品質標準](quality-standards-v2.md)、[測試計畫](test-verification-plan-v2.md)

## 1. 領域語彙（Ubiquitous Language）

| 詞 | 定義 |
|---|---|
| 工作流定義（WorkflowDefinition） | 版本化簽核流程；含有序階段；建立後不可變，發布才可被任務引用 |
| 階段（Stage） | 簽核的一站；有 id、名稱、SLA 秒數、升級路徑、逾時動作 |
| 任務（ApprovalTask） | 一筆待簽核事項；綁定定義＋版本＋目前階段；狀態機：Pending → Approved/Rejected/Escalated/Expired/Cancelled |
| 升級（Escalate） | 任務停在同階段但標記升級對象並累計次數；不推進階段 |
| 逾時（Expire） | due_at 已過且未決策；由 cleanup 批次標記 Expired（終態） |
| 決策者（Approver） | 可核准/駁回/升級的角色；來自 session，不接受表單指定 |

## 2. 驗收場景（每場景對應具名測試）

### 組別 A：定義驗證（domain）

| # | 場景 | Given | When | Then | 測試 |
|---|---|---|---|---|---|
| A1 | 合法定義 | 一個含單階段、正 SLA 的定義 | validate | Ok | `valid_definition_passes_validation` |
| A2 | 空階段 | 定義無任何階段 | validate | Err「no stages」 | `definition_without_stages_is_invalid` |
| A3 | 重複階段 id | 兩個同 id 階段 | validate | Err「duplicate stage id」 | `duplicate_stage_ids_are_invalid` |
| A4 | 零 SLA | 某階段 sla=0 | validate | Err「zero sla」 | `zero_sla_is_invalid` |
| A5 | 升級指向未知階段 | escalation_path 含不存在 id | validate | Err「unknown stage」 | `escalation_to_unknown_stage_is_invalid` |

### 組別 B：任務生命週期（domain 聚合）

| # | 場景 | Given | When | Then | 測試 |
|---|---|---|---|---|---|
| B1 | 提交落在第一階段 | 已發布的兩階段定義 | submit_task | 任務 Pending、stage=第一階段、due=now+該階段 SLA | `submitting_task_places_it_in_first_stage_with_stage_sla` |
| B2 | 核准中間階段前進 | 任務在第 1/2 階段 | decide(Approve) | stage=第 2 階段、仍 Pending | `approving_mid_stage_advances_to_next_stage` |
| B3 | 核准最終階段 | 任務在最後階段 | decide(Approve) | state=Approved（終態） | `approving_final_stage_completes_task` |
| B4 | 駁回 | 任務 Pending | decide(Reject) | state=Rejected（終態），無論階段 | `rejecting_task_is_terminal_from_any_stage` |
| B5 | 升級 | 任務 Pending | decide(Escalate) | state=Escalated、count+1、escalated_to=該階段升級路徑首項 | `escalating_task_records_target_and_count` |
| B6 | 終態拒操作 | 任務已 Approved | 任何 decide | Err InvalidTransition | `deciding_terminal_task_is_refused` |
| B7 | 未發布定義不可提任務 | 定義 active=false | submit | Err | `submitting_task_against_unpublished_definition_is_refused` |
| B8 | 逾時判定 | now > due_at 且未終態 | is_overdue | true；已終態 false | `overdue_detection_respects_due_at_and_terminal_states` |
| B9 | cleanup 只清逾期未決 | 1 筆逾期 Pending、1 筆未逾期、1 筆已 Approved | cleanup_expired | 回傳 1；逾期者變 Expired；其他不動 | `cleanup_expires_only_overdue_open_tasks` |
| B10 | 版本綁定 | 任務引用 definition v1；store 只有 v2 | submit | Err（版本不存在） | `task_must_reference_existing_published_version` |

### 組別 C：儲存（JSONL 帳鏈 adapter）

| # | 場景 | Given | When | Then | 測試 |
|---|---|---|---|---|---|
| C1 | 重開後狀態還原 | 寫入定義+任務+決策 | open 同路徑 | 投影與關閉前一致 | `reopening_store_replays_identical_projection` |
| C2 | 竄改拒載 | 追加後手改檔案內容 | open | Err（hash mismatch），不部分載入 | `tampered_store_refuses_to_load` |
| C3 | 斷行拒載 | 檔案末行不完整 | open | Err「truncated final line」 | `torn_final_line_refuses_to_load` |
| C4 | 事件鏈可驗證 | 任意多次寫入 | verify | Ok | `event_chain_verifies_after_writes` |
| C5 | 獨佔鎖 | 已開啟的 store | 同路徑再 open | Err（鎖定中） | `second_open_on_same_path_is_refused` |

### 組別 D：工作台整合（HTTP，零 JS）

| # | 場景 | Given | When | Then | 測試 |
|---|---|---|---|---|---|
| D1 | 頁面呈現真實任務 | 已提交 1 筆任務 | Approver GET /approvals | 200；頁面含任務 id 與階段；鏈完整性徽章 | `approvals_page_renders_submitted_task` |
| D2 | 提交動作寫入 store | Editor 登入 | POST /approvals/submit（含 CSRF cookie） | 303；store 出現任務；actor=session 使用者 | `submitting_via_web_persists_with_session_actor` |
| D3 | 表單 actor 不被信任 | Editor 登入；表單夾帶 actor=Jordan | POST /approvals/submit | store 中 actor=登入者，非 Jordan | `form_supplied_actor_is_ignored` |
| D4 | 決策需 Approver | Editor 登入 | POST /approvals/{id}/decide | 403；store 無變化 | `editor_cannot_decide_tasks` |
| D5 | 決策推進階段 | Approver 登入；任務在第 1 階段 | POST decide approve | 303；任務進第 2 階段 | `approver_decision_advances_stage` |
| D6 | 未登入不可讀 | 無 session | GET /approvals | 401 | `approvals_page_requires_session` |
| D7 | API 未接線仍誠實 | 已登入 | GET /api/v1/approvals | 501（API 契約屬 S4） | 既有 `unwired_api_endpoints_return_honest_501_never_fabricated_data` |

## 3. 明確非目標（本切片）

- `/api/v1/*` 維持 501（API 契約於 S4 以 SDD 定義後才實作）。
- 不做 PostgreSQL 實作（骨架維持 fail-closed）。
- 不做多層條件分支 UI（ConditionExpr 已在 domain 支援，UI 於後續切片）。
- 不修 `jsonl_ledger.rs` 歷史編碼缺陷（IMP-019 另案追蹤）。
