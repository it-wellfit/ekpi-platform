# reference-python（非規範性參考實作）

> **狀態：NOT RUN（降級處置）**
> 2026-10-25 創辦人指示：平台採 Rust 全端主軸（tech-selection-round1.md §0）。
> 本 Python 版 harness 降為參考實作，僅供語意對照（Python ↔ Rust 兩版共用同一套
> 指標定義與 `../cases/seed_cases.jsonl` 種子案例）。
> 規範性實作為 `eval/` 的 Rust crate `ekp_eval`（已真機驗證：selftest 10/10 PASS、clippy 零警告）。

注意事項（若要參考執行）：

1. 本機需自行安裝 Python（本機無任何 Python 執行環境；`python.exe`/`python3.exe` 僅為
   Microsoft Store 別名 stub）。
2. `ekp_eval/runner.py` 的 `DEFAULT_CASES` 原指向 `ekp_eval/cases/seed_cases.jsonl`；
   種子案例已上移為 `eval/cases/seed_cases.jsonl`，執行時需傳
   `--cases ../cases/seed_cases.jsonl` 或自行調整該常數。
3. 依工作區反模擬規範：本目錄的任何 PASS 宣稱在其被實際執行之前，均不得寫入報告。
