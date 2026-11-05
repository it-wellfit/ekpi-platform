---
name: tdd-and-implementation
description: TDD 與實作，先寫測試再實作，保持 100% 測試通過與零警告，對應 MASTER-WORKFLOW G6。
category: implementation
---

# 06-TDD and Implementation TDD 與實作

## 用途

採用測試驅動開發，先寫測試再實作，保持 100% 測試通過與零警告。

## 輸出

- 程式碼
- 測試
- 實作記錄

## 執行步驟

1. 寫失敗的測試（紅）
2. 寫最少程式碼使測試通過（綠）
3. 重構並保持測試通過
4. 執行 `cargo clippy --all-targets -- -D warnings`（Rust 專案）
5. 確認 `#![forbid(unsafe_code)]`（Rust 專案）

## 品質閘門

- [ ] 測試先行（紅綠重構）
- [ ] 零警告
- [ ] `#![forbid(unsafe_code)]`（Rust 專案）

## 停止條件

測試未通過或有警告時，停止並修正。

## AI 自稱完成 vs 實際證據

僅接受測試通過報告與零警告輸出的實際證據，不接受 AI 僅說「實作已完成」。
