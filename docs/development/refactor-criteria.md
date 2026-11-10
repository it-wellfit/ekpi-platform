# 重構決策準則（Refactor Criteria）v1.0

- **狀態：** Active — 任何重構提案先過本準則，再動手
- **Owner：** Engineering
- **最後審閱：** 2026-11-09
- **上位：** N1–N4 不變需求（[演化藍圖](../product/evolution-roadmap-v1.md)）；誠實鐵律（AGENTS.md）；門檻只升不降

## 1. 第一原理（為何要設準則，而非憑手感）

重構的目的是**降低未來變更成本**，不是讓今天的程式碼好看。任何重構若增加「行為不變性」的證明成本、或在並行開發期擴大碰撞面，其淨值為負。因此：

- **行為不變是預設**；改變可觀察行為＝變更需求，不是重構（須先有紅測試與規格變更）。
- **不刪不知其所以然的東西**（Chesterton's Fence）：先查歷史、消費者與證據，再談處置。
- **全部改動可回溯**：每一筆重構＝一個提交＋同位性證據＋完整 canonical。

## 2. 引用的成熟原則（如實標注性質，不誇稱定理已證於本專案）

| 原則 | 來源性質 | 本專案用法 |
|---|---|---|
| 耦合階（content > common > external > control > stamp > data） | Stevens–Myers–Constantine 結構化設計指標 | 跨模組共用實作＝common coupling（差）；共用**資料介面**＝data coupling（好）→ 去重時選「委派」而非「共享可變狀態」 |
| 資訊隱藏 | Parnas 1972 | 判斷哪些細節不該跨邊界暴露（例：路徑語義不該外洩成兩個同名函式） |
| 無環依賴 | Martin ADP | crate 依賴圖須保持無環（`cargo check` 已保證） |
| Hyrum's Law | 業界經驗律 | 邊界上任何可觀察行為都會被依賴 → 只對**內部且零外部消費者**的符號做改名 |
| DRY（語義層） | Hunt & Thomas | 只合併**語義相同**的重複；形式相似但語義可分歧者不合併 |
| 絞殺者模式 | Fowler | 大檔拆分以「先設計後遷移、逐段搬運」進行，不一次性重寫 |

## 3. 決策演算法（可重跑、可審計）

對每個候選收集五個量：**P**（同位性已驗證？）、**B**（外部消費者數＝爆炸半徑）、**C**（既有測試覆蓋該路徑？）、**H**（歷史變更提交數＝並行碰撞風險）、**S**（規模/複雜度）。

```
if P == false:            REJECT（先補表徵/紅測試，當成行為變更走規格流程）
elif 語義重複 && P:
    if B <= 內部 && C:    DO NOW（去重：委派/單一來源）
    else:                 NEXT SLICE（先降 B）
elif 同名異義:
    if B <= 2:            DO NOW（改名＋文檔；編譯器即證明）
    else:                 ADR 後改名
elif B > 10 || H >= 10:   DEFER（並行期大改＝禁止；需設計文件＋表徵套件＋單寫者窗口）
elif 純風格 && 無量測收益:  REJECT（禁止無規格的投機重構）
else:                     NEXT SLICE（附量測或覆蓋前提）
```

門檻（B>10、H≥10）非神聖數字：調整需 ADR；門檻只升不降的精神同品質標準。

## 4. 實測裁決表（2026-11-09 首次掃描；2026-11-10 重測 C3/C6）

| 案 | 候選 | P | B | C | H | S | 裁決 | 證據/理由 |
|---|---|---|---|---|---|---|---|---|
| C1 | `platform::governance::default_ledger_path` 與 `adapters::configuration::default_ledger_path` 邏輯重複 | ✅（僅差 `PathBuf::from` vs `Path::new`，語義相同） | 2（bin、main） | ✅（configuration 單元測試＋編譯） | 6＋1 | 12 行 | **DO NOW → 已執行** | 單一事實來源：platform 委派 adapters；消費者面名保持穩定（Hyrum） |
| C2 | `adapters::jsonl_ledger::default_ledger_path` 與 C1 同名但回傳**目錄** | 語義不同，非重複 | 1（內部） | ✅ | 3 | 10 行 | **DO NOW → 已執行** | 改名 `default_data_dir`＋文檔區辨；編譯器保證零外部影響 |
| C3 | `governance_web.rs` 拆分為模組（2026-11-10 重測：**4025 行**，含未提交 detail pages） | ❌（`page_registry` 表徵套件剛補齊 2 個測試；golden HTML、完整角色矩陣與 mutation 契約仍缺） | **7 個檔案引用**（含 bin/tests） | 部分（頁面 auth/zero-JS/title 契約；未涵蓋所有 mutation） | **24** | 4025 行（未提交工作 +289 行） | **DEFER** | 2026-11-09 舊值 3246 行／19 引用／21 提交已漂移；重測後碰撞面更大。前提不變：模組設計文件＋完整表徵套件＋單寫者窗口 |
| C4 | TODO/FIXME 19＋`allow(too_many_arguments)` 1 | — | 0 | — | — | — | **分級** | postgres 骨架 9 筆＝已聲明 fail-closed（可接受，IMP 追蹤）；其餘為語義註記；`approvals_content` 參數列 → NEXT SLICE 以參數結構體收斂 |
| C5 | 核准三個 handler 的重複「讀 body＋解析失敗分支」 | ✅（已抽 `read_limited_body`；剩錯誤分支樣式） | 0（檔內） | ✅（route 測試） | 3 | ~24 行 | **NEXT SLICE** | 與 C4 的參數結構體同一微切片處理，避免為小收益頻繁觸碰高扇出檔 |
| C6 | **兩個獨立的 `EventPayload` enum**：`platform::governance::EventPayload`（治理帳鏈在用）與 `ekp_domain::governance::EventPayload`（9 變體同名同序） | ❌（尚無逐位元組 serde 等價測試） | **80 引用**（2026-11-10 重測）；9 個 payload struct 亦為兩份獨立定義 | 平台版由帳鏈測試覆蓋；無跨型別等價測試 | 2 個定義提交 | 兩份 9 變體＋9 組 payload struct | **NEXT SLICE（需型別遷移，非一行 re-export）** | 2026-11-10 重測：兩 enum 變體相同，但 `Project`／`Evidence`／… 等 payload struct 是 platform 與 domain 各自的獨立型別，直接 `pub use` 不會編譯。需先做 serde 輸出逐位元組等價測試，再以專用遷移切片（或 ADR）收斂；S11 資料地圖對準**在用**的 platform 版 |

## 5. 整體優構次序（不可調換）

1. **正確性**（N1–N4、fail-closed）→ 2. **可測性**（表徵套件）→ 3. **邊界一致性**（耦合階降級、同名異義清除）→ 4. **量測驅動效能** → 5. **美學/命名打磨**。理由：每一層都比下一層更便宜、更可證；反序＝用最貴的方式改最不重要的事。

## 6. 變更控制

- 每筆重構提交附：同位性證據（diff/推導）＋B 實測＋完整 canonical 輸出。
- 本準則修訂需 ADR；新增案例只進 §4 表，不散落他處。
