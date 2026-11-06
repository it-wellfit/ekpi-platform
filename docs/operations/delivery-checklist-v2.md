# 交付清單 v2.0（Delivery Checklist）

- **狀態：** Active
- **Owner：** Engineering + QA + Operations
- **最後審閱：** 2026-11-06
- **配套：** [再設計總計畫](../product/redesign-v2-master-plan.md)、[測試驗證計畫](../quality/test-verification-plan-v2.md)、[release evidence bundle](release-evidence-bundle.md)

> 使用方式：每個交付切片結束時逐項勾選；「證據」欄必須填**已存在**的路徑或指令輸出，缺者該項視為未交付。

## 1. 文件

| # | 交付物 | 位置 | 驗收 |
|---|---|---|---|
| D-01 | 產品再設計總計畫（架構/模組/介面） | [docs/product/redesign-v2-master-plan.md](../product/redesign-v2-master-plan.md) | 已審，與 governance 規格無衝突 |
| D-02 | 品質標準與驗收準則 | [docs/quality/quality-standards-v2.md](../quality/quality-standards-v2.md) | 指標皆可量測 |
| D-03 | 測試驗證計畫 | [docs/quality/test-verification-plan-v2.md](../quality/test-verification-plan-v2.md) | 層級/工具/時點齊 |
| D-04 | 交付清單（本文件） | [docs/operations/delivery-checklist-v2.md](delivery-checklist-v2.md) | 逐項可勾 |
| D-05 | ADR：整合邊界（0002/0003） | `docs/adr/` | 收斂後補實作證據 |
| D-06 | 部署與維運指南 | `docs/deployment/`、`docs/operations/` | 含啟停/備份/還原/回退 |
| D-07 | 操作手冊（本機工作台） | `docs/operations/` | 新操作員可照做完成一次全流程 |
| D-08 | 已知限制與誠實邊界 | `docs/quality/phase-gates.md`、`docs/product/README.md` | 不含未驗證宣稱 |

## 2. 程式碼

| # | 交付物 | 驗收 |
|---|---|---|
| C-01 | 確定性核心（platform） | selftest 49、schema 20、eval gate 28 |
| C-02 | 治理工作台（governance_web） | 25 route 測試、CSRF/RBAC/稽核鏈全綠 |
| C-03 | 整合層（application/adapters：approval、channels、postgres） | WIP 收斂，紅→綠 6+6 條 |
| C-04 | 呈現層（apps/web 9 頁面） | e2e 每頁 1 條、零 `<script>` |
| C-05 | 評測（eval） | 12 測試 + 10 selftest 檢查 |
| C-06 | 遷移（migrations/） | 對 postgres adapter 整合測試通過 |

## 3. 測試報告

| # | 交付物 | 位置 | 驗收 |
|---|---|---|---|
| T-01 | Canonical 執行記錄 | register 證據列（指令+修訂+結果） | 最新修訂有兩次同指令 exit 0 |
| T-02 | 引擎硬門檻 | `out/engine_hard_gates.json` | H-1…H-6 全 OK |
| T-03 | 檢索評測 | `--eval` 輸出 | 28/28、isolation 1.00 |
| T-04 | 效能基線 | `artifacts/benchmark/*.json` | 有前/後對照 |
| T-05 | 還原演練 | `docs/operations/backup-log.md` | 最新 bundle PASS + SHA-256 |
| T-06 | 七狀態審計表 | `docs/quality/ui-state-audit.md`（新增時） | 每頁簽核 |
| T-07 | E2E 報告 | `platform/tests/e2e/` 執行輸出 | 主流程全綠 |

## 4. 部署與維運

| # | 交付物 | 驗收 |
|---|---|---|
| O-01 | 本機啟動指引（workbench + CLI） | 從 clone 到登入 < 10 分鐘，照文件即可 |
| O-02 | 備份與還原程序 | `backup.ps1` + `backup-log.md` 最新列 PASS |
| O-03 | 健康探針與指標 | `/healthz`、`/ready`、`/metrics` 文件化 |
| O-04 | 回退程序 | bundle → clone → 驗證 三步文件化 |
| O-05 | 維運邊界 | 明列：本機範疇、無企業身分、Phase B 未決項 |

## 5. 發布判定

全部 D/C/T/O 項存在且有證據 → 該切片可宣告 `Verified (local)`。
任何一項缺證據 → 切片退回；不得以「整體看起來完成」代替逐項核對。
