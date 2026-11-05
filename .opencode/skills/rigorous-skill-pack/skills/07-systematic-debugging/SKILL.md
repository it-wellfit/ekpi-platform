---
name: systematic-debugging
description: 系統化除錯，對 Bug 優先使用系統化除錯流程，不盲目修改，對應 MASTER-WORKFLOW G7。
category: debugging
---

# 07-Systematic Debugging 系統化除錯

## 用途

對 Bug 優先使用系統化除錯流程，確認根因後再修正，避免盲目修 Bug。

## 輸出

- 根因分析
- 除錯記錄
- 修正程式碼

## 執行步驟

1. 重現問題
2. 收集證據（日誌、截圖、測試失敗）
3. 確認根因
4. 撰寫對應測試防止回歸
5. 修正程式碼並驗證測試通過

## 品質閘門

- [ ] 根因已確認
- [ ] 修正有對應測試
- [ ] 回歸測試通過

## 停止條件

根因不明時，停止並 escalate。

## AI 自稱完成 vs 實際證據

僅接受根因分析記錄與測試通過的實際證據，不接受 AI 僅說「Bug 已修好」。
