# EKPI 評測骨架（Ablation Harness）

> 對應 Master Prompt v1 §九 + Addendum A5/A7。
> **規範性實作：Rust crate `ekp_eval`**（`#![forbid(unsafe_code)]`；serde/serde_json 為僅有依賴）。
> Python 版降為 `reference-python/` 參考實作，非規範性。

## 執行狀態（2026-11-03）

- 固定 67 案 verdict corpus 已可一次載入、去重、驗證，並由 `--corpus-selftest` 全量執行；此命令已納入 canonical `scripts/check.ps1` / `check.sh`。
- `--selftest` 仍以 9 案種子集驗證 adapter 契約；`--corpus-selftest` 另外驗證 67 案 corpus、四類 verdict 覆蓋、盲化標註表與三個 stub 的硬門檻。
- `--kappa`、`--make-annotation-sheet` 與 `eval/annotation/` 研究協定已就緒。
- **雙人標註 κ 尚未測量**：0/2 位標註者、0 份完成檔案。工具存在不等於 agreement 存在。
- **CI 尚未執行**：workflow 已定義且 canonical 入口已包含 corpus selftest，但此 repo 沒有 remote/runner；不得把 workflow 檔或本機執行稱為 CI PASS。

## 指令

```powershell
cd eval
cargo run -- --selftest                    # 9 案 adapter 契約自檢
cargo run -- --corpus-selftest             # 67 案固定 corpus 全量自檢
cargo run -- --out report_out              # 種子集報告（report.json + report.md）
cargo run -- --corpus cases/corpus.json --out target/corpus-report
cargo run -- --systems stub:oracle --out r # 只跑指定 adapter
cargo run -- --make-annotation-sheet annotation/blinded-cases.jsonl
cargo run -- --kappa annotation/annotator-a.jsonl annotation/annotator-b.jsonl
cargo fmt ; cargo clippy --all-targets -- -D warnings   # 提交前必跑（工作區規則）
```

`--kappa` 退出碼：`0` = 達門檻、`1` = 低於門檻、`2` = 輸入無效或未執行。
`--corpus-selftest` 在 corpus 少於 50 案、case id 重複、verdict 類別缺漏、盲化表洩漏標籤或硬門檻未觸發時退出 `2`。

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

## 固定 corpus（`cases/corpus.json`，67 案）

`cases/corpus.json` 是唯一入口；它宣告三個 JSONL 檔、`minimum_cases: 50`，載入時拒絕重複 case id。總數不得低於 50。

| 檔案 | 案數 | 角色 |
| --- | --- | --- |
| `cases/seed_cases.jsonl` | 9 | Adapter 契約與預設 `--selftest` |
| `cases/casebook_v3.jsonl` | 46 | 擴充裁決 corpus |
| `cases/edge_cases.jsonl` | 12 | 邊界、拒答與對抗案例 |
| **合計** | **67** | `--corpus-selftest` 全量執行 |

Verdict 分布：`comply` 28、`violate` 11、`escalate` 10、`insufficient_evidence` 18。
corpus 是合成資料，版本由 Git 固定；不得為了讓某次執行變綠而刪除失敗案例。

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
runner 目前以 `--systems` 選擇 adapter，旗標對映在接入真實系統時由 adapter 聲明。

## 雙人標註與 Cohen's κ（M-11）

- 工具：`src/annotation.rs` 的 `evaluate_study` 會拒絕少於 50 案、同一 annotator id、兩邊 case 集合不一致、重複 id、未知 verdict 與 undefined κ。
- 盲化表：`annotation/blinded-cases.jsonl` 由 `--make-annotation-sheet` 從 corpus 產生，只含 `case_id`、`task`、`doc_versions`、`events`、`rules`、`loop`；不含 `expected_verdict`、`notes`、`citations`、`category`、`abstain_required`。
- 協定與 rubric：`annotation/README.md`；record schema：`annotation/annotation.schema.json`。
- **現況：NOT MEASURED。** 尚未指派兩位獨立標註者，也沒有 `annotator-a.jsonl` / `annotator-b.jsonl`。單一作者、AI 標註或同一人的兩次標註都不構成 inter-annotator agreement。
- 提議門檻 κ ≥ 0.80；仍需 product owner 簽核才成為 release gate。

## R2-7 狀態（2026-11-03）

| 項目 | 狀態 | 證據／缺口 |
| --- | --- | --- |
| 50+ 測試案例 | **完成（可執行）** | 67 案；`--corpus-selftest` 全量執行並納入 canonical gate；corpus manifest 拒絕重複 id 與低於 50 案 |
| 雙人標註 κ | **工具與協定就緒；未測量** | `--kappa`、盲化表、schema、rubric 已完成；0/2 標註者，κ 無數值 |
| CI 接入 | **workflow 已接線；未執行** | `.github/workflows/ci.yml` 呼叫 `scripts/check.sh` / `check.ps1`，兩者已含 corpus selftest；無 remote/runner，故無 CI run ID |

R2-7 不能標記為「全部完成」，因為 κ 與 CI execution 兩項仍缺外部條件。詳見 `docs/quality/ai-evaluation.md` 與 `docs/tech-selection-round1.md`。

## 下一步（外部條件）

1. 指派兩位獨立標註者，依 `annotation/README.md` 完成 67 案並以 `--kappa` 記錄結果。
2. 設定 git remote 與 CI runner，實際觸發 workflow；在取得 run ID 前不得聲稱 CI PASS。
3. 接入 R2-1…R2-6 真實構件的 adapter，產出首份消融報告。
4. 成本/延遲欄位進入比較基準報告。
