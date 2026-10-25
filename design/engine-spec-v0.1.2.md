# 引擎規格 v0.1.2 — 裁決語義決定表

> 配套 `rule-schema-v0.1.1.schema.json`（**不新增 schema 欄位**；本版重新定義既有欄位的
> 完整語法）。每一條決定編號 D-xx，寫明動機、公理/門檻映射與驗證鉤子（考卷案例 ID）。
> 本文件可追溯：任何引擎行為都能回推到一條 D 決定；任何 D 決定都具有考卷案例為其真機證據。
> 參照硬門檻表：`docs/tech-selection-round1.md` §5（H-1..H-6）。

## 0. 與 v0.1.1 的關係

v0.1.1 schema 已含本版所需全部欄位；v0.1.2 只把「語法空缺」收攏為明文語義：

| 既有欄位 | v0.1.2 新增語義 | 重量：D 決定編號 |
| --- | --- | --- |
| `sequencing.deadline` | 錨點事件存在性 + ISO-8621 時長超期判定 + `on_violation` 三態處置 | D-04/D-05/D-14 |
| `sequencing.forbidden_after` | subject 完成後禁入活動（ts 嚴格大於） | D-06 |
| `sequencing.on_violation` | 超期/違禁處置分流：block→violate；escalate→escalate；warn→note 保 comply | D-05 |
| `evidence_requirements[].missing_evidence_result=partial_pass_note` | 缺口升 note 不降判定（規則作者治理聲明） | D-07 |
| `subject.actor_role` | 主體資格不匹配 → escalate（審慎不裁越權） | D-08 |
| `subject.scope.processes` | 規則作用域與案件匹配流程不一致 → NotApplicable + 引用 | D-09 |
| `temporal.effective_to` | 規則有效期屆滿 → insufficient（保守，不擅自找替代規則） | D-10 |
| `exceptions.overridden_by/overrides` | 覆寫鏈存在 → 衝突不 escalate，覆寫者勝 + 雙方引用保留 | D-11 |
| `decision.method/missing_evidence_result` | 未知 evidence_type 一律保守 insufficient（已實作） | D-12 |

以下決定表逐條展開。

---

## 1. 決定表

### D-01 版本時間引擎（保持 R2 語義）
宣告綁定 `(doc_id, version)` 於案件**最早事件時點** `earliest` 判定 `effective_from ≤ earliest ≤ effective_to`。
失效綁定僅記 note（`declared_version_not_in_force`），不自行改判——判定的唯一路徑是規則/流程/證據三條，時間失效只會透過規則層 D-10 落地。
- 動機：拒絕「時間失效→自動改判」的推理跳躍（缺 ≠ 沒發生的姊妹原則：過期 ≠ 違規）。
- 證據鉤子：BC-101（stale pool）、BC-122（註記不升級）。

### D-02 規則解析三態（保持 R2）
`RULE-X`（無 pin）在案件 `doc_versions` 內解析；`RULE-X@v` 以 `v` 釘**來源文件版本**。
0 候選帶 pin → `PinnedMismatch`；0 候選無 pin → `Unlocatable`；皆 → insufficient（保守拒答）。
規則庫**不存在**該 rule_id 時，note 顯式區分「rule_id 未登錄」。
- 新增：資料層唯一性門禁——`(rule_id, rule_version)` 重複即載入失敗（防止同名不同義覆蓋，偏愛顯式版本而非靜默後者勝）。
- 證據鉤子：BC-160（未登錄 rule_id）、BC-161（重複引用去重）、BC-162（draft 規則隱形）。

### D-03 裁決優先序（保持 R2，全鏈條重申）
`escalate > violate > insufficient_evidence > comply`（RuleVerdict 級別 4>3>2>1>0）。
流程層 Deviation ≤ 規則層最高判定時不降不升；僅當規則層最高判定 < violate 時，Deviation 才把判定推到 violate（H-1 保護帶：把規則層漏掉的程序違規抓回來）。
- 證據鉤子：BC-150/151/153（流程層抓重複與未知活動）、BC-190（雙層同時違規，citations 合併）。

