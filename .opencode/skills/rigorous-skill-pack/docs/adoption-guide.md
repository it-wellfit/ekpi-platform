# 採用指南 (Adoption Guide)

**版本**: v1.0.0  
**日期**: 2026-10-26

---

## 如何開始使用

1. 確認 `.kilo/skills/rigorous-skill-pack/` 已安裝。
2. 先閱讀 `README.md`、`AGENTS.md` 和 `MASTER-WORKFLOW.md`。
3. 將需要的技能整合進既有專案，**不要直接覆蓋原本的 `AGENTS.md` 或專案規則**。
4. 對新功能按「產品探索 → 需求規格 → 架構 → 任務規劃 → TDD → 審查 → 驗收」順序執行；對 Bug 則優先使用系統化除錯流程。
5. 執行 `scripts/validate_pack.py`，並用 `evals/workflow-evals/cases.json` 檢查技能行為。

## 整合方式 (工作區層級)

- 在 `WORKSPACE_CHARTER.md` 第 7 節新增技能包記錄。
- 在各子專案 `AGENTS.md` 新增「技能參考」小節，僅以連結方式指向 `.kilo/skills/rigorous-skill-pack/`，**不直接覆寫**任何現有 `AGENTS.md` 或專案規則。

## 不要做的事

- 不直接覆蓋專案的 `AGENTS.md`
- 不接受僅由 AI 自稱完成而無證據的情況
- 不跳過品質閘門與停止條件
