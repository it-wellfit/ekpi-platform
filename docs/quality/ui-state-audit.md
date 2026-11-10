# UI 七狀態逐頁審計 v1.0（Q-21）

- **狀態：** Active（S3 產物；每次新增/修改頁面後更新本表）
- **Owner：** Engineering + QA
- **最後審閱：** 2026-11-06
- **準則：** [品質標準 Q-20…Q-26](quality-standards-v2.md)；樣式僅 Design Token（`platform/assets/governance.css`、`apps/web/static/css/design-system.css`）
- **方法：** 每格標注證據（測試名稱或 CSS 選擇器）。`N/A（設計）` 表示該狀態在零 JS 同步伺服器渲染下不適用——記錄理由，不虛構。

## 1. 全域狀態基元（所有頁面共用，`governance.css`）

| 狀態 | 實作 | 證據 |
|---|---|---|
| Hover | `a:hover`、`.nav-link:hover`、`.button:hover`、`tbody tr:hover`、`input:hover` 等 | CSS L29–126 |
| Focus | 全域 `:focus-visible`（3px outline）＋ `input/select/textarea:focus-visible` 邊框 | CSS L32、L127；測試 `stylesheet_includes_state_primitives` |
| Disabled | `button:disabled`、`input:disabled` 等（opacity + not-allowed，hover transform 中止） | CSS `button:disabled` 區塊 |
| 使用者驗證失敗 | `:user-invalid`（紅邊框，與 `required`/`maxlength` 原生驗證聯動） | CSS `input:user-invalid` 區塊 |
| Reduced motion | `prefers-reduced-motion` 關閉 transition 與 hover transform | CSS L226 區塊 |
| Loading | **N/A（設計）**：零 JS 同步伺服器渲染，頁面無客戶端載入期；表單送出即 PRG 導頁。長時非同步作業目前不存在；未來引入時須先補 CSS 骨架 | 本表聲明 |

## 2. 逐頁審計

| 頁面 | Default（有資料渲染） | Empty（空狀態） | Error（錯誤處理） | 備註 |
|---|---|---|---|---|
| `/` 儀表板 | 路由測試（governance_web suite） | inline「尚無活動記錄…」 | `page_error_response`；401/403 結構化 | |
| `/login` | 表單＋本機提示 | N/A（表單頁） | 通用 401/403；CSRF 失敗訊息不揭露細節；鎖定不揭露 | `login_post_without_csrf_token_is_refused` 等 |
| `/projects` | 路由測試 | `empty_state("尚無專案資料")` | 錯誤 re-render 帶 notice | |
| `/evidence` | 路由測試 | `empty_state("尚無證據參照")` | 同上 | |
| `/inspections` | 路由測試 | inline「尚無檢驗」 | 同上 | |
| `/workflows` | 路由測試 | `empty_state("尚無流程範本")` | 同上 | |
| `/controls` | 路由測試 | `empty_state("尚無追蹤事項")` | 同上；CAPA 關閉需驗證訊息 | |
| `/lifecycle` | 路由測試 | inline「尚無關卡決策」 | 同上；exception＋non-waivable 拒絕 | |
| `/audit` | 路由測試 | `empty_state("尚無稽核事件")` | 鏈徽章（綠/紅） | |
| `/approvals/calendar`（S12 新增） | 月曆格（`?month=YYYY-MM` 伺服器渲染）＋到期任務連結＋逾期紅格＋「已逾時」文字（顏色非唯一傳達）＋前/後月 chips | 空月→「0 筆」誠實計數；任務不在本月→outside 計數說明 | 未登入 401；無效月份（`2026-13`/`not-a-month`）→ 400 fail-closed（不靜默回退當月）；`sla_calendar_navigation_and_validation` |
| `/settings` | 靜態資訊（版本/儲存模式） | N/A | N/A（無資料依賴） | |
| `/api-docs` | 端點狀態表（誠實 501 標注） | N/A | 端點本身一律 501 結構化錯誤 | `unwired_api_endpoints_return_honest_501_never_fabricated_data` |
| `/events`（S12 批次） | 事件表（序號/標籤/操作者/時間/雜湊膠囊）＋標籤過濾 chips＋分頁 | 過濾無命中→誠實空狀態；鏈空→同 | 401；`events_browser_filters_by_type`（鏈事實渲染、誠實省略 payload 內文） |
| `/audit/chains`（S12 批次） | 雙鏈 KPI 卡＋各類事件計數表 | 鏈空→「尚無事件」 | 鏈驗證徽章（綠/紅）；401 |
| `/role-matrix`（S12 批次） | 角色 × 能力表＋「鏡像程式碼」聲明 | N/A（常數表） | 401；`role_matrix_renders_role_capability_table` |
| `/glossary`（批次 3） | 領域字彙表（核准工作流＋資料與鏈） | N/A（常數表） | 401；`info_pages_render_glossary_help_about` |
| `/help`（批次 3） | 操作說明（登入/角色/匯出/疑難排解表） | N/A | 401；同上 |
| `/about`（批次 3） | 誠實邊界（是什麼/明確不是什麼） | N/A | 401；同上 |
| `/search`（S7 新增） | 檢索頁＋`doc@version§section` 引用卡＋分數計＋快速查詢 chips＋索引統計 | 空查詢引導狀態；無命中誠實空結果（「找不到相符條款」）；無假結果 | 文件庫載入失敗 notice-red；未登入 401；`search_page_renders_honest_states_and_index_counts`、`search_page_requires_session` | |
| `/data-map`（S11 新增） | 雙鏈血緣表（治理 9＋核准 5：事件/欄位/寫入面/頁面）＋**現況事件數**欄 | 血緣表為常數（無空狀態語義）；頁面本身唯讀 | 未登入 401；`data_map_page_requires_session_and_renders` | |

