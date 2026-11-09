# S14 邊界對抗深審（Workflow B 首次執行）

- **日期：** 2026-11-09
- **範圍：** S11 資料地圖、S12 SLA 行事曆、S13 合規報告的對外宣稱（修訂 `68ad30e`…`514d59e`）
- **審查者：** Engineering（自我紅隊；非獨立審查）
- **方法：** Workflow B 四手法（反演提示／重放鑑識／並行壓力／偽造事件）——見 [QA 演進流程](../quality/qa-evolution-flow.md) §4

## 1. 反演檢驗（每項宣稱問「什麼會使它為假？」）

| 宣稱 | 否證嘗試 | 結果 |
|---|---|---|
| S11「血緣由型別系統鎖定，新增變體即編譯失敗」 | 檢查兩個窮舉 match 是否真無萬用臂；檢查表規模測試是否鎖 9/5 | **成立**（`governance_lineage_for`/`approval_lineage_for` 無萬用臂；`lineage_table_sizes_match_the_event_enums` 鎖 9/5） |
| S11「/data-map 計數反映實際帳鏈」 | 建立專案後計數是否增長（`data_map_counts_reflect_live_ledger`） | **成立** |
| S12「日期演算法決定論且正確」 | 紀元/閏年/月界/120 個月星期不變式 | **成立**——但紅測試曾抓出 `days_from_civil` 缺 −719,468 紀元平移（修復入檔） |
| S12「逾期以顏色＋文字雙重標注」 | cell 級斷言（`data-date` 格內含任務 id＋「已逾時」） | **成立** |
| S13「報告決定性、無時效計算」 | 雙跑逐位元組；grep 確認無 generated_at；逾期欄位不存在（due_at 為事實欄位） | **成立** |
| S13「/data-map、/approvals 頁徽章真實」 | HTTP 徽章斷言（drill v7 抓出 /audit 文案錯置→修正） | **成立（修正後）** |

## 2. 過程發現（誠實揭露）

1. **並行 session 汙染升級**：本切片期間並行 session 以 junk 訊息（`b16688d "okk"`）收走路徑上**未提交**的 S13 實作（main.rs 302 行＋cli 測試 105 行），並活躍編輯 main.rs（RAG/--search 方向）。內容倖存、已驗證，但提交訊息不可審計。**對策已寫入**：register R16 provenance 註記＋單寫者紀律重申。
2. **drill 腳本三連錯（皆已修復）**：(a) PS 5.1 對無 BOM UTF-8 腳本以 ANSI 讀取→中文斷言變亂碼 regex（Quantifier 錯誤）→改 UTF-8 BOM；(b) 缺 `Add-Type -AssemblyName System.Net.Http`；(c) login 未走雙重提交流程（伺服器**正確**以 403 拒絕——drill 設計錯誤非伺服器缺陷）→改為 GET 取 token＋POST 回帶。
3. **/audit 徽章文案錯置**：drill 斷言假設 /audit 用「鏈已驗證」，實際是「本機雜湊鏈驗證通過」——斷言對準真實頁面文案後 PASS（伺服器無缺陷）。
4. **無模式守衛遺漏**（S13）：`compliance_report_dir` 未加入無模式條件 → 分支不可達、無訊息 usage exit 2——新 CLI 測試抓出，已修（`514d59e`）。
5. **雙 `EventPayload` enum**（C6）：S11 發現，合併前置（serde 逐位元組相容驗證）列 NEXT SLICE。

## 3. 並行壓力觀察（重放鑑識）

- 本切片期間 workspace 清單被並行改寫（apps/web deps 增 tower-http/tracing/uuid/chrono）→ 曾致 `cargo check -p` 暫態「package not match」（轉瞬即復）。
- 增量建置腐敗一次（LNK2019，`cargo clean -p ekp_platform` 解決）——已記錄於 R15，非程式缺陷。

## 4. 結論與否證清單

S11–S13 的對外宣稱在反演下**站得住**（見 §1 全表成立）。流程層發現（drill 腳本、文案、守衛遺漏）已全部修復並以測試/演練鎖定。**未變**：A1 `Not met`（DEC-01）、Phase B 阻塞、無獨立審查、無 CI 證據。

否證清單（任一成立即撤回本審查結論）：
1. 519→後續測試總數下降且無說明。
2. 任何 IMP-019/020/021 被文檔-only 關閉。
3. 雙鏈 `chain_verified` 在無人為干預下轉 false。
4. /data-map 血緣表與 enum 變體數不同步而測試仍綠。
