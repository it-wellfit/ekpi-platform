# Matt Pocock Skills 安裝與 repo 設定

本 checkout 以 **OpenCode** 執行，技能放在 `.opencode/skills/<skill-id>/SKILL.md`，不是 Codex 插件。以下記錄實際安裝方式、技能清單、更新方法，以及本 repo 已完成的三項 setup 決策。

## 安裝現況（2026-10-26）

- 位置：`.opencode/skills/`
- 數量：27 個技能＝engineering 20＋productivity 7；27 份 `SKILL.md` 全部具備 `name` 與 `description` frontmatter。
- 來源：[mattpocock/skills](https://github.com/mattpocock/skills) 的 `main` 分支，逐檔自 `raw.githubusercontent.com` 下載。
- 原因：本機沒有 Node.js／npm，官方 `npx skills@latest add mattpocock/skills -a opencode` 無法執行。若日後裝好 Node，仍建議改用官方 CLI 取得版本管理與更新流程。
- 規模：27 檔合計 129,220 bytes，無空檔或異常小檔。

### engineering（20）

ask-matt、code-review、codebase-design、diagnosing-bugs、domain-modeling、grill-with-docs、implement、implement-spec、improve-codebase-architecture、pr、prototype、research、retro、setup-matt-pocock-skills、tdd、to-spec、to-tickets、triage、wayfinder、wizard。

### productivity（7）

grill-me、grilling、handoff、teach、to-questionnaire、wait-what、writing-for-agents。

`disable-model-invocation: true` 的技能只接受手動 `/name` 呼叫，共 16 個：ask-matt、grill-me、grill-with-docs、handoff、implement、implement-spec、improve-codebase-architecture、retro、setup-matt-pocock-skills、teach、to-questionnaire、to-spec、to-tickets、triage、wait-what、wayfinder。其餘 11 個可被模型自動選用。

`setup-matt-pocock-skills/` 另含 `domain.md`、`triage-labels.md` 與 GitHub／GitLab／local 三份 issue-tracker 範本；本 repo 已依其輸出建立 `docs/agents/` 下的設定。

## 驗證安裝

在 repo 根目錄執行：

```powershell
$files = Get-ChildItem .opencode\skills -Recurse -Filter SKILL.md
"COUNT=$($files.Count)"
foreach ($f in $files) {
    $head = Get-Content -LiteralPath $f.FullName -TotalCount 8 -Encoding UTF8
    if (-not ($head -match '^name:\s*\S')) { "MISSING name: $($f.FullName)" }
    if (-not ($head -match '^description:\s*\S')) { "MISSING description: $($f.FullName)" }
}
```

預期 `COUNT=27`，且沒有任何 `MISSING` 行。OpenCode 重新載入後可用 `/ask-matt` 確認技能可被叫用。

## 更新技能

有 Node.js 時，優先用官方 CLI：

```powershell
npx skills@latest add mattpocock/skills -a opencode
```

沒有 Node.js 時，重新下載每份 `SKILL.md`。類別清單如下，請保持相同目錄結構：

```powershell
$engineering = @('ask-matt','code-review','codebase-design','diagnosing-bugs','domain-modeling',
    'grill-with-docs','implement','implement-spec','improve-codebase-architecture','pr','prototype',
    'research','retro','setup-matt-pocock-skills','tdd','to-spec','to-tickets','triage','wayfinder','wizard')
$productivity = @('grill-me','grilling','handoff','teach','to-questionnaire','wait-what','writing-for-agents')

foreach ($name in $engineering) {
    $dir = ".opencode\skills\$name"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    curl.exe -fsSL "https://raw.githubusercontent.com/mattpocock/skills/main/skills/engineering/$name/SKILL.md" -o "$dir\SKILL.md"
}
foreach ($name in $productivity) {
    $dir = ".opencode\skills\$name"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    curl.exe -fsSL "https://raw.githubusercontent.com/mattpocock/skills/main/skills/productivity/$name/SKILL.md" -o "$dir\SKILL.md"
}
```

更新後重跑上面的驗證，並在 commit 前檢視 diff；技能更新可能改變工作路由或要求新的 repo 檔案。

## Repo setup 決策

1. **Issue tracker：GitHub Issues**。工單與 spec 都走 GitHub Issues，使用 `gh issue create` 等命令；不建立本地 Markdown 工單系統。設定與 bootstrap 狀態見 `issue-tracker.md`。
2. **Triage 標籤：五個 canonical 名稱**。`needs-triage`、`needs-info`、`ready-for-agent`、`ready-for-human`、`wontfix`；對映與尚未 provision 的狀態見 `triage-labels.md`。
3. **Domain docs：單一 context**。根目錄 `GLOSSARY.md` 加 `docs/adr/`；consumer 規則見 `domain.md`，產品規格仍以 `docs/governance/` 為權威。

## 技能入口速查

- 不確定走哪條路：`/ask-matt`
- 需求釐清：`/grill-with-docs`；通用訪談 `/grilling`、`/grill-me`
- 規格與工單：`/to-spec`、`/to-tickets`
- 實作與測試：`/implement`、`/implement-spec`、`/tdd`
- 審查與交付：`/code-review`、`/pr`
- 疑難排查：`/diagnosing-bugs`；事後改善 `/retro`
- 研究與領域：`/research`、`/domain-modeling`
- 架構改善：`/codebase-design`、`/improve-codebase-architecture`
- 其他：`/prototype`、`/wayfinder`、`/triage`、`/handoff`、`/wizard`、`/teach`、`/to-questionnaire`、`/wait-what`、`/writing-for-agents`
- 一次性設定：`/setup-matt-pocock-skills`

## 歷史備註：Codex 安裝法

本 repo 最初由另一個會話以 Codex 設定，舊文件記載：

    codex plugin marketplace add mattpocock/skills
    codex plugin add mattpocock-skills@mattpocock

本 checkout 不使用 Codex，以上指令不適用，保留僅為追溯來源。若日後改回 Codex，插件版可與 `.opencode/skills/` 並存，但 repo 設定只需維護一套 `docs/agents/`。
