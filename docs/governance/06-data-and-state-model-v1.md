# 資料模型、狀態機與業務不變條件 v1.0

**用途：** 提供資料庫、API、UI 驗證與測試共用的語意契約。識別碼格式與保存政策等租戶決定不得由工程師自行推定。

## 1. 關聯模型

```text
Project 1 ── N WorkflowRun N ── 1 WorkflowTemplateVersion
   │              │
   ├── N Requirement ── N EvidenceLink N ── 1 EvidenceReference ── 1 ExternalDocumentVersion
   ├── N Stage/GateDecision ── N EvidenceReference
   ├── N QualityInspection ── 29 QualityCheckResult
   ├── N Risk / Change / Defect / CAPA
   └── N AuditEvent (事件亦可關聯其他物件)
```

多對多關係以明確關聯表保存；不得將多個文件 ID 塞在自由文字欄位作為唯一追溯機制。`ExternalDocumentVersion` 是來源文件版次的正規化參照；`EvidenceReference` 綁定專案、提交者與來源確認；`EvidenceLink` 表示該版證據證明哪項需求／交付物／關卡／QC。相同租戶、來源、外部物件、外部版本只建立一個外部版本參照；同一版可有多個用途連結，提交重試不得製造重複證據。所有物件 ID 全域唯一，所有引用須以資料庫外鍵／應用層交易保證物件存在及租戶範圍一致。軟刪除或封存不得使歷史決策無法解析。

## 2. 核心欄位字典

| 物件 | 欄位 | 型別／規則 | 建立或更新約束 |
| --- | --- | --- | --- |
| Project | `project_id` | 不可變唯一識別碼 | 系統產生；建立冪等 |
| Project | `name`, `department_id` | 非空；名稱長度上限由產品設定定義 | 組織架構選項須有效 |
| Project | `sponsor_id`, `owner_id`, `business_owner_id` | 身分目錄主體 ID | 需在租戶啟用且具適當狀態；展示姓名作快照但以 ID 為關聯 |
| Project | `risk_tier`, `classification` | 租戶受控代碼 | 建立時保存分類規則版本；政策變更不可追溯改寫舊值 |
| Project | `purpose`, `scope`, `success_criteria` | 有版本的基準欄位 | G1 前可編輯；核准後變更需變更事件 |
| WorkflowTemplate | `template_id`, `version` | ID + 正整數版本 | 已發布版本不可修改；每版保留內容雜湊 |
| WorkflowTemplateVersion | `status` | draft / in_review / published / superseded / retired | 只允許狀態機允許的轉換 |
| WorkflowStage | `stage_id`, `sequence`, `entry_criteria`, `activities`, `deliverables`, `roles`, `exit_criteria` | 每模板版內 ID 唯一；序號唯一且連續 | 發布前完成性／循環檢查 |
| GateDefinition | `gate_id`, `criteria`, `required_artifacts`, `approver_roles`, `non_waivable_controls` | 引用版本化條件 | 核准後不得被一般模板修改繞過 |
| WorkflowRun | `run_id`, `project_id`, `template_id`, `template_version` | 不可變模板版引用 | 建立時鎖定版本；升版須另建核准的遷移決議 |
| EvidenceReference | `evidence_id`, `project_id`, `source_id`, `external_object_id` | 系統 ID + 可選外部穩定 ID | 每筆指向一個文件／一版，不混多版 |
| EvidenceReference | `source_url`, `version_locator`, `external_version`, `etag`, `provider_digest` | 穩定 URI、歷史版次定位符、來源版本、可選供應方版戳／digest | 每次版本更新新增參照；不存秘密權杖；digest 僅用供應者回傳或經核准計算 |
| EvidenceReference | `purpose`, `linked_requirement_ids`, `linked_gate_ids`, `linked_qc_ids` | 關聯集合 | 關聯項目必須存在且在授權範圍 |
| EvidenceReference | `source_status`, `review_status`, `preservation_status`, `verified_at`, `verification_method` | 來源確認、內容審查、保存完整性為三個獨立欄位 | 不可用審查替代來源確認；亦不可用連結存活替代核定保存政策 |
| EvidenceLink | `evidence_id`, `target_type`, `target_id`, `purpose`, `required` | 對應需求／交付物／關卡／QC；`required` 由版本化流程定義 | 目標物件存在且與專案一致；刪除連結須用新事件記錄 |
| Review | `review_id`, `evidence_id`, `evidence_version`, `reviewer_id`, `result`, `note`, `created_at` | 每次一筆不可變決定 | 退回需說明；版本不相等時不可標為有效接受 |
| QualityInspection | `inspection_id`, `project_id`, `inspector_id`, `reviewer_id` | 檢查與獨立覆核者分欄 | 依風險政策判斷是否必須不同身分 |
| QualityCheckResult | `item_id`, `result`, `evidence_id`, `note`, `issue_id` | 固定 QC-01…QC-29；列舉值 | 每張檢查每項恰一筆；按結果驗證必填欄位 |
| Case | `case_id`, `kind`, `severity`, `owner_id`, `due_at`, `status` | kind=risk/change/defect/capa | 狀態合法性與類型專屬必填由服務端驗證 |
| GateDecision | `decision_id`, `project_id`, `run_id`, `gate_id`, `decision`, `approver_id`, `reason` | 決策引用當下流程／模板版本 | 每一次決策不可覆寫；重新提交建立新決策並關聯被取代決策 |
| AuditEvent | `event_id`, `tenant_id`, `aggregate_id`, `aggregate_version`, `tenant_sequence`, `schema_version`, `actor_id`, `occurred_at`, `action`, `target`, `outcome`, `correlation_id`, `causation_id`, `prev_hash`, `hash` | 租戶／物件識別、事件 schema 版次、伺服器 UTC 時間；追加式；canonicalization／digest algorithm versioned | 每個租戶序列及物件版本唯一；不可更新／刪除；完整性檢查失敗即告警與停止相關放行 |

