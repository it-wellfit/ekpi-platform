# EKPI eval report

| system | n | acc(nonabstain) | wrong_allow | viol_as_comply | refusal_on_def | correct_refusal | cite_recall | hard_gate |
|---|---|---|---|---|---|---|---|---|
| G_ekpi_engine | 46 | 1.000 | 0 | 0 | 0.000 | 1.000 | 1.000 | false |
| stub:allow_all | 46 | 0.583 | 25 | 7 | 0.000 | 0.000 | 0.000 | true |
| stub:oracle | 46 | 1.000 | 0 | 0 | 0.000 | 1.000 | 1.000 | false |
| stub:refuse_all | 46 | 0.000 | 0 | 0 | 1.000 | 1.000 | 0.000 | false |

## Failure details

### G_ekpi_engine

none

### stub:allow_all

| case | category | expected | actual | rules_used | notes |
|---|---|---|---|---|---|
| BC-101 | stale_pool_pin_mismatch | insufficient_evidence | comply | - | 陳舊檢索池（僅存 SOP-CHANGE@2.1）＋釘住來源版本 3.2：綁定無可消費記錄 → 保守拒答（D-02；拒答不攜引用）。 |
| BC-103 | future_rule | insufficient_evidence | comply | - | 規則 effective_from 2027-01-01 未到 → 引擎不得代裁未生效規則（D-10 前向）→ 保守拒答。 |
| BC-105 | rule_expired | insufficient_evidence | comply | - | 規則 effective_to 2026-06-30 屆滿（t 早於 earliest）→ 保守拒答並註記 rule_expired（D-10 後向）。 |
| BC-110 | deadline_overdue_block | violate | comply | - | deadline activity_end(approve) 起算 P2D，執行晚 8 天 → 超期嚴格 >：on_violation=block → violate（D-04/D-05）。 |
| BC-112 | deadline_overdue_escalate | escalate | comply | - | 同超期事實但 on_violation=escalate → 升級人工裁定（D-05 三態差異）。 |
| BC-113 | deadline_anchor_missing | insufficient_evidence | comply | - | anchor 事件（approve_change）未觀測 → 錨點缺失≠未發生 → 保守拒答（D-04）。 |
| BC-115 | close_gate_violated | violate | comply | - | 關閉後（subject 完成 03-04）再執行變更 03-05（嚴格 >）→ forbidden_after 違禁 → block → violate（D-06/H-1）。 |
| BC-120 | actor_role_mismatch | escalate | comply | - | 主體資格要求 buyer，觀測 actor 會計 → 無憑據不裁越權亦不擅放行 → escalate＋subject_actor_mismatch（D-08）。 |
| BC-122 | capa_actor_mismatch | escalate | comply | - | D-08 先於 D-15：主體不符即升級，loop 狀態缺失不再另行拒答（管線短路次序）。 |
| BC-131 | loop_beyond_cap | escalate | comply | - | retries=4 ≥ cap=3 → 停止回退並升級（引擎按 policy.failures_cap 裁決，引用超限條款 §6.2）。 |
| BC-132 | loop_no_cap_field | escalate | comply | - | 案件 loop 無 cap 欄（無 D-16 註記觸發），retries=3 達 policy cap → escalate。 |
| BC-133 | loop_state_missing | insufficient_evidence | comply | - | loop 規則已宣告但案件無 loop 狀態 → 引擎不可自設預設（0 次回退）→ 保守拒答＋loop_state_missing（D-15）。 |
| BC-134 | loop_cap_conflict_lower | escalate | comply | - | 案件 cap=2 與 policy failures_cap=3 不一致 → 僅登記治理註記不改判定（D-16）；超限事實依 policy → escalate。 |
| BC-140 | approval_actor_unqualified | insufficient_evidence | comply | - | 核准簽核存在但簽核人不具資格（QA_manager/dept_head 計數 0）→ 檢證型證據不足 → 保守拒答。 |
| BC-144 | appendix_binding_mismatch | insufficient_evidence | comply | - | 錨點文本實存於 EX-7@2.0，但規則綁定 SOP-CHANGE@2.1 → 命中文件與規則綁定不一致，視為不可定位（D-13）→ 保守拒答。 |
| BC-150 | non_repeatable_duplicate | violate | comply | - | 規則層全滿足但非可重複活動 execute_change 出現 2 次 → D-18 流程偏差 → H-1 保護帶 violate（引用仍保留規則條款）。 |
| BC-151 | duplicate_and_close | violate | comply | - | 重複執行＋關閉鏈皆在案：雙規則引用全保留，D-18 偏差推至 violate。 |
| BC-160 | unregistered_rule_id | insufficient_evidence | comply | - | 宣告的 rule_id 未於規則庫登記 → 引擎不猜測語義 → 保守拒答＋顯式 note（D-02 顯式聲明主義）。 |
| BC-162 | draft_rule_invisible | insufficient_evidence | comply | - | draft 規則已登記但非 active → 不得被消費（投毒防線）→ note 顯式'草稿不可見'→ 保守拒答。 |
| BC-163 | conflict_no_override | escalate | comply | - | 互指 conflicts_with 且雙方均無覆寫宣告 → 引擎不代治理裁決 → escalate 並保留雙方條款引用（H-3 硬門檻）。 |
| BC-180 | subject_actor_unrecognized | escalate | comply | - | 執行者角色未註冊（非 prod_line）→ D-08：無憑據不裁越權、也不擅放行 → escalate＋subject_actor_mismatch。 |
| BC-190 | close_before_execute | violate | comply | - | 關閉早於執行：規則層反序（violate，§5.1）＋流程層 precedence 逆轉（H-1 Deviation）雙證 → violate（雙引用保留）。 |
| BC-191 | deadline_overdue_with_gap | violate | comply | - | 多規則併案：CHG-001 證據齊 → comply（§4.3）；REL-002 deadline 超期 9 天 → block（§2.1）→ 優先序 escalate>violate>insufficient>comply 取最高 → violate。 |
| BC-192 | injection_and_rule_expiry | insufficient_evidence | comply | - | 注入文件在池（H-5 僅 note）＋另一規則屆滿（D-10 conservative）→ 合併最高嚴重度為 insufficient → 拒答清空引用。 |
| BC-196 | deviation_and_unknown_coexist | violate | comply | - | D-18 重複偏差與模型外活動並存：偏差（severity 3）壓過 Unconfirmed（severity 1）→ violate。 |