### S7 視覺化元件與其狀態（2026-11-06 十維躍升層）

| 元件 | 狀態覆蓋 | 證據 |
|---|---|---|
| KPI 卡（`.kpi-card`） | default / good（綠值）/ alert（紅值）/ hover 陰影升階 | `dashboard_renders_kpi_cards_and_grouped_nav`；CSS 鎖 `.kpi-card` |
| 甜甜圈（`.donut`，conic-gradient） | 空資料→`empty-inline`；有資料→百分比環 | 同上（品質事項/關卡區塊） |
| SLA 計（`.meter` 三色） | info（充足）/ warn（<15%）/ over（逾時紅＋「已逾時」）/ 終態「—」 | `approvals_page_shows_sla_meter_and_stage_dots` |
| 階段點（`.stage-dots`） | done 綠 / current 藍環 / todo 灰；附「第 k/n 階段」文字（不以顏色為唯一傳達） | 同上 |
| 時間軸（`.timeline`） | done/alert 標記；無事件→`empty_state` | 稽核與儀表板活動 |
| 披露（`details.disclosure`） | 收合/展開箭頭旋轉；`prefers-reduced-motion` 下停用過渡 | CSS 鎖 |
| 雜湊膠囊（`.hash-chip`） | 截短顯示（ASCII hex 安全截斷） | 稽核頁迴歸 |

## 3. 已修補缺口（2026-11-06）

1. `.badge-grey`（核准任務狀態/低急分級使用）— 補 CSS。
2. `:user-invalid` 表單驗證樣式 — 補 CSS。
3. `:disabled` 家族樣式 — 補 CSS。
4. `.inline-form`（核准決策行內表單）— 補 CSS。
5. reduced-motion 現亦中止 hover transform。

## 4. 遺留事項

- Loading 狀態設計上 N/A；引入任何非同步行為前，本表必須先補載入設計（禁止無設計即上線）。
- `governance.css` 內部分歷史色值為字面值（該檔本身是 token 源之一）；完全 token 化列為精雕後續項。
