# S11–S15 執行計畫 v1.0（詳細版）

- **狀態：** Active — 逐片執行；每片以 canonical 為唯一收口
- **Owner：** Engineering + QA
- **建立：** 2026-11-09（基線：HEAD `680aacf`、樹乾淨、workspace **503 測試**全綠、docs PASS）
- **上位：** [演化藍圖](../product/evolution-roadmap-v1.md)、[重構決策準則](../development/refactor-criteria.md)、[品質標準](../quality/quality-standards-v2.md)、[BDD 場景](../quality/bdd-scenarios-approval.md)
- **執行紀律：** 單寫者（紅隊 §2.1 教訓）；一個切片＝一個提交（gate 輸出讀完才提交——`52b4e6c` 教訓）；證據列即回顧輸入

## 0. 全域基線與每片共同 DoD

**基線（2026-11-09 量測）：** workspace 測試 **503**；canonical `scripts/check.ps1` exit 0（ALL CHECKS PASSED）；clippy `-D warnings` 零警告；fmt 乾淨；docs 檢查 PASS；benchmark 基線 2026-11-06（eval p50/p95 36.2/42.6ms；HTTP 10.2/12.3ms；4,758 req/s）。

**每片共同 DoD（缺一不可）：**
1. 紅測試先行（新行為必有具名失敗測試，BDD 情境 ID 對應）
2. `cargo fmt` → `cargo clippy --locked --workspace --all-targets -- -D warnings` 零警告
3. `cargo test --locked --workspace` 全綠，**總數 ≥ 基線+Δ**（記錄確切數字）
4. canonical `scripts/check.ps1` exit 0（`ALL CHECKS PASSED`；被 host 政策擋下記 `Blocked`）
5. register 證據列＋roadmap 勾稽＋（文件變更時）check-docs PASS
6. **gate 輸出讀完才 commit**；一個切片一個提交

---

## S11 資料地圖（Data Map）

**目標：** 欄位→事件→頁面的血緣可查證。生成來源是**型別系統**（對 `EventPayload` 變體的窮舉 match），不是文字掃描——新增事件變體時，窮舉測試**編譯失敗**即強制更新血緣，杜絕漂移。

**交付物：**
- `platform/src/data_map.rs`（新模組）：`DATA_MAP: &[LineageEntry]` 常量表＋`build_data_map()` 產生 JSON
- CLI：`--data-map DIR`（獨立模式）寫 `DIR/data_map.json`，schema `ekpi-data-map-v1`（無時間戳/路徑，重放逐位元組一致——承 CTR-EXP-002/003 同一紀律）
- 頁面：`/data-map`（GET，core router，Viewer+）以表格渲染血緣
- 契約：`docs/contracts/data-formats.md` §Data map（`CTR-DATA-008`）
- 窮舉鎖：`fn lineage_for_payload(payload: &EventPayload) -> Vec<&'static str>` 對 `EventPayload` **窮舉 match**；任何未覆蓋變體＝編譯錯誤

**紅→綠測試（具名）：**
- `data_map_is_exhaustive_over_event_payloads`（窮舉 match 編譯即證）
- `data_map_export_is_deterministic`（雙跑逐位元組一致）
- `data_map_page_requires_session_and_renders`（401/200＋表格）

**驗收（DoD）：** 血緣表覆蓋 snapshot 全部事件變體＋核准 store 五種 payload（serde tags）；每列含「欄位｜事件｜寫入面（表單/API）｜呈現頁」；無人工虛構列。

**canonical 檢查：** 同 §0 全套；預期 exit 0、測試 ≥ 503+4、docs 連結 ≥757+Δ、clippy/fmt 乾淨。

**依賴：** 無外部決策。**風險/退路：** 血緣列遺漏→由窮舉測試強制；退路＝還原提交。

---

## S12 SLA 行事曆