建立 API 回應須回傳伺服器產生 ID、版本、時間、結果和驗證錯誤。API 不接受用戶端提供 `created_at`、核准身分或稽核雜湊作權威值。欄位長度、分頁上限、最大附件參照數等技術上限列為 API 契約設定並在 OpenAPI 中固定，不可依資料庫默認行為。

### 格式與輸入驗證基線

- 內部 ID 為不可變、不含語意的全域唯一識別碼；UI 可顯示租戶設定的人類可讀代碼，但不可把顯示代碼當主鍵或授權依據。
- 所有字串先移除頭尾空白並拒絕空白值；服務端依文字型別套用長度限制：名稱／標題最多 160 字元、使用者／部門顯示值最多 160、簡述最多 4,000、理由／審查意見最多 2,000、網址最多 2,048。若外部系統實際值超限，保存受控短值並記錄不可完整同步原因，不可靜默截斷。
- URL 僅允許 HTTPS 作正式連接器來源；試點可依已核准的本機來源政策接受 HTTP，但必須標記非正式。拒絕 user-info、非允許 scheme、空主機及含控制字元的 URL。重新導向目的地須重新檢查 scheme／網域及租戶允許清單。
- 日期輸入用 ISO 8601；儲存成 UTC instant，只有日期型期限另用 ISO `YYYY-MM-DD` 且保存其解讀時區。夏令時間／無效日期要明確拒絕。
- 列舉值保存穩定代碼，介面文案可本地化；新增／停用代碼須版本化，歷史記錄仍可用舊標籤解讀。
- Unicode 正規化使用 NFC；搜尋可做大小寫／空白標準化，但不得因正規化合併不同主體或文件識別碼。
- 所有顯示內容按文字安全輸出；使用者輸入不可被當作 HTML、SQL、搜尋運算式或提示控制指令。

## 3. 狀態機

### 專案與流程執行

```text
draft → awaiting_G1 → active → awaiting_G4 → accepted → awaiting_G5 → closed
             ↘ returned       ↘ on_hold             ↘ rejected
```

