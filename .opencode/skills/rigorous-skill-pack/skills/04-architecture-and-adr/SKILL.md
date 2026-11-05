---
name: architecture-and-adr
description: 架構與 ADR，確定系統架構並記錄架構決策，對應 MASTER-WORKFLOW G4。
category: architecture
---

# 04-Architecture and ADR 架構與 ADR

## 用途

確定系統架構，記錄架構決策 (ADR)，確保架構符合工作區憲章不變量。

## 輸出

- 架構圖
- ADR (`templates/adr.md`)
- 元件關係圖

## 執行步驟

1. 確定系統架構與資料流
2. 記錄 ADR 決策與原因
3. 驗證架構符合工作區憲章不變量
4. 確認零 JavaScript 展示層（若適用）

## 品質閘門

- [ ] ADR 已記錄決策與原因
- [ ] 架構符合工作區憲章不變量
- [ ] 零 JavaScript 展示層（若適用）

## 停止條件

架構決策未經審查時，停止並審查。

## AI 自稱完成 vs 實際證據

僅接受 ADR 檔案存在且架構圖完整的實際證據，不接受 AI 僅說「架構已確定」。
