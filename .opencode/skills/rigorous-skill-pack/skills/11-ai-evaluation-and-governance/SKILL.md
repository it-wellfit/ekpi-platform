---
name: ai-evaluation-and-governance
description: AI 評估與治理，評估 AI 代理行為與治理合規，防止 AI 自動付款與虛報測試通過，對應 MASTER-WORKFLOW G9。
category: governance
---

# 11-AI Evaluation and Governance AI 評估與治理

## 用途

評估 AI 代理行為與治理合規，防止 AI 自動付款、虛報測試通過等風險。

## 輸出

- AI 評估報告
- 治理合規記錄
- 停止條件觸發記錄

## 執行步驟

1. 評估 AI 代理行為是否符合治理規範
2. 防止 AI 自動付款（需人工核准）
3. 檢測虛報測試通過
4. 記錄評估與治理結果

## 品質閘門

- [ ] AI 評估報告已完成
- [ ] 治理合規已驗證
- [ ] 停止條件觸發已記錄

## 停止條件

AI 行為違反治理規範時，停止並升級。

## AI 自稱完成 vs 實際證據

僅接受 AI 評估報告與治理合規記錄的實際證據，不接受 AI 僅說「已符合治理」。
