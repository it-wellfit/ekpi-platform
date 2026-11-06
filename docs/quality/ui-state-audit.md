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
| `/approvals` | 測試 `approval_lifecycle_through_web` | `empty_state("暫無簽核任務")`；定義「尚無定義。」 | 鏈徽章＋操作錯誤 notice；角色 403 | D1–D5 場景 |
| `/settings` | 靜態資訊（版本/儲存模式） | N/A | N/A（無資料依賴） | |
| `/api-docs` | 端點狀態表（誠實 501 標注） | N/A | 端點本身一律 501 結構化錯誤 | `unwired_api_endpoints_return_honest_501_never_fabricated_data` |
| `/ready` `/healthz` `/metrics` | 探針輸出 | — | readiness 檢查失敗回非 200 | |

## 3. 已修補缺口（2026-11-06）

1. `.badge-grey`（核准任務狀態/低急分級使用）— 補 CSS。
2. `:user-invalid` 表單驗證樣式 — 補 CSS。
3. `:disabled` 家族樣式 — 補 CSS。
4. `.inline-form`（核准決策行內表單）— 補 CSS。
5. reduced-motion 現亦中止 hover transform。

## 4. 遺留事項

- Loading 狀態設計上 N/A；引入任何非同步行為前，本表必須先補載入設計（禁止無設計即上線）。
- `governance.css` 內部分歷史色值為字面值（該檔本身是 token 源之一）；完全 token 化列為精雕後續項。