### D-04 deadline 錨點語義
`deadline { anchor: activity_start|activity_end|event, ref: <activity_name>, duration_iso }`。
- 錨點事件存在性：ref 活動無觀測事件 → 證據缺口（insufficient，保守拒答，不推測超期）；
- 基準時點：`activity_start` 取該活動**最早**事件 ts；`activity_end`/`event` 取**最晚**事件 ts；
- subject 基準時點取其活動**最早**事件 ts；
- 超期判定：`subject_ts − anchor_ts > duration_iso` →逾期（嚴格大於，等於不觸發）。
- ISO-8601 時長解析（近似日曆語義，D-14）：`P[nY][nM][nW][nD][T[nH][nM][nS]]`；
  換算 Y=365d、M=30d、W=7d、H=3600s、M(分)=60s、S=1s；解析失敗 → 引擎 note + insufficient（設定參數劣值不得靜默通過）。
- 證據鉤子：BC-110..114。

### D-05 超期處置走 `sequencing.on_violation`
超期 → `block` → Violate（判定 vi）；`escalate` → Escalate；`warn` → note 保 Comply。
- 動機：處置是規則作者的治理聲明，引擎不越權改類；同一超期事實在不同治理 SOP 下對策不同（變更執行逾期=程序違規；財務逾期=升級裁定）。
- 引用：三態一律引用規則主條款 citation；warn 態 note 標記 `deadline_overdue_warned`。
- 證據鉤子：BC-110/112/194。

### D-06 `forbidden_after` 語義（規則層）
subject（其活動之**最晚**事件 ts 基準 `subject_ts_max`）完成後，`forbidden_after` 清單活動若有事件 `ts > subject_ts_max`（**嚴格大於**；等於視為同批並行，不罰），→ 違禁事實確立 → 依 `on_violation` 三態分流（同 D-05）。
- 動機：資料來源是 OCEL/XES 事件序；`ts` 爽約即為「完成後再發生」。等點放行＝尊重多事件同秒的自然批次不確定性。
- 證據鉤子：BC-115/116/117。

### D-07 `partial_pass_note` 語義
`evidence_requirements[i].missing_evidence_result == "partial_pass_note"` 且該項證據不滿足時：
判定**不降級**（不受此缺口拉到 insufficient），但 note 必記 `evidence_gap_noted: <evidence_type>`。
- 防線：這是**規則作者的治理聲明**（schema 明文允許該枚舉值），引擎僅執行；H-3 的量測對象仍是測試集案件（expected=insufficient 被判 comply 才會失守），考卷已明確此語義案件的態樣下覆蓋率為 100%。
- 證據鉤子：BC-142（缺第二簽 → comply + note）、BC-143（雙簽齊 → comply 淨）。

### D-08 主體資格語義
`subject.actor_role != null` 時，subject 活動在場任一事件 `actor == actor_role` 才算資格成立；
不成立 → engaged 但 verdict = **Escalate**（引用主條款），note `subject_actor_mismatch`。
- 動機：引擎無憑據裁「越權」（需 HR 授權表佐證）→ 審慎送人工；同時不上 comply（漏報代價大）。
- 證據鉤子：BC-120/121/122。

### D-09 流程作用域語義
`subject.scope.processes` 非空時，必須包含案件匹配到的 process_id（match_for D-25），否則該規則 NotApplicable（note + 引用保留）。
- 動機：規則的管轄邊界聲明；跨流程誤用屬治理噪音，不是違規。
- 證據鉤子：BC-123。

### D-10 規則有效期屆滿
`temporal.effective_to` 已過案件時點且 `superseded_by == null` → Insufficient + note `rule_expired`。
「到期後行為是否違規」須由新版本規則承接，引擎拒絕代裁（保守管道）。
- 證據鉤子：BC-105（屆滿）、BC-106（期內）。

