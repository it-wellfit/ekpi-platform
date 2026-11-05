# AI 時代嚴謹軟體工程技能包 (Rigorous AI Engineering Skill Pack)

**版本**: v1.0.0  
**語言**: 繁體中文優先  
**適用**: 工作區層級，供 AI 編碼代理逐步執行  
**安裝位置**: `.kilo/skills/rigorous-skill-pack/`  
**評估案例**: `evals/workflow-evals/cases.json` (12 個)

---

## 這是什麼

本技能包是一份可下載、可放進專案、可由 AI 編碼代理逐步執行的技能集合。它不是超長提示詞，也不是僅有的目錄規劃。每個技能皆為獨立載入的 `SKILL.md`，具備 `name` 與 `description` frontmatter，可由 AI 代理按需呼叫。

技能包整合了九個核心領域與一個總控層，並提供明確的品質閘門與能檢查技能本身是否有效的評估案例。

---

## 包含什麼

| 類別 | 檔案 / 目錄 | 說明 |
|------|-------------|------|
| 總控層 | `MASTER-WORKFLOW.md` | G0–G10 端到端工程總控流程，含風險分級與停止條件 |
| 技能包規則 | `AGENTS.md` | 技能包自身的 AGENTS，不覆蓋專案 AGENTS.md |
| 15 個技能 | `skills/00-orchestration/` … `skills/14-retrospective-and-evolution/` | 可獨立呼叫的技能，每個含 `SKILL.md` |
| 範本 | `templates/` | 產品簡報、PRD、ADR、實施計畫、測試計畫、威脅模型、發布檢查、回顧 |
| 檢查表 | `checklists/` | Definition of Done、階段品質關卡、安全審查、事件復原 |
| 評估 | `evals/workflow-evals/cases.json` | 12 個情境，檢查技能是否正確路由及遵守停止規則 |
| 驗證腳本 | `scripts/validate_pack.py` | 套件結構與 ZIP 完整性檢查 |
| 來源登記 | `docs/source-registry.md` | 上游來源與授權注意事項 |

---

## 九個核心領域與總控層

1. `00-orchestration` — 總控層，負責技能路由與停止條件判定
2. `01-product-discovery` — 產品探索
3. `02-requirements-specification` — 需求與領域建模
4. `03-ux-and-accessibility` — UX 與無障礙
5. `04-architecture-and-adr` — 架構與 ADR
6. `05-task-planning` — 任務規劃
7. `06-tdd-and-implementation` — TDD 與實作
8. `07-systematic-debugging` — 系統化除錯
9. `08-code-review` — 程式碼審查
10. `09-testing-and-quality` — 品質測試
11. `10-security-and-privacy` — 安全與隱私
12. `11-ai-evaluation-and-governance` — AI 評估與治理
13. `12-ci-cd-and-release` — CI/CD 與發布
14. `13-observability-and-reliability` — 可觀測性與可靠性
15. `14-retrospective-and-evolution` — 回顧與演進

---

## 如何開始使用

1. 確認 `.kilo/skills/rigorous-skill-pack/` 已安裝。
2. 先閱讀 `README.md`、`AGENTS.md` 和 `MASTER-WORKFLOW.md`。
3. 將需要的技能整合進既有專案，**不要直接覆蓋原本的 `AGENTS.md` 或專案規則**。
4. 對新功能按「產品探索 → 需求規格 → 架構 → 任務規劃 → TDD → 審查 → 驗收」順序執行；對 Bug 則優先使用系統化除錯流程。
5. 執行 `scripts/validate_pack.py`，並用 `evals/workflow-evals/cases.json` 檢查技能行為。

---

## AI 自稱完成 vs 實際證據

- **AI 自稱完成**：代理人表示「已完成」，但無可觀察證據。
- **實際證據證明完成**：檔案存在、命令退出碼為 0、測試通過、驗證腳本回報 PASS、SHA-256 比對一致。
- 所有評估案例必須以「實際證據」為準，不僅接受 AI 的自稱。

---

## 來源與授權

請參閱 `docs/source-registry.md`。本技能包參考 Matt Pocock Skills、Superpowers、GitHub Spec Kit、BMAD-METHOD、NIST SSDF 與 OWASP ASVS；上游技能內容未整包複製，來源與授權注意事項已記錄在 `docs/source-registry.md`。

---

## 限制

本版為 v1.0.0 已通過壓縮檔完整性與基本結構檢查的基礎版。不代表 12 個評估案例已在所有 AI 模型與 IDE 上實際跑完，也不代表已通過完整安全稽核或實際產品驗收。請針對你的開發環境和實際專案持續測試、修正與演進。