**目標：** 以 CSS grid 月曆呈現任務到期日分佈與逾期紅點——讓「SLA 失控」痛點在時間軸上可見（零 JS）。

**交付物：**
- 路由：`GET /approvals/calendar`（approval 子路由，state `(SharedAuth, SharedApprovals)`）
- `calendar_content(...)`：月曆格（7 欄）＋任務點（due_at 落點）；逾期任務紅點＋「已逾時」；當月任務列表（複用 `sla_meter_view`）
- CSS：`.calendar-grid`、`.calendar-cell`、`.calendar-day`、`.calendar-dot`（含 reduced-motion/對比度合規）
- 導航：/approvals 頁加「行事曆」入口 link

**紅→綠測試：**
- `calendar_renders_overdue_task_on_its_due_date`（以 store `submit_task` 用過去 now 注入逾期任務→斷言紅點類別＋任務 id 出現於對應格）
- `calendar_renders_empty_month_honestly`（無任務→誠實空狀態）
- `calendar_requires_session`（401）

**驗收：** 逾期/即期/未來三色可辨（不以顏色為唯一傳達——同時有文字標注）；無 JS 互動（月份切換以 `?month=YYYY-MM` GET 參數，伺服器渲染）；月界由伺服器時鐘計算並在頁面標明時區語義（UTC 秒）。

**canonical 檢查：** 同 §0；預期 exit 0、測試 ≥ 前基線+3。

**依賴：** S2 store（due_at 資料）。**風險/退路：** 時區語義誤導→以 UTC 標注＋`format_timestamp` 既有慣例；退路＝還原提交。

---

## S13 合規報告產生器

**目標：** 從**實際資料**產生可稽核的狀態報告（不是合規宣稱）：品質個案收口率、證據審查覆蓋、雙鏈驗證、缺口清單。

**交付物：**
- CLI：`--export-compliance-report DIR [--governance-data PATH] [--approval-data PATH]`（S9 同款旗標機制；衝突規則同 CTR-EXP-003）
- 產物：`compliance_report.json`（`ekpi-compliance-report-v1`）＋`compliance_report.md`（人讀版，由同一 JSON 渲染——單一事實來源）
- 內容（全部可推導）：個案狀態分佈、逾期未決清單、證據審查覆蓋率（reviewed/total）、雙鏈 `chain_verified`、平台語料計數、**缺口清單**（未審證據逐筆 id、開放事項逐筆 id）——**不含**任何「符合 X 法規」宣稱
- 契約：`docs/contracts/data-formats.md` §Compliance report（`CTR-EXP-004`）

**紅→綠測試：**
- `compliance_report_is_deterministic`（雙跑逐位元組）
- `compliance_report_lists_open_items_honestly`（注入開放個案→缺口清單含其 id）
- `compliance_report_refuses_conflicting_modes`（usage error 家族）

**驗收：** 報告每個數字可回推至 snapshot/store；文件明載「本報告為狀態事實，非合規認證」。

**canonical 檢查：** 同 §0；預期 exit 0、測試 ≥ 前基線+3。

**依賴：** S9（旗標機制與 pack 慣例）。**風險/退路：** 語義誤用（被當合規證書）→ 文件內建免責與來源連結；退路＝還原。

---

## S14 稽核包＋IMP-021 runtime 資料還原演練

**目標：** 一鍵產出可移交的稽核包，並**首次執行 runtime 資料還原演練**（關閉 IMP-021 的演練缺口）。

