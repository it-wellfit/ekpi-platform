---
name: security-and-privacy
description: 安全與隱私，執行威脅模型與安全審查，驗證最小權限與無資料外洩，對應 MASTER-WORKFLOW G9。
category: security
---

# 10-Security and Privacy 安全與隱私

## 用途

執行威脅模型與安全審查，驗證最小權限原則，防止資料外洩與跨租戶授權問題。

## 輸出

- 威脅模型 (`templates/threat-model.md`)
- 安全審查結果 (`checklists/security-review.md`)
- 權限驗證記錄

## 執行步驟

1. 建立威脅模型
2. 驗證最小權限原則
3. 檢查資料外洩風險（提示注入、跨租戶授權等）
4. 記錄安全審查結果

## 品質閘門

- [ ] 威脅模型已完成
- [ ] 最小權限原則已驗證
- [ ] 無資料外洩風險

## 停止條件

安全審查未通過時，停止並修正。

## AI 自稱完成 vs 實際證據

僅接受威脅模型與安全審查結果檔案存在的實際證據，不接受 AI 僅說「安全已確保」。