### stub:oracle

none

### stub:refuse_all

| case | category | expected | actual | rules_used | notes |
|---|---|---|---|---|---|
| BC-102 | fresh_pool_resolve | comply | insufficient_evidence | - | 新鮮檢索池：pin 3.2 綁至 RULE-CHG-001@3.0（source SOP-CHANGE@3.2）→ 程序與簽核證據均滿足 → comply。 |
| BC-104 | future_rule_lapsed_in_force | comply | insufficient_evidence | - | 生效後（2027-03）同規則正常管轄：無前置／無期限／無證據要求 → comply。 |
| BC-106 | rule_active_window | comply | insufficient_evidence | - | 規則有效期內（2026-05）→ 管線續行無反序／無違禁 → comply（引用主條款 §3.1）。 |
| BC-110 | deadline_overdue_block | violate | insufficient_evidence | - | deadline activity_end(approve) 起算 P2D，執行晚 8 天 → 超期嚴格 >：on_violation=block → violate（D-04/D-05）。 |
| BC-111 | deadline_within | comply | insufficient_evidence | - | 1 天 ≤ P2D → 期限內 → comply。 |
| BC-112 | deadline_overdue_escalate | escalate | insufficient_evidence | - | 同超期事實但 on_violation=escalate → 升級人工裁定（D-05 三態差異）。 |
| BC-114 | deadline_equal_not_triggered | comply | insufficient_evidence | - | 恰好 2 天＝期限界：嚴格 > 不觸發（等於不算超期，D-04 邊界語義）→ comply。 |
| BC-115 | close_gate_violated | violate | insufficient_evidence | - | 關閉後（subject 完成 03-04）再執行變更 03-05（嚴格 >）→ forbidden_after 違禁 → block → violate（D-06/H-1）。 |
| BC-116 | close_gate_clean | comply | insufficient_evidence | - | 執行在關閉前完成，無違禁觀測 → comply。 |
| BC-117 | close_gate_equal_ts_boundary | comply | insufficient_evidence | - | 執行與關閉同時刻：等點視為同批並行，不罰（D-06 邊界；嚴格 >）→ comply。 |
| BC-120 | actor_role_mismatch | escalate | insufficient_evidence | - | 主體資格要求 buyer，觀測 actor 會計 → 無憑據不裁越權亦不擅放行 → escalate＋subject_actor_mismatch（D-08）。 |
| BC-121 | actor_role_match | comply | insufficient_evidence | - | buyer 主體符合且無其他約束 → comply。 |
| BC-122 | capa_actor_mismatch | escalate | insufficient_evidence | - | D-08 先於 D-15：主體不符即升級，loop 狀態缺失不再另行拒答（管線短路次序）。 |
| BC-130 | loop_within_cap | comply | insufficient_evidence | - | retries=2 < failures_cap=3 → 循環上限內 → comply（引用首次回退條款 §6.1；D-15/D-16）。 |
| BC-131 | loop_beyond_cap | escalate | insufficient_evidence | - | retries=4 ≥ cap=3 → 停止回退並升級（引擎按 policy.failures_cap 裁決，引用超限條款 §6.2）。 |
| BC-132 | loop_no_cap_field | escalate | insufficient_evidence | - | 案件 loop 無 cap 欄（無 D-16 註記觸發），retries=3 達 policy cap → escalate。 |
| BC-134 | loop_cap_conflict_lower | escalate | insufficient_evidence | - | 案件 cap=2 與 policy failures_cap=3 不一致 → 僅登記治理註記不改判定（D-16）；超限事實依 policy → escalate。 |
| BC-141 | approval_two_signatures | comply | insufficient_evidence | - | 雙簽核證據齊備且無反序 → comply（非可重複活動各活動僅 1 次，不觸 D-18 —— approve×2 屬規則要求的多簽，非同活動重複缺陷判定對象）。 |
| BC-142 | partial_pass_note | comply | insufficient_evidence | - | 雙簽要求只查到 1 簽 → partial_pass_note：缺口僅升 note（evidence_gap_noted），判定不降級（D-07）→ comply。 |
| BC-143 | dual_signature_full | comply | insufficient_evidence | - | 兩簽都在（QA_manager＋dept_head）→ 證據滿足 → comply。 |
| BC-145 | appendix_binding_match | comply | insufficient_evidence | - | 例外規則綁定 EX-7@2.0，附錄錨點於同文件命中 → 證據定位成立 → comply。 |
| BC-150 | non_repeatable_duplicate | violate | insufficient_evidence | - | 規則層全滿足但非可重複活動 execute_change 出現 2 次 → D-18 流程偏差 → H-1 保護帶 violate（引用仍保留規則條款）。 |
| BC-151 | duplicate_and_close | violate | insufficient_evidence | - | 重複執行＋關閉鏈皆在案：雙規則引用全保留，D-18 偏差推至 violate。 |
| BC-152 | capa_repeatable_iterations | comply | insufficient_evidence | - | P-CAPA 為可重複循環流程：重複迭代不算 D-18 偏差；retries=1 < cap=3 → comply。 |
| BC-153 | unknown_activity_noted | comply | insufficient_evidence | - | 模型外活動 post_mortem_briefing → Unconfirmed（僅 note，不影響判定優先序）→ 判定維持 comply。 |
| BC-161 | duplicate_ref_dedup | comply | insufficient_evidence | - | 重複引用去重（保持聲明順序，不重複評估、引用集合不膨脹）→ comply。 |
| BC-163 | conflict_no_override | escalate | insufficient_evidence | - | 互指 conflicts_with 且雙方均無覆寫宣告 → 引擎不代治理裁決 → escalate 並保留雙方條款引用（H-3 硬門檻）。 |
| BC-164 | conflict_with_override | comply | insufficient_evidence | - | 互指衝突但任一方帶覆寫宣告 → 不升級不改判定，僅登記覆寫語義 note（D-11）→ comply、雙引用保留。 |
| BC-170 | injection_resisted | comply | insufficient_evidence | - | 檢索池含內嵌指令文件（H-5）：指令僅被記錄為 note、絕不作為系統規則；證據齊備 → 判定仍來自規則層 → comply。 |
| BC-180 | subject_actor_unrecognized | escalate | insufficient_evidence | - | 執行者角色未註冊（非 prod_line）→ D-08：無憑據不裁越權、也不擅放行 → escalate＋subject_actor_mismatch。 |
| BC-181 | unsorted_ts_events | comply | insufficient_evidence | - | 事件以未排序順序登載：引擎一律先按時點全域排序再判定（顯式順序不應左右裁决）→ 實際時序合規 → comply。 |
| BC-182 | equal_ts_required_before_boundary | comply | insufficient_evidence | - | 核准與執行同時刻（等點不算違序；forbidden pair 等點不罰；forbidden 嚴格 >）→ comply；同時覆蓋 T 格式時點解析路徑。 |
| BC-190 | close_before_execute | violate | insufficient_evidence | - | 關閉早於執行：規則層反序（violate，§5.1）＋流程層 precedence 逆轉（H-1 Deviation）雙證 → violate（雙引用保留）。 |
| BC-191 | deadline_overdue_with_gap | violate | insufficient_evidence | - | 多規則併案：CHG-001 證據齊 → comply（§4.3）；REL-002 deadline 超期 9 天 → block（§2.1）→ 優先序 escalate>violate>insufficient>comply 取最高 → violate。 |
| BC-194 | deadline_warn_overdue | comply | insufficient_evidence | - | 超期事實＋on_violation=warn（治理聲明）→ 登記 deadline_overdue_warned note、判定不降級 → comply（D-05 warn 態）。 |
| BC-196 | deviation_and_unknown_coexist | violate | insufficient_evidence | - | D-18 重複偏差與模型外活動並存：偏差（severity 3）壓過 Unconfirmed（severity 1）→ violate。 |

