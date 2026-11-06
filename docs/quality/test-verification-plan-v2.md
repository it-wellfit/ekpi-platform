# 測試驗證計畫 v2.0

- **狀態：** Active
- **Owner：** QA + Engineering
- **最後審閱：** 2026-11-06
- **配套：** [品質標準](quality-standards-v2.md)、[再設計總計畫](../product/redesign-v2-master-plan.md)、[AI evaluation](ai-evaluation.md)

## 0. 原則

- 測試金字塔：底寬（單元/property）→ 中（整合）→ 尖（E2E）；上層失敗先在底層加鎖。
- 紅→綠：缺陷修復必先有失敗測試；新功能先契約後實作。
- 一切「PASS」綁指令+修訂；Blocked ≠ PASS。

## 1. 測試層級與現況

| 層級 | 工具 | 現況（2026-11-06） | 本計畫新增 |
|---|---|---|---|
| 單元 | 內建 `#[test]` | platform 273、eval 12、adapters/application/domain 41+ | approval/channels 單元補齊 |
| Property | proptest | 18 條（決定論、H-3 拒答、帳鏈、權限、cosine） | +匯出 v2 重放性、+郵冊邊界 |
| CLI 契約 | 黑箱 spawn | 31 條 | +`--export-json`（v2 契約後） |
| Web 整合 | oneshot 路由 | governance 25 | +approval 流程 6 條 |
| 真 HTTP | axum::serve+loopback | integration_web 11 | +apps/web 頁面煙霧 |
| E2E | Playwright（`platform/tests/e2e/`） | 腳本在、未入 gate | 每頁 1 條，納入 `gate.ps1 -IncludeSmoke` |
| 效能 | `scripts/benchmark.ps1` | eval p50/p95、HTTP 吞吐 | 基線入 CI 比對（本機） |
| 安全 | check-secrets、audit、deny、邊界測試 | 全入 canonical | `cargo-deny` licenses/bans 週期化 |
| 可用性 | 七狀態審計表 | 表已定義 | 每頁簽核記錄 |
| 故障注入 | 人工 | — | 帳本鎖死、磁碟滿、斷行注入 3 條腳本 |

## 2. 各層通過標準（節錄）

- **單元/Property**：全綠；property 縮小反例不得重現。
- **整合**：路由層狀態碼與帳本效果同時驗證（不只能 200，要看 ledger 事件）。
- **真 HTTP**：真標頭（Host/Origin/Cookie/x-request-id）逐項斷言；安全頭全出現。
- **E2E**：登入→瀏覽→變更→登出主流程；每頁 zero-`<script>` 斷言。
- **效能**：與基線 JSON 並列比對；退步 >10% 即退回。
- **安全**：canonical 內建項 + 邊界回歸全綠；新依賴需 audit/deny 乾淨。
- **可用性**：Q-20…Q-26 審計表每頁簽核（見品質標準）。

## 3. 執行點（何時跑什麼）

| 時點 | 執行 |
|---|---|
| 每次存檔 | `cargo check --locked -p <crate>`（開發者本機） |
| 每切片收口 | `scripts/check.ps1`（canonical，exit 0） |
| 文件/報告/匯出變更 | `gate.ps1 -IncludeSmoke`（13 項含煙霧） |
| 發布/移交 | `gate.ps1 -IncludeSmoke` + bundle 還原演練 + benchmark 並列 |
| 每週 | `cargo audit`、`cargo deny`、連結檢查 |
| 每修缺陷 | 先紅測試，修後綠；回歸測試永久保留 |

## 4. 缺陷處理

1. 發現 → 登錄 IMP-*（描述、重現、分級 P0-P3）。
2. P0/P1：立即紅測試 → 修復 → 綠 → canonical。
3. P2/P3：入切片排程；不阻塞當前 gate 但須在 register 追蹤。
4. 任何「測了但沒寫下來」視同未測。
