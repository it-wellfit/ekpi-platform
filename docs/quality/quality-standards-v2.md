# 品質標準與驗收準則 v2.0

- **狀態：** Active
- **Owner：** QA + Engineering
- **最後審閱：** 2026-11-06
- **上位：** [phase gates](phase-gates.md)、[quality gates by risk](../development/quality-gates.md)、[再設計總計畫](../product/redesign-v2-master-plan.md)

本文件將「精緻…精選」十二維度轉為**可量測、可否決**的驗收準則。任何一項 FAIL 即整批退回，不以個別項目通過冒充整體通過。

## 1. 工程底線（每提交必過，電腦可判）

| ID | 標準 | 量測指令 | 門檻 |
|---|---|---|---|
| Q-01 | 建置 | `cargo check --locked --workspace --all-targets` | 0 錯誤 |
| Q-02 | 警告 | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 警告 |
| Q-03 | 格式 | `cargo fmt --check` | 0 diff |
| Q-04 | 測試 | `cargo test --locked --workspace` | 100% 通過，總數只增不減（現基線 445+） |
| Q-05 | 不安全碼 | 每個 `.rs` 首行 | `#![forbid(unsafe_code)]` |
| Q-06 | 文件 | `scripts/check-docs.ps1` | 必要檔案 53、連結全解析 |
| Q-07 | 秘密 | `scripts/check-secrets.ps1` | 0 發現 |
| Q-08 | 總門 | `scripts/check.ps1` | exit 0；被 host 政策擋下記 `Blocked`，不得記 PASS |

## 2. 產品正確性（領域門檻）

| ID | 標準 | 量測 | 門檻 |
|---|---|---|---|
| Q-10 | 引擎硬門檻 H-1…H-6 | `--out` 產出 `engine_hard_gates.json` | 全 OK（forbidden released=0、viol-as-comply=0、abstain 無引用、定論必有錨、注入只登記、重放一致） |
| Q-11 | 規則庫 schema | `--declared-rules-schema` | 全量 0 違規 |
| Q-12 | 檢索回歸 | `--eval --test-set data/eval/search_eval.json` | top-1 全中、拒答全對、權限隔離 1.00、violations 0 |
| Q-13 | 決定論 | property：同輸入兩次 decide 等價；拒答⇒零引用 | 永真 |
| Q-14 | 帳鏈完整性 | 竄改/斷行/空檔案拒收測試 | 全 PASS；append+fsync+臨界行拒讀 |
| Q-15 | 認證/CSRF | 登入 double-submit、缺 token 403、token 不符 403、輪替 cookie 生效、缺 csrf cookie 的已登入變更 403 | 全 PASS |

## 3. 介面與可用性（人工＋腳本混合）

| ID | 標準 | 驗收方法 | 門檻 |
|---|---|---|---|
| Q-20 | 零 JS | 掃描 `<script` 於模板/資產/report.html | 計數 = 0 |
| Q-21 | 七狀態 | 逐頁審計表（Default/Hover/Focus/Disabled/Loading/Empty/Error） | 每互動元件齊備；缺者記 IMP-* |
| Q-22 | 焦點可見/鍵盤可達 | 手動走查 + `:focus-visible` 存在 | 每頁通過 |
| Q-23 | 設計 Token | 掃描模板/資產內聯色碼/間距 | 僅 design-system.css 變數 |
| Q-24 | 錯誤訊息三段式 | 抽查所有錯誤文案 | 含「問題＋原因＋處理」 |
| Q-25 | 語言 | 介面文字 | 繁體中文（台灣情境） |
| Q-26 | 動效節制 | `prefers-reduced-motion` 規則存在且生效 | CSS 覆蓋 |

## 4. 效能預算（量測制，先基線後優化）

| ID | 標準 | 量測 | 基線（2026-11-05 debug） | 門檻 |
|---|---|---|---|---|
| Q-30 | 檢索延遲 | `scripts/benchmark.ps1` eval 樣本 | p50/p95 51.9/93.6 ms（debug） | release p95 < 50ms 或維持不退步 |
| Q-31 | HTTP 吞吐 | loopback `/healthz` | 歷史 5,369 req/s（release） | 不低於基線 80% |
| Q-32 | 啟動 | 平台載入時間 | 記錄於 benchmark JSON | 無回歸 |

## 5. 安全（本機範疇內可驗證者）

| ID | 標準 | 量測 | 門檻 |
|---|---|---|---|
| Q-40 | 供應鏈 | `cargo audit`（兩個 lockfile） | 0 未豁免漏洞 |
| Q-41 | 輸入防護 | 64 KiB 表單上限、重複 Cookie/Host/Origin 拒收、BPMN 解析邊界 | 既有回歸全 PASS |
| Q-42 | 逾時/限流 | 25ms 逾時測試、信號量併發上限 | 408/429 正確 |
| Q-43 | 會話 | PBKDF2、CSPRNG token、鎖定、過期清理 | 既有測試全 PASS |
| Q-44 | 企業身分 | — | **不做宣稱**：僅本機 session（DEC-12 未決） |

## 6. 驗收流程

1. 實作者自跑 Q-01…Q-08 → 貼輸出。
2. QA 抽查 Q-10…Q-15 與 Q-20…Q-26（審計表簽名）。
3. 每切片結尾：更新 register 證據列（指令+修訂+結果），FAIL 項登錄 IMP-*。
4. 門檻不得為通過現況而調降；放寬需 ADR + 贊助人核准。
