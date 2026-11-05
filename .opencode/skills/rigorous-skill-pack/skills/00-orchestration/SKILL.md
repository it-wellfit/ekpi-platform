---
name: orchestration
description: 總控層，負責技能路由與停止條件判定，協調其他 14 個技能的執行順序。
category: control
---

# 00-Orchestration 總控層

## 用途

總控層負責根據 MASTER-WORKFLOW 的 G0–G10 階段，將任務路由到正確的技能，並判定是否觸發停止條件。它是所有技能的入口與調度中心。

## 路由規則

| 階段 | 對應技能 | 觸發條件 |
|------|----------|----------|
| G0 | 00-orchestration | 專案盤點與風險分級 |
| G1 | 01-product-discovery | 產品探索 |
| G2 | 02-requirements-specification | 需求與領域建模 |
| G3 | 03-ux-and-accessibility | UX 與無障礙 |
| G4 | 04-architecture-and-adr | 架構與 ADR |
| G5 | 05-task-planning | 任務規劃 |
| G6 | 06-tdd-and-implementation | TDD 與實作 |
| G7 | 07-systematic-debugging | 系統化除錯 |
| G8 | 08-code-review | 程式碼審查 |
| G9 | 09-testing-and-quality、10-security-and-privacy | 品質測試與安全 |
| G10 | 12-ci-cd-and-release、13-observability-and-reliability、14-retrospective-and-evolution | CI/CD 與發布 |

## 停止條件判定

當符合以下任一條件，必須停止執行並升級：

- 風險分級為「高」且缺乏安全技能支援
- 價值假設無法驗證
- 需求歧義未消除
- 無障礙檢查未通過
- 架構決策未經審查
- 工單過大或依賴環狀
- 測試未通過或有警告
- 根因不明
- 審查未通過
- 安全審查未通過
- 部署回滾測試未通過

## AI 自稱完成 vs 實際證據

總控層不接受僅由 AI 自稱完成而無證據的情況。每個階段完成必須有可觀察證據。

## 呼叫方式

由 AI 代理人根據當前階段呼叫對應技能，總控層記錄路由結果與停止條件判定。
