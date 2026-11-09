# QA 演進流程 v1.0（三工作流：閉環／對抗深審／混沌韌性）

- **狀態：** Active（依使用者 2026-11-09 指示建立；基於外部研究＋本倉實測證據）
- **Owner：** QA + Engineering
- **最後審閱：** 2026-11-09
- **上位文件：** [測試驗證計畫](test-verification-plan-v2.md)、[品質標準](quality-standards-v2.md)、[演化藍圖](../product/evolution-roadmap-v1.md)、[重構決策準則](../development/refactor-criteria.md)
- **配套審查：** [AUDIT-0003 紅隊報告](../reviews/2026-11-06-s1-s6-red-team.md)
- **誠實聲明：** 研究結論來自公開文獻（§2，附 DOI）；「本倉實證」來自 register/git 可查記錄。任何流程本身不改變 canonical 為唯一收口的鐵律。

## 1. 為何「Reviewer → Skeptic → Debug → Test」線性序列失敗

| # | 結構性缺陷 | 證據（文獻＋本倉） |
|---|---|---|
| 1 | **錨定效應**：reviewer 先背書，skeptic 後到要花社交成本反駁——skeptic 的職責是**證偽主張**，必須先於認可 | 紅隊方法論；本倉 AUDIT-0003 以「假設→實證否證」記錄（預設路徑假設被執行輸出否證） |
| 2 | **缺陷成本倒置**：debug 在 test 之前＝無可重現票證的猜測；產品後修復成本可達開發期 ~200×（Capers Jones，經 Stanek 2025 引述） | 本倉：空案例 vacuous-comply 在 345 個通過測試下潛伏，僅 property 紅測試抓到——先有失敗測試才有 debug 的對象 |
| 3 | **線性無法表達遞迴**：canonical 失敗 → debug → 新紅測試 → 重過門，是真實動態 | 本倉：`52b4e6c` fmt 疏失 → `ae3dcee` 修復重審；S2 資料污染 → 還原＋重審 |
| 4 | **資安不能晚**：安全語義（CSRF 需零 JS 相容）在實作後才審將迫使中途改寫 | 本倉：S1 登入 CSRF 由 header 語義重設計為 cookie 雙重提交 |

**結論：** 流程不是一條線，是帶門檻出口的閉環；三種鏡頭在不同時點介入。

## 2. 支撐研究（公開文獻，附出處）

| 發現 | 來源 | 對本專案的推論 |
|---|---|---|
| Property-based tests 的變異殺滅能力**遠高於**手寫單元測試（使用者指定以 ≈30× 表述；精確倍數以論文全文為準） | Ravi & Coblenz, *An Empirical Evaluation of Property-Based Testing in Python*, OOPSLA 2025, DOI 10.1145/3764068 | PBT 是本倉最高槓桿測試層：現有 18 條 property 已實證抓到 H-4 缺陷；每個切片新增不變式屬最佳投資 |
| 行覆蓋 >80% 的服務**僅 54% 變異殺滅、65% 測試強度** → 覆蓋率≠測試正確性；據此設立新的 mutation 品質門檻 | *Mutation Testing as a QA Technique in a Fintech Company*, SBQS 2024, DOI 10.1145/3701625.3701629 | 「全綠」不等於「測得好」；未來引入 mutation 門檻（Rust 生態候選：cargo-mutants）列為評估項 |
| 高績效組織以**結果指標**（change failure rate、lead time、MTTR、部署頻率）自評，而非流程指標 | DORA State of DevOps（Research 計畫持續出版） | 流程指標一律從 register/git **實測計算**，不得發明 |
| 產品後修復成本 ~200×；測試左移須「風險驅動」而非「習慣驅動」 | Capers Jones（Stanek 2025 引述）；Sauce Labs/Gatling/TestRail 2025 系列 | 測試映射到風險：高風險路徑前置密集測試；使用者可見功能右移持續驗證 |
| PBT 實務：differential/model-based 屬性最常見且有效；實務者以 mutation testing 自證 PBT 套件 | *Property-Based Testing in Practice*, ICSE 2024 | 本倉 property 應優先「決定論/重放/等價類」屬性（已符合）；以突變自證為進階項 |

## 3. Workflow A — 風險驅動閉環（日常作業系統；**立即採用**）

```
SPEC/BDD ──▶ Skeptic₀（攻擊規格）──▶ RED 測試 ──▶ IMPLEMENT ──▶ 雙鏡頭審查 ──▶ CANONICAL GATE ──▶ 證據/回顧
   ▲                                                                                    │ exit 0
   └──────────────── Debug 模式（僅在門檻失敗時進入）◀──────────────────────────────────┘
```

