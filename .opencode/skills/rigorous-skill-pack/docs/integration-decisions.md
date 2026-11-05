# 整合決策 (Integration Decisions)

**版本**: v1.0.0  
**日期**: 2026-10-26

---

## 決策 1: 安裝位置

安裝於 `.kilo/skills/rigorous-skill-pack/`，工作區層級，三子專案 (ekpi-platform、Wellfit、SyncMesh) 共用。

## 決策 2: 不覆蓋現有 AGENTS.md

僅新增參考連結，不直接覆寫任何現有 `AGENTS.md` 或專案規則。

## 決策 3: 與現有技能並存

現有 20 個 `.kilo/skills/` 技能與 27 個 `.opencode/skills/` (Matt Pocock) 技能不受影響，與本技能包並存。

## 決策 4: 技能名稱重複處理

若 15 個新技能名稱與現有技能重複，保留兩者並由路由規則分辨；總控層 (00-orchestration) 負責路由。

## 決策 5: 評估案例執行

12 個評估案例必須以「實際證據」驗證，不僅接受 AI 自稱。執行結果記錄至 `eval-results/`。
