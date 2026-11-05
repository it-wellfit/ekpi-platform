# AI 時代嚴謹軟體工程技能包 — AGENTS.md

本檔為技能包自身的 AGENTS。它僅適用於 `.kilo/skills/rigorous-skill-pack/` 內容，**不覆蓋**任何專案的 `AGENTS.md`，也不取代子專案專屬規範。

---

## 原則

1. **不覆蓋專案規則**：安裝時僅新增參考連結，不直接覆寫現有 `AGENTS.md` 或專案規則。
2. **證據優先於自稱**：AI 代理人自稱「已完成」不等於完成。每個任務必須有可觀察的執行證據（檔案存在、命令退出碼為 0、測試通過、驗證腳本 PASS）。
3. **停止條件優先**：當遇到風險分級為「高」或停止條件觸發時，必須停止並升級，不得繼續執行。
4. **最小權限**：資料與工具只取得完成任務所需的最小權限，禁止過度授權。
5. **來源授權**：上游技能內容未整包複製，來源與授權注意事項記錄於 `docs/source-registry.md`。

---

## 15 個技能與用途

| 技能 | 路徑 | 主要用途 |
|------|------|----------|
| 00-orchestration | `skills/00-orchestration/SKILL.md` | 總控層，技能路由與停止條件判定 |
| 01-product-discovery | `skills/01-product-discovery/SKILL.md` | 產品探索 |
| 02-requirements-specification | `skills/02-requirements-specification/SKILL.md` | 需求與領域建模 |
| 03-ux-and-accessibility | `skills/03-ux-and-accessibility/SKILL.md` | UX 與無障礙 |
| 04-architecture-and-adr | `skills/04-architecture-and-adr/SKILL.md` | 架構與 ADR |
| 05-task-planning | `skills/05-task-planning/SKILL.md` | 任務規劃 |
| 06-tdd-and-implementation | `skills/06-tdd-and-implementation/SKILL.md` | TDD 與實作 |
| 07-systematic-debugging | `skills/07-systematic-debugging/SKILL.md` | 系統化除錯 |
| 08-code-review | `skills/08-code-review/SKILL.md` | 程式碼審查 |
| 09-testing-and-quality | `skills/09-testing-and-quality/SKILL.md` | 品質測試 |
| 10-security-and-privacy | `skills/10-security-and-privacy/SKILL.md` | 安全與隱私 |
| 11-ai-evaluation-and-governance | `skills/11-ai-evaluation-and-governance/SKILL.md` | AI 評估與治理 |
| 12-ci-cd-and-release | `skills/12-ci-cd-and-release/SKILL.md` | CI/CD 與發布 |
| 13-observability-and-reliability | `skills/13-observability-and-reliability/SKILL.md` | 可觀測性與可靠性 |
| 14-retrospective-and-evolution | `skills/14-retrospective-and-evolution/SKILL.md` | 回顧與演進 |

---

## 評估案例

`evals/workflow-evals/cases.json` 包含 12 個情境：
- 產品探索
- 跨租戶資料外洩
- 正式環境刪除
- 盲目修 Bug
- AI 自動付款
- 虛報測試通過
- 以及其他 6 個情境

每個案例都必須以「實際證據」驗證，不僅接受 AI 自稱。

---

## 驗證

執行 `scripts/validate_pack.py` 檢查套件結構完整性。

---

## 回滾

移除 `.kilo/skills/rigorous-skill-pack/` 即可卸載本技能包；不影響其他 `.kilo/skills/` 技能或專案 `AGENTS.md`。
