# SLO 草案（本機試點範疇）v0.1

- **狀態：** Draft — 目標值為草案，未經產品 owner 簽核（`metrics/targets.md` 之數字多為 `proposed`）；本文件不宣稱任何生產 SLO
- **Owner：** Operations（owner 未指派，`DEC-16` 未決）
- **最後審閱：** 2026-11-06
- **範圍聲明：** 本專案為本機試點，僅 loopback；無多實例、無負載均衡。本文件定義「可量測的本地目標」，量測基準見 `artifacts/benchmark/`。

## 1. 服務等級指標（SLI → SLO）

| SLI | 量測點 | 草案 SLO | 現況證據 |
|---|---|---|---|
| 就緒探針成功率 | `/healthz`、`/ready` | 每次檢查 100% OK（資料檔齊備時） | `ready.rs` 檢查；CLI `--health` OK/DEGRADED |
| 檢索延遲（eval 語料） | `scripts/benchmark.ps1` | debug p95 ≤ 100ms；release p95 ≤ 50ms | 2026-11-05：debug p50/p95 51.9/93.6 ms |
| 請求吞吐（/healthz） | loopback 基準 | 不低於歷史基準 80%（release 5,369 req/s） | `artifacts/benchmark/*`（port 占用時跳過，記錄不造數） |
| 結果正確性 | 28-case 檢索 gate | top-1 100%、拒答 100%、權限隔離 1.00、violations 0 | 每次 canonical |
| 硬門檻 H-1…H-6 | `engine_hard_gates.json` | 全 OK；任何 VIOLATED 即不可交付 | 每次 `--out` |
| 帳鏈完整性 | 兩條 JSONL 鏈 verify | 100% 通過；竄改/斷行必拒載 | 竄改/斷行/獨佔鎖測試 |
| 登入正確性 | 會話測試 | 合法憑證 100% 成功；跨站/無 CSRF 100% 拒絕 | governance web 25 測試 |

## 2. 錯誤預算（草案）

- 錯誤預算 = 1 − SLO。本機試點無 24/7 運行，預算以「每次 canonical 執行」為單位：一次 canonical 內任何門檻 FAIL 即耗盡當次預算，該修訂不可交付（已在 `ablation()` fail-closed 實施）。

## 3. 量測紀律

1. 每次發布前：`benchmark.ps1`（port 占用 → 如實記 Blocked）＋ canonical。
2. 延遲退步 >10% → 不得宣稱效能未退步；需量測後優化（精研）。
3. 本文件為草案；升級為正式 SLO 需 DEC-16/DEC-18 決策與 owner 簽核。
