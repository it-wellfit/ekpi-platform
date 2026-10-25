# 研究登錄表：欄位定義與更新作業流程

## `papers.csv` 欄位

| 欄位 | 說明 |
| --- | --- |
| `paper_id` | 短 slug 主鍵（如 `raptor-2024`），引用時以此為錨點 |
| `title` | 官方標題（以出版頁為準，不用部落格轉述名） |
| `venue` | 會議/期刊與卷別（例：PMLR v306 = ICML 2026 首爾） |
| `year` | 出版年 |
| `type` | `peer_reviewed / preprint / open_source_prototype / official_doc / vendor_doc` |
| `doi` | 有則必填；Elsevier PII 可經 Crossref `alternative-id` 反查 DOI |
| `url` | 官方出版頁（Doi 連結優先） |
| `code_url` | 官方/作者 repo（存在才填） |
| `benchmark_claims` | 該文報告的主要結果，摘要一句 |
| `claim_source` | `peer_reviewed / author_reported / vendor_reported / not_applicable` |
| `verified` | `yes / no` — 只有實際抓取/交叉核對過才可 `yes` |
| `verified_date` | 核實日（本輪為 2026-10-25） |
| `verification_method` | `official_page_fetch / crossref_pii / openalex / websearch_cross_check` |
| `relevance_layer` | 對應八層地圖（L1 解析 / L2 索引 / L3 圖 / L4 規則 / L5 流程 / L6 因果 / L7 驗證執行 / L8 證據治理） |
| `adoption` | `adopt / adopt_as_baseline / adopt_with_care / watch / research / research_only` |
| `notes` | 風險、限制、次要來源警告 |

## `standards.csv` 欄位

同上結構，`purpose_in_platform` 描述該標準在平台的分工位置；`version` 欄需注意版本交替
（實例：XES 已由 IEEE 1849-2016 **改版為 1849-2023**，引用時以 2023 版為準）。

## 新增引用 SOP

1. 取得官方出版頁或 DOI（Elsevier 文章可用 PII 走 Crossref `filter=alternative-id:PII`）。
2. OpenAlex 做第二來源交叉（記 `openalex`）。
3. 把 `benchmark_claims` 與 `claim_source` 分開記錄：**論文正文結果 ≠ 作者在自家基準上自報的數字 ≠ 廠商宣傳**。
4. 抓不到（DNS/TLS/HTTP 錯誤）→ 標 `verified=no` 並註明原因；此時該條**不得**出現在任何正式報告中。
5. 每季复查 `adoption=watch` 的條目，升降級需在 `notes` 記錄理由。

## 本輪（2026-10-25）核實摘要

- 論文 18 條：全數 `verified=yes`（其中 2 條經 Crossref PII / OpenAlex 補驗，出版頁被反爬蟲擋下）。
- 標準 11 條：10 條 `verified=yes`；另新增 1 條 **Mojo（含 MAX）標 `verified=partial`**——
  `modular.com/mojo` 官方頁存在（title "Mojo"，含 MAX/GPU），但 `mojo.modular.com` DNS 未解析，
  授權／穩定版本／可重現建置三項未核實，定為 **research-only**，禁入 compliance-critical 路徑。
  `tf-pm.org` 在本機 TLS 異常，改以 `standards.ieee.org` 與 websearch 交叉核實；
  `legalruleml.oasis-open.org` DNS 未解析，經 websearch 核實 OASIS docs 與 TC 頁（v1.0 已是 OASIS 正式標準，2021-08-30）。
- 心得：對 Elsevier（ScienceDirect）一律直接走 Crossref PII；對 OASIS/IEEE 用標準入口頁交叉；
  廠商新語言（Mojo 類）先抓主站再驗 DNS/授權，兩者其中之一不可核實即回落 partial。

## Round 3 增補（2026-10-25）核實摘要

- 論文 +9 條（合計 27 條，全數 `verified=yes`、`verified_date=2026-10-25`）：
  arpaccino-2025（arXiv 2507.10584，official_page→arxiv_abs_fetch）、
  citeeval-2025（ACL 2025 Long pp.32759-32778）、garage-2025（ACL 2025 Findings pp.17030-17049）、
  satgraph-2025（JURIX 2025 / arXiv 2505.00039，DOI 10.3233/FAIA251598）、
  ijcai25-declare-2025（IJCAI-25 pp.4500-4508，DOI 10.24963/ijcai.2025/501；正式標題為
  《Are Large Language Models Fluent in Declarative Process Mining?》）、
  kaist25-legalrag-2025（CIKM 2025 pp.1396-1405，DOI 10.1145/3746252.3761151）、
  schnitzhofer25-dtal-2025（NLL2FR 2025 @ JURIX 2025 pp.157-163，JKU Linz 出版頁核實）、
  wimmer26-nlpbpm-2026（BPM 2025 Workshops LNBIP 569 pp.425-437，DOI 10.1007/978-3-032-13426-4_31）、
  formalmethods-sosym-2025（SoSyM 24:271-272 editorial，DOI 10.1007/s10270-025-01287-0）。
- `standards.csv` 無新增：LegalRuleML 1.0 與 BPMN 2.0 均已登記核實，本批文獻未引入新標準依憑。
- 一名候選（「Wallat」相關條目）經兩輪 websearch 無法對應任何官方源，判定為早輪誤記，**不登記**。
