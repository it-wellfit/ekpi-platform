# EKPI 規則覆蓋率報告（真機統計）

此報告由 CLI `--coverage-report` 於本次真機執行生成：兩個案例源逐案呼叫判定引擎，數字不是常數。

| 案例源 | 案數 |
|---|---|
| seed_cases.jsonl | 9 |
| casebook_v3.jsonl | 46 |
| 合計 | 55 |

## 每條 active 規則的覆蓋

| rule_id | 條款錨點（doc@ver §sec） | 觸及案數 | verdict 分布 | 觀測引用（錨點∩引用池） |
|---|---|---|---|---|
| RULE-CAPA-012 | SOP-CAPA@1.4 §6.1 | 9 | comply=3; escalate=5; insufficient_evidence=1 | SOP-CAPA@1.4 §6.1 |
| RULE-CHG-001 | SOP-CHANGE@2.1 §4.3; SOP-CHANGE@3.2 §4.3 | 18 | comply=6; escalate=1; insufficient_evidence=5; violate=6 | SOP-CHANGE@2.1 §4.3; SOP-CHANGE@3.2 §4.3 |
| RULE-CHG-002 | SOP-CHANGE@3.2 §5.1 | 3 | comply=1; violate=2 | SOP-CHANGE@3.2 §5.1 |
| RULE-CLOSE-GATE | SOP-CLOSE@1.0 §3.1 | 3 | comply=2; violate=1 | SOP-CLOSE@1.0 §3.1 |
| RULE-DRF-004 | SOP-REL@1.0 §3.1 | 3 | comply=1; insufficient_evidence=2 | SOP-REL@1.0 §3.1 |
| RULE-EX-007 | EX-7@2.0 §7.2 | 2 | comply=1; insufficient_evidence=1 | EX-7@2.0 §7.2 |
| RULE-EXB-002 | SOP-CHANGE@2.1 §4.3 | 1 | insufficient_evidence=1 | SOP-CHANGE@2.1 §4.3 |
| RULE-FUT-005 | SOP-REL@1.0 §2.1 | 2 | comply=1; insufficient_evidence=1 | SOP-REL@1.0 §2.1 |
| RULE-PAY-001 | SOP-PURCH@4.5 §3.2 | 1 | comply=1 | SOP-PURCH@4.5 §3.2 |
| RULE-PAY-002 | SOP-PURCH@4.5 §3.3 | 1 | comply=1 | SOP-PURCH@4.5 §3.3 |
| RULE-PAY-003 | SOP-PURCH@4.5 §3.4 | 1 | escalate=1 | SOP-PURCH@4.5 §3.4 |
| RULE-PAY-004 | SOP-PURCH@4.5 §3.5 | 1 | escalate=1 | SOP-PURCH@4.5 §3.5 |
| RULE-PUR-002 | SOP-PURCH@4.4 §3.1 | 3 | comply=1; escalate=2 | SOP-PURCH@4.4 §3.1 |
| RULE-QM-014 | QM-01@5.0 §2.4 | 1 | escalate=1 | QM-01@5.0 §2.4 |
| RULE-REL-002 | SOP-REL@1.0 §2.1 | 5 | comply=2; insufficient_evidence=1; violate=2 | SOP-REL@1.0 §2.1 |
| RULE-REL-003 | SOP-REL@1.0 §2.2 | 1 | escalate=1 | SOP-REL@1.0 §2.2 |
| RULE-REL-006 | SOP-REL@1.0 §2.2 | 1 | comply=1 | SOP-REL@1.0 §2.2 |
| RULE-SIG-002 | SOP-CHANGE@3.2 §4.3 | 2 | comply=2 | SOP-CHANGE@3.2 §4.3 |

## 引擎觸及的非 active 引用 id

以下 declared 引用被引擎登記但庫內無 active 紀錄；依治理語義以顯式 note 處置並保守拒答（A1/A8：draft 規則不可消費；D-02：未登記規則不猜測）：

- RULE-CHG-001-DRAFT：1 案（insufficient_evidence=1）
- RULE-CHG-001@2.1：1 案（comply=1）
- RULE-CHG-001@3.2：3 案（comply=2; insufficient_evidence=1）
- RULE-EX-007@2.0：1 案（comply=1）
- RULE-NOSUCH-999：1 案（insufficient_evidence=1）

## 未覆蓋規則

無——每一條 active 規則皆被至少一案件觸及。

## 總結

執行案件合計 55（seed 9 + casebook 46，全部真機 decide）；active 規則 id 18 個，覆蓋 18 個（0 案觸及 0 個）；引擎觸及的非 active 引用 id 5 個；觀測引用池 16 條。