### D-11 override 鏈語義
衝突對判定改為：**任一方帶有覆寫宣告（overrides 或 overridden_by 非空）→ 不 escalate**。
- 保留雙方 engaged 判定與雙方主條款引用；覆寫宣告登記為 note `override_rendered: A overridden_by B`；
- 全新語義仍未實作「覆寫者單獨勝出」的判定覆寫（R3 登始初版：覆寫鏈存在 → 不送 escalate，由覆寫規則自己的評估結果代表管轄見解）。
- 無覆寫宣告的雙向 conflicts_with → escalate（R2 語義保持：不自行擇一）。
- 證據鉤子：BC-163（無覆寫 → escalate 雙引用）、BC-164（帶覆寫 → comply 雙引用 + note）。

### D-12 未知證據類型 → 保守 insufficient（保持 R2）
葫蔖中無法建模的 evidence_type 一律按證據缺口處理，note 顯式聲哑（隱式通過是漏報，العكس不是）。
- 證據鉤子：單元測試覆蓋（unknown type）。

### D-13 附錄定位綁定一致性（R2 可選收緊點落地）
`appendix_text` 命中必須 `hit.doc_id@hit.version == record.source.document_id@record.source.document_version`，
否則視為**不可定位**（即便相似度高）。防止 A 條款文本相似段落替充 B 規則的跨文件引用欺�。
- 證據鉤子：BC-144（EXB 規則訂在 SOP-CHANGE@2.1 但 anchor 文本在 EX-7——即相似也判 insufficient）、
  BC-145（同 anchor 規則若訂在 EX-7@2.0 → hit doc/version == 綁定 → comply）。

### D-14 ISO-8601 近似日曆語義（量測公理透明化）
Y/Mo/W 換算採固定近似（365/30/7 天），適用於「治理 SOP 層級」的粗粒度時限；
需要精密日曆（月末日、閏年）的期限不應以本引擎為最終裁定（note 提示），屬 R3+ sidecar 范畴。
- 動機：確定性重放優先——不引入任何 timezone/日曆庫的不確定性進判定路徑。
- 證據鉤子：BC-114（等於不觸發）。

### D-15 loop 規則的資料完整性
loop_policy 規則 engaged 但 `case.loop` 缺失（或 retries 不可解析）→ Insufficient + note `loop_state_missing`（保守拒答，不假設 retries=0）。
記既有：retries 与牌 policy.failures_cap 比較；case `loop.cap` 與 policy.failures_cap 不一致僅 note（治理偏差留審計）。查D-16
### D-16 循環規則與 case `loop.cap` 的信任序
policy.failures_cap 為**引擎執行量**；case.loop.cap 不一致時僅 note（審計），判定依 policy。
- 動機：cap 變更屬規則生命週期（schema v0.1.1 §2），不屬案件配置。
- 證據鉛子：BC-132/134。

### D-17 案件 `doc_versions` 中未 declared 規則不參與（顯式聲明主義）
判定範圍＝case.rules 顯式引用之 active 規則；庫內其他規則靜默不參。
- 動機：稽核可解釋性（auditable scope），避免隱式規則引入不可重放的影響。
- （無專屬考卷；BC-123 間接覆蓋 scope 預設。）

### D-18 流程層 repeatable 語義（新；真機考卷後修訂 v0.1.2-r1）
`repeatable = false` 模型的重複觀測缺陷，以「活動×主體」配對判定：
- **(a) 同主體重複**：同一活動由同一 `actor` 觀測 ≥ 2 次 → 一律 Deviation（重複執行）；
- **(b) 多主體重複**：同一活動多筆**不同主體**觀測，僅當活動未列入模型
  `repeat_parties`（多簽白名單，預設空）時為 Deviation；
  列入者視為治理合法的多簽（如 approve_change 雙簽）。
- `repeatable = true`（循環）整體不受此限。
- 配套（同節）：列入 `repeat_parties` 的活動在 precedence 配對中視為**同一環
  節**的多主體記錄——全體記錄共用第一次出現的前置配對，不逐簽重複要求前置。