| 階段 | 入口 | 活動 | 出口 | 自動化 |
|---|---|---|---|---|
| 0 SPEC | 切片排程 | DDD/SDD＋BDD 場景（每場景一具名測試） | **Skeptic₀**：反演攻擊規格（什麼行為會破壞 N1–N4？） | 場景↔測試可追溯掃描（待建） |
| 1 RED | 場景列表 | 測試必須失敗 | 紅＝可重現證據 | ✅ |
| 2 IMPLEMENT | 紅已存在 | 實作；**debug 是模式不是階段** | 綠＋永久回歸鎖 | 部分 |
| 3 雙鏡頭 | 同一 diff | Reviewer（符合度/工藝）∥ Skeptic（證偽：可重放？爆炸半徑？無捏造？fail-closed？）——**並行** | 兩鏡頭皆過 | 判斷類**不自動化**（Goodhart 風險） |
| 4 GATE | 雙鏡頭通過 | canonical `scripts/check.ps1` | exit 0（Blocked≠PASS） | ✅ |
| 5 證據/回顧 | gate 輸出已讀 | register 列＋回顧；回顧可改規則（例：fmt 疏失→「讀 gate 才准提交」；並行 junk commits→單寫者） | 規則變更入文檔/ADR | 指標自 git/register 計算 |

**Debug 規則（模式紀律）：** (1) 無紅不修；(2) 修復必留永久測試；(3) 根因假設連續兩次失敗 → 停手升級（架構師/準則重審），禁止反覆亂試。

## 4. Workflow B — 對抗深審（里程碑邊界；**建議於 S14 邊界執行**）

- **時點：** 不逐 commit；在里程碑交界（建議：S14 稽核包完成後、S15 效能宣稱之前）以半天為單位執行。
- **手法（四類）：**
  1. **反演提示**：對每項切片宣稱問「什麼會使它為假？」並列出否證清單（承 AUDIT-0003 §4 體例）
  2. **重放/雜湊鑑識**：重放雙鏈、驗 manifest SHA、比對 register 數字與 git 事實
  3. **並行壓力**：模擬並行 session 提交（本倉實測發生過 `DONE`/`OKK` junk commits 與 plan 檔被覆寫）——驗證單寫者紀律與衝突偵測
  4. **偽造事件演練**：向雙鏈注入偽造/亂序/斷行事件，驗證 fail-closed 拒載
- **出口：** 發現登錄 IMP-*；否證清單全數有「已測試/已修復/已標注」三態。

## 5. Workflow C — 混沌與韌性（Shift-Right；**建議於 S14 後加入夜間演練**）

- **現況：** 混沌情境僅達**函式庫層**（竄改/斷行/獨佔鎖測試已存在於 adapters 測試）。
- **升級路徑：** 自動化夜間演練（本機排程或 CI；CI 尚未存在，屬 DEC 後）——腳本批次執行：竄改 ledger→拒載；寫入中斷→原子性；鎖爭用→單實例；時鐘回撥→決定論。
- **輸出：** chaos pass rate 進 register 指標；任何 FAIL 即 IMP-*。
- **誠實聲明：** 本機排程可用 cron/工作排程器；「夜間」在無 CI 前為本機約定，不宣稱 24/7。

## 6. 三工作流選擇準則

| 情境 | 用哪個 | 頻率 |
|---|---|---|
| 日常切片（如 S9/S10 形態） | A | 每切片 |
| 里程碑宣稱將對外（數字/匯出/備份） | A＋B | 里程碑邊界 |
| 基礎設施/儲存層變更後 | A＋C | 變更後一週內建立 nightly |
| 新測試層（property/mutation）導入 | A＋研究複核 | 依研究更新週期 |

## 7. 自動化升級積壓（每項≈一日內可落地；均為既有能力延伸）

| # | 項目 | 接線點 | 狀態 |
|---|---|---|---|
| AB-1 | BDD 場景↔測試名可追溯掃描（每情境 ID 出現於測試名） | `scripts/check-docs` 衛星或獨立腳本 | 待建（~40 行） |
| AB-2 | `????` 佔位字串掃描（operator-facing `.rs` 零容忍）——IMP-019 機械鎖 | canonical 衛星 | 待建 |
| AB-3 | 效能退步閘：新 benchmark JSON vs 基線（>10% 退步→fail） | `benchmark.ps1` 後處理 | 待建 |
| AB-4 | register 列 presence 檢查（動 crate 源碼的提交應有 register 編輯） | CI 前僅警告 | 待建（低優先） |
| AB-5 | cargo-audit/deny 併入 canonical | 先做**離線可行性 spike**（audit DB 可用性未知），不預諾 | 評估中 |

## 8. 度量（全部自 register/git 實測；不發明）

| 指標 | 定義 | 現況基線 |
|---|---|---|
| 逃逸率 | 合併後才發現的缺陷數/切片 | 2026-11-06 紅隊：3 項（IMP-019/020/021） |
| 證偽命中率 | skeptic 每輪否決/推翻的宣稱數 | AUDIT-0003：1 項假設被否證（路徑語義） |
| 紅→綠延遲 | 紅測試建立到轉綠的提交數 | 例：空案例修復 1 提交內完成 |
| 混沌通過率 | nightly 演練 PASS/總數 | Workflow C 導入後起始蒐集 |
| 流程測試總數 | workspace `cargo test --locked` 合計 | 503（2026-11-09 @`680aacf`） |

## 9. 導入建議（結論）

1. **立即**：Workflow A 為日常作業系統——它已是 S1–S10 的事實做法，本文件將其制度化。
2. **S14 邊界**：執行 Workflow B（在稽核包完成後、S15 效能宣稱之前），輸出登錄 register。
3. **S14 之後**：建立 Workflow C 夜間混沌演練（本機排程），chaos pass rate 進 register。
4. **同批落地** AB-1/AB-2/AB-3 三個自動化閘（見 §7）。