- `awaiting_G1` 只有在章程、目的、贊助人、範圍及風險分類齊全時可送審；批准後進 `active`。
- `active` 依核准流程逐階段完成；不可跳越尚未通過的必要關卡。暫停需理由、負責人及恢復條件。
- `awaiting_G4` 表示交付證據已提交；通過才可進最終驗收。拒絕／退回返回指定階段並留下決議。
- `awaiting_G5` 需移交、封存、未結責任與改善安排均符合核定規則；通過後關閉。已關閉項目以受控重開事件處理，不直接改成未關閉。
- 具體專案狀態標籤與是否允許取消、重開，列入租戶政策決定；取消仍須保存原因及審批。

每個 `WorkflowRunStage` 使用獨立狀態：`blocked`（前置未滿足）、`ready`、`in_progress`、`submitted`、`accepted`、`returned`。允許的轉換是 `blocked → ready → in_progress → submitted → accepted`；核准者可令 `submitted → returned`，負責人補正後由 `returned → in_progress`。只有流程定義明確標為可選、且前置／替代條件已核准的活動可記為 `not_applicable`；必要階段不可跳過。每次提交綁定該階段輸出版本，輸出改動後重新回到 `in_progress` 並讓相關核准失效。

每個 `WorkflowRunStage` 使用獨立狀態：`blocked`（前置未滿足）、`ready`、`in_progress`、`submitted`、`accepted`、`returned`。允許的轉換是 `blocked → ready → in_progress → submitted → accepted`；核准者可令 `submitted → returned`，負責人補正後由 `returned → in_progress`。只有流程定義明確標為可選、且前置／替代條件已核准的活動可記為 `not_applicable`；必要階段不可跳過。每次提交綁定該階段輸出版本，輸出改動後重新回到 `in_progress` 並讓相關核准失效。

### 文件參照與審查

```text
source_status: unverified → confirmed | inaccessible | not_found | version_mismatch
review_status: pending → accepted | returned | rejected
preservation_status: unassessed → source_retention_confirmed | alternative_copy_approved | preservation_unknown | retention_hold | disposed_by_policy
```

來源檢查結果可因時間而過期；有效期間由來源型態／風險政策定義。若外部版本變更，來源狀態改為 `version_mismatch`，相關引用上的 `accepted` 審查失效為 `re_review_required`（保留原決議歷史）。權限不足不得以保存 URL 替代確認。來源版次不能取得、來源內容被移動／刪除或保存規則不明時，`preservation_status=preservation_unknown` 並阻擋依賴該紀錄的 Gate；只有文件控制／法遵核定來源平台凍結、合規封存或唯讀副本後，才能轉換到對應保存狀態。處分必須依到期規則及法律保留檢查執行，留下物件清單、依據、執行者、時間與結果。

保存狀態合法轉換：`unassessed → source_retention_confirmed | preservation_unknown`；`preservation_unknown → source_retention_confirmed | alternative_copy_approved | retention_hold`；來源政策變更、版次消失或無法存取會使任何有效保存狀態轉成 `preservation_unknown` 或 `retention_hold`。法遵解除保留後可回到 `source_retention_confirmed`／`alternative_copy_approved`；只有保存期已滿、無法律保留及核准處分任務完成後才轉成終止狀態 `disposed_by_policy`。處分狀態不得復活；若同一來源文件重建或新版本上傳，建立新參照並連到舊參照，不覆寫。

### 範本版

```text
draft → in_review → published → superseded → retired
             ↘ draft
```

退回必須附意見。發布記錄批准者與內容雜湊。新版本發布後，既有流程實例維持舊版，除非完成逐項差異、影響評估、所有者核准及遷移紀錄。

### 品質事項共通狀態

```text
open → triaged → in_progress → awaiting_verification → closed
  └───────────────↘ rejected / accepted_risk（僅適用政策允許類型）
```

驗證失敗由 `awaiting_verification` 回到 `in_progress`，記錄退回原因。任何終結狀態重新開啟須具權限、理由及新期限，並保留原結案記錄。逾期是計算標記，非狀態。

### 變更專屬狀態

```text
proposed → impact_assessment → pending_approval → approved → implementing → verified → closed
                                  ↘ rejected          ↘ returned
```

正式執行不得早於批准。被拒變更不得更新基準。批准後若範圍、影響或實施方案再次變更，回到影響評估並建立新版本／決策。