- 動機：V0.1.1 replay 四分類只覆蓋順序；真機考卷 BC-141 證明「一刀切次數」
  會把治理合法的雙簽誤判為缺陷（假報），而 executor 連簽兩次必須仍為缺陷。
- 事件重放憑據（`events_sha256`）一併納入 actor 欄（活動|時點|主體）。
- 證據鉤子：BC-150/151/152/196（缺陷），BC-141/143（雙簽合法）。

### D-19 未知活動（模型外）→ Unconfirmed 僅登記（保持 R2）
`Cannot confirm` ≠ 違規：件證不減判定。
- 證據鉤子：BC-153、BC-196（與 Deviation 並存時 Deviation 主導）。

### D-20 evidence `event_activity/approval_record` 計數語義（保持）
`count(events 滿足 metadata_constraints) ≥ min_count`；metadata 未建模欄位/劣值一律不滿足（保守）。
`approval_record` 的活動語源依 D-27 錨定。
- 證據鉤子：BC-140（未在白名單 actor）、BC-180（空 actor）。

### D-21 引用格式（全鏈條保持）
`{doc_id}@{doc_version} §{section}`（附加 ` §{subsection}` 於 loop_policy 的 doc_ref/first_iteration_ref）；
insufficient 判定引用清空（abstain；establihed R2）；其餘判定去重排序後輸出。
- 證據鉤子：所有 BC 案件 expectations。

### D-22 snapshot manifest 語義（保持 R2，加註 A6 完整性缺口）
manifest = case_id + verdict + citations + rules_used + doc 版本內容 SHA + 規則記錄 SHA + 事件序列 SHA 的 canonical JSON SHA-256。
**未入**：模型版本/提示模板（本引擎無 LLM，路徑全部確定性 → A6 該兩項以「無適用元件」狀態登記）。
- 證據鉤子：integration `snapshot_manifest_stable_across_rebuilds` + selftest H-6。

### D-25 規則所屬流程匹配（match_for 簡約法保持）
活動覆蓋率最高者勝；平手取 process_id 字典序最小。多流程一致性驗證 R3+。

### D-26 引擎信任路徑分開（無 LLM）
判定路徑全確定性；任何機率成分進入判定前必須：
(1) 版本化，(2) 可快照雜湊，(3) 經 H-6 重放驗證——目前均無此類成分，note 聲明即審計完備。

### D-27 `approval_record` 證據的活動語源錨定（新；真機考卷後修訂）
核准型紀錄的合格事件必須**活動 ∈ subject_activities ∪ sequencing.required_before**，
metadata 約束僅作附加過濾。僅按 metadata（actor_role）過濾會把無關活動上
恰有合格角色的觀測（如 risk_assessment_completed 上的 QA_manager）計入核准
證據——BC-140 真機重現的「不足判合規」漏報缺口，本節封閉之。
- 語源：approval_record 的本義是「前置環節之批准事件留下的紀錄」；
  required_before 即該環節的顯式宣告。
- `event_activity` 型證據維持純 metadata 計數（活動由 constraints 表達）。
- 證據鉤子：BC-140（risk(QA) 不再冒充核准紀錄 → insufficient）。

---

## 2. 與市面系統的能力對照聲明（收段）

本引擎的四特徵組合在市面常見工具中**無同時具備者**：

1. 每一判定可回推：規則 SQL-like 命中 → 條款錨點 → 文件版本 → 內容 SHA（單向雜湊鏈）；
2. 禁止性轉移放行率 = 0 的硬門檻（H-1）由流程 replay + 規則雙層把關；
3. 拒答為一等公民且引用清空（abstain），漏報類錯誤零容忍；
4. 無 LLM 判定路徑 → manifest 兩次重放必等（H-6）。

对照单位的详细可查證依據將在 `docs/competitive-position.md`（R3 後段）以
「已登錄 SEC 匯總」模式逐項填寫——本節僅做能力聲明，於 vendor 文件逐站核實前不勾銷別名。