**交付物：**
- `scripts/audit-package.ps1`：產出 `audit-package-<ts>/`＝git bundle＋runtime 雙 JSONL 複本＋`evidence_pack.json`＋`manifest.json`（各檔 SHA-256；**manifest 本身不含時間戳**，時間戳僅在目錄名）＋`restore-drill.ps1`
- `scripts/drill-runtime-data.ps1`：**合成非敏感資料**驅動：設定 `EKP_GOVERNANCE_DATA`/`EKP_APPROVAL_DATA`/bootstrap 帳號 env → 啟動 `governance_web`（背景）→ HTTP 流程（登入→建專案→建/發布工作流→提交→決策）→ 停止 → **複製雙檔→刪除原檔→還原複本** → 再啟動 → 斷言 `/audit` 與 `/approvals` 徽章為「鏈已驗證」→ 記錄 PASS/FAIL
- 執行一次 drill 並記錄於 `backup-log.md`（標明：合成資料、真實檔案、觀察到的還原＋雙鏈驗證）→ **IMP-021 → Done**（誠實註記：以合成資料演練，非真實營運資料）
- 契約：manifest schema `ekpi-audit-manifest-v1`（`CTR-EXP-005`）

**紅→綠/驗證：**
- drill 腳本自我斷言（HTTP 狀態碼＋徽章文字＋manifest 雜湊重算一致）
- `audit_package_manifest_is_deterministic`（對固定輸入目錄：manifest 內容排除時間戳後逐位元組一致）

**驗收：** `restore-drill.ps1` 在乾淨目錄執行 exit 0；`backup-log.md` 新列 PASS（演練範圍如實）；IMP-021 狀態更新含證據連結。

**canonical 檢查：** 同 §0（腳本不影響 cargo 測試數；docs 連結數 +Δ）。

**依賴：** S9（evidence pack）；bootstrap 帳號機制（既有）；**不依賴 DEC**（合成資料）。**風險/退路：** PowerShell HTTP 會話複雜→以 .NET HttpClient＋CookieContainer 明確實作；host 政策擋下→記 `Blocked` 不記 PASS。

---

## S15 效能與可用性複核

**目標：** 以量測收口：無回歸或有據的改善；無障礙/美學終檢。

**交付物：**
- `scripts/benchmark.ps1` 執行兩次（前/後各一）→ `artifacts/benchmark/` 新 JSON；與 **2026-11-06 基線**並列（eval p50/p95 36.2/42.6ms；HTTP 10.2/12.3ms；4,758 req/s）
- 對比表寫入 register 證據列（前/後數字並列，任何 >10% 退步須附原因與後續動作）
- 終檢清單：`prefers-reduced-motion` 全域生效、`:focus-visible` 可見、`:user-invalid` 樣式、對比度抽樣（body/panel/badge 五組）——更新 `ui-state-audit.md`
- S11–S14 新頁面的七狀態審計補行

**驗收：** 數字並列無隱瞞；退步有原因與行動；審計表更新且 `stylesheet_includes_state_primitives` 仍綠。

**canonical 檢查：** 同 §0＋`gate.ps1 -IncludeSmoke`（報告/匯出類變更的強化門）。

**依賴：** S11–S14 完成（量測最終態）。**風險/退路：** 效能退步→回滾該切片或量測後優化（精研原則：先量測）。

---

## 執行順序與相依

```
S11（資料地圖）──► S12（行事曆）──► S13（合規報告）──► S14（稽核包＋drill）──► S15（量測複核）
   無外部依賴        依 S2 資料        依 S9 旗標機制      依 S9＋合成資料          量測最終態
```

- **順序理由：** S11 先建立血緣（S13 報告的引用基礎）；S12 獨立但同檔案（governance_web），單寫者下排 S11 後；S13 需 S9 機制＋S11 血緣；S14 需 S9 pack＋drill；S15 必須最後（量測最終態）。
- **並行性：** 不建議並行（單寫者紀律）；S12 理論上可與 S11 並行但共用 `governance_web.rs`，碰撞面由重構準則 §3 排除。

## 誠實邊界（全系列）

- A1 真實試點（`DEC-01`）、IdP/WORM（`DEC-12`）、部署目標（`DEC-17`）等維持外部決策；S14 演練使用**合成資料**並如實標注；S13 報告是**狀態事實**不是合規認證；S15 只比較量測數字。