### 風險專屬狀態

```text
identified → assessed → mitigation_planned → monitoring → closed
                 ↘ accepted_by_authority (限可接受剩餘風險)
```

風險接受記錄接受者、有效期限、剩餘風險、依據與下次複核日。法律／安全等不可接受風險不得進入接受狀態。

### QC 與關卡

- QC 每個項目為 `conforming`、`nonconforming`、`not_applicable` 或尚未提交的空值；空值不可算通過。
- 檢查表提交條件：29 項各一結果；符合有有效證據引用；不符合有理由和有效缺失 ID；不適用有理由及所需核准；風險政策要求的獨立覆核完成。
- Gate 狀態為 `not_ready`、`ready_for_review`、`approved`、`returned`、`rejected`。`ready_for_review` 只表示必填材料已齊，非實質核准。
- Gate 批准條件由版本化模板及租戶政策組合；不可豁免控制有一項未通過則 API 必須回拒，不能由 UI 隱藏欄位繞過。

## 4. 跨模組不變條件

1. 任何核准都必須指向不可變的物件版本與其當時證據版本。
2. 來源參照與來源審查不同；檢查失敗、未檢查、權限不足不能等同符合。
3. 任何狀態轉換均由伺服器驗證角色、狀態前置條件、關聯物件存在與不可豁免控制。
4. 治理決策事件為歷史來源；當前狀態由事件投影計算或以可核對的物化投影加速。命令必須在同一交易中追加事件、驗證租戶序列／物件版本並更新投影；投影可由事件重建。不可透過直接編輯歷史事件修正錯誤，須新增具 `causation_id` 的更正事件。
5. 同一對象版本的重複審查、重試請求需有冪等語意；不可產生重複核准或重複 CAPA。
6. 被撤權使用者的既有稽核事件仍保留穩定主體 ID；個資顯示依租戶保留與隱私政策最小化。
7. 跨專案引用、跨租戶 ID、未授權來源連結一律拒絕；錯誤訊息不得洩漏其他租戶是否存在該物件。
8. 時間排序以伺服器 UTC 時間及事件序號為準；客戶端時間僅可作診斷資料。
9. 稽核事件使用定版序列化以避免不同 JSON 欄位順序造成假警報；每租戶事件鏈定期外部錨定到與一般管理者不同權限邊界的防竄改儲存，單純的 hash chain 不等於 WORM。
10. 事件 schema 變更具向前相容策略；舊事件不可就地遷移覆寫。投影重建前須在隔離資料副本驗證事件數、版本、決議與關聯結果。

## 5. 試點指標口徑

| 指標 | 計算 | 必須保存的維度／限制 |
| --- | --- | --- |
| 版本錯置率 | 使用已過期／不符核准基準版本的文件操作數 ÷ 所有需文件版本確認的操作數 | 記錄錯置定義、期間、專案／文件類型、樣本數；若分母為零顯示 N/A |
| 證據完整度 | 在抽樣範圍內具有效來源參照且適用審查已接受的必要證據項數 ÷ 全部必要證據項數 × 100% | 必要證據清單先核定；`not applicable` 須依政策核准並單獨報表，不可默默排除 |
| 簽核週期 | 每筆首次送審至終結決議的經過時間；報中位數／P90；另計退回次數、重送週期與首次通過率 | `returned` 後再次送審仍屬同一案件；依同一邏輯報整體送審到終結及各輪次。工作日口徑若要使用，需核定營業日曆並與經過時間分開 |
| 流程建立時間 | 從第一次編輯草稿到首次發布的實際操作時間 | 以工作階段／工時紀錄收集；審查等待時間另列；「需求確認至可試跑」屬另一個端到端週期，不混入此指標 |
| 使用者任務／回饋 | 固定任務的成功次數 ÷ 嘗試次數；記任務時間、阻礙；前後採相同問卷量尺 | 題目、尺度、任務腳本、匿名／識別方式、最小回覆數由業務負責人核准；樣本不足顯示 N/A |

所有指標需附基線／試點期間、資料來源、抽樣規則、資料負責人、缺失資料比例、核准目標及驗收者。比較不得把期間／樣本差異隱去。
