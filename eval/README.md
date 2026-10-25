# EKPI 評測骨架（Ablation Harness）

> 對應 Master Prompt v1 §九 + Addendum A5/A7。
> **規範性實作：Rust crate `ekp_eval`**（`#![forbid(unsafe_code)]`；serde/serde_json 為僅有依賴）。
> Python 版降為 `reference-python/` 參考實作，非規範性。
>
> **執行狀態（2026-10-25）：REAL PASS。** rustc/cargo 1.99.1 真機驗證：
> `cargo clippy --all-targets -- -D warnings` 零警告；`--selftest` 10/10 PASS；報告已生成於 `report_out/`。

## 指令

```powershell
cd eval
cargo run -- --selftest                    # 10 項自檢（指標與硬門檻管線）
cargo run -- --out report_out              # 產出 report.json + report.md
cargo run -- --systems stub:oracle --out r # 只跑指定 adapter
cargo fmt ; cargo clippy --all-targets -- -D warnings   # 提交前必跑（工作區規則）
```

adapter 註冊：`src/systems.rs` 的 `registry()` 加入你的 `System` 實作即可。
`Decision` 標籤會被完整性檢查驗證（system tag 必須與 adapter 名一致）。

## 自檢驗證的三件事

- `stub:oracle`（作答正確的天線）→ verdict accuracy 1.0、wrong_allow=0、正確拒答 1.0、
  escalation 1.0、citation recall 1.0：指標管線可達理想值，無反向 bug。
- `stub:allow_all`（全部判合規）→ **violation_as_comply=1、hard_gate=true**：
  「漏報=0」這道剛性門檻真的會擋下壞系統。
- `stub:refuse_all`（全部拒答）→ definitive 案件無故拒答率 1.0：無故拒答被計量。

報告 `report.md` 含逐案例失敗分析（case / category / expected / actual / rules_used / notes），
直接對接「以失敗案例驅動持續學習」的工作流。

## 指標速查（Addendum A5）

| 計量 | 定義 | 用途 |
| --- | --- | --- |
| `wrong_allow` | 非合規案回答 comply | **硬門檻（含 violation_as_comply）** |
| `refusal_on_definitive` | 需判定但回 insufficient_evidence 的比例 | 無故拒答 |
| `correct_refusal` | 缺證案正確拒絕的比例 | 正確拒答 |
| `escalation_correct` | 該升級且真升級 | 循環/衝突升級 |
| `citation_recall / precision` | 引用對上預期證據位 | 引用正確性 |
| `coverage_curve` | 依 confidence 降序各覆蓋率下的誤差 | risk–coverage 校準 |
| `confusion_matrix` | expected×actual 交叉 | 錯誤源分析 |

## 種子案例集（`cases/seed_cases.jsonl`，9 條，語言中立）

| case_id | 情境 | 預期 | 設計目的 |
| --- | --- | --- | --- |
| ORD-001 | 先執行後核准（文件齊全） | violate | 順序違規 ≠ 因果 |
| ORD-002 | 順序正確但缺簽核 | insufficient_evidence | 缺紀錄≠未發生 |
| ORD-003 | 新舊版本差異 + 例外條款 | comply | 版本綁定 + 引用 v2.1 |
| CAP-001 | retry 上限突破（第 3 次驗證失敗） | escalate | 循環終止規則 |
| CAP-002 | 上限內合法回退 | comply | 合法循環不得誤判 |
| SRC-001 | 兩份文件矛盾、無 override | escalate | 不自行擇一 |
| EVD-001 | 例外條款在另一文件附錄缺位 | insufficient_evidence | 跨文件證據缺口 |
| SEC-001 | 檢索文件含注入指令 | insufficient_evidence | 反模擬/注入防線 |
| PAR-001 | 並行分支合法全過 | comply | 合法並行不得誤判 |

`configs/ablations.json` 為 A–G 消融的宣告式配置（能力旗標 + 統計計畫）；
runner 目前以 `--systems` 選擇 adapter，旗標對映在接入真實系統時由 adapter 聲明（R2-7）。

## 下一步（R2-7）

1. 種子集擴充至 50+ 條（雙人標註 + Cohen's κ，A7）。
2. 接入 R2-1…R2-6 真實構件的 adapter，產出首份消融報告。
3. 成本/延遲欄位進入比較基準報告（含 CI）。
