# 端到端需求追溯基線 v1.0

**狀態：** 由使用者提供的治理藍圖推導而成，尚未通過實際部門訪談／業務確認。這份基線用來防止「需求、設計、程式、測試各自一份文件」而失去關聯；業務確認後要用正式需求系統維護同等欄位與關聯。

## 1. 追溯規則與語意

每個 ID 穩定且不得重複使用。以下標籤 `DRAFT-USER-BRIEF` 表示已從使用者明確提供的目標推導，並非真實使用者訪談證據；`OPEN-DEC` 表示規則／閾值待指定業務角色核定；`PROTOTYPE-PARTIAL` 表示目前本機原型只有部分支援；`NOT-IMPLEMENTED` 表示目前原型沒有此能力；`NOT-RUN` 表示此正式產品驗收案例尚未執行。

關聯的允許語意：

- `derived_from`：需求從哪一個組織目標／使用者 need / 政策來源推導。
- `decomposed_into`：高層需求分解到系統需求／資料／狀態／介面。
- `realized_by`：需求分配給哪個模組、畫面、API、流程控制或操作程序。
- `verified_by`：哪個測試／分析／檢查可證明這條需求被正確實作。
- `validated_by`：真實角色／業務負責人用哪些情境確認需求解決實際問題。
- `impacted_by`：改版或來源政策變更會影響哪些在途實例、文件、角色與驗收案例。
- `evidenced_by`：PASS 證據的位置、版本、產生時間、建置識別及審查者。

只有 `verified_by` 測試通過是「符合規格」證據；只有使用者／業務 `validated_by` 是「適合預期用途」證據。兩者不可互相取代。

## 2. 藍圖的待驗證業務需要

| Need ID | 推導需要 | 可觀察問題／結果 | 推導來源 | 狀態與確認角色 |
| --- | --- | --- | --- | --- |
| SN-01 | 文件版本與證據能準確回連決策 | 抽樣案件中找得到正確權威來源和當時使用的版次；過期／無權限不被誤判為有效 | 使用者提供的跨工具治理計畫，優先試點「文件版本與證據追溯」 | DRAFT-USER-BRIEF；需文件控制者、實際工作者、QA 確認 |
| SN-02 | 部門能配置自身流程，並完成跨部門交接 | 範本能表達階段、關卡、角色、交付物、期限、完成條件和例外；責任變化可查 | 使用者藍圖：部門自訂與跨部門流程；手動配置要易用 | DRAFT-USER-BRIEF；需流程擁有者／PMO 確認 |
| SN-03 | 專案品質控制從立案一路閉環至效益追蹤 | 12 階段／5 關卡、QC、風險、變更、缺失、CAPA、驗收、封存和改善相互追溯 | 使用者藍圖生命週期與品質治理要求 | DRAFT-USER-BRIEF；需贊助人／QA／PMO 確認 |
| SN-04 | 先用核准既有工具證明問題／效果，再決定是否自建 | M365、Google Workspace 與混合工具有實際租戶能力／成本／權限證據；平台 gap 有無法以設定合理解決的需求 ID | 使用者選擇的既有工具試點與 MVP go/no-go 方案 | DRAFT-USER-BRIEF；需業務贊助人／IT／資安確認 |
| SN-05 | 工作人員能在不同裝置上清楚處理待辦、證據、進度與狀態 | 任務完成率、時間、錯誤、協助次數及無障礙結果可觀察改善 | 使用者要求專業、一致、可存取、響應式、直覺 UX | DRAFT-USER-BRIEF；需 UX／代表性使用者確認 |
| SN-06 | AI 提升草稿效率但不取代負責人決策 | 生成內容可指出來源、未知及矛盾；人可逐項修改／拒絕；AI 無發布、批准、豁免、Gate 權限 | 使用者的 AI 草稿與人工檢查核准要求 | DRAFT-USER-BRIEF；需流程擁有者／AI 負責人／資安確認 |

## 3. 需求分解與實作追溯表

### 功能需求

| Requirement ID | 上游 Need | 正式要求位置 | 實作／設計配置 | 驗證案例 | 業務確認／依賴 | 原型狀態 |
| --- | --- | --- | --- | --- | --- | --- |
| FR-01 | SN-03 | 05 §4 FR-01；06 Project | Project service、租戶識別碼、立案／基準版本 | AT-01, AT-20 | DEC-01…04；具名贊助人及組織結構 | PROTOTYPE-PARTIAL：本機專案欄位；無企業身分、完整章程／租戶權限 |
| FR-02 | SN-01/04 | 05 §4 FR-02；06 EvidenceReference/ExternalDocumentVersion | Evidence service、版本定位、來源／保存狀態 | AT-06…08, AT-31 | DEC-05/10/13；來源版次與記錄保存控制 | PROTOTYPE-PARTIAL：人工 URL／版本參照；無來源驗證、留存或連接器 |
| FR-03 | SN-01/03 | 05 §4 FR-03；06 Review | Review service、版本綁定、退回／重審 | AT-05, AT-08/09 | DEC-02/07/12；審查分工與核准權 | PROTOTYPE-PARTIAL：手動 review；actor 非目錄身份、無範圍授權 |
| FR-04 | SN-02/06 | 05 §4 FR-04；06 WorkflowTemplateVersion | Workflow service、結構化 stage／gate schema、發布版 | AT-12…14, AT-23 | DEC-01/02/06/15/18；角色、模板所有權、規則 | PROTOTYPE-PARTIAL：手動純文字階段、草稿／簡易發布；無結構條件、正式退回、AI |
| FR-05 | SN-03 | 02 生命周期；05 §2 S01–S12/G1–G5；06 WorkflowRunStage/GateDecision | Lifecycle service、版本化條件、transition policy | AT-02…05, AT-14 | DEC-03/06/07/18；Gate 決策人與不可豁免要求 | PROTOTYPE-PARTIAL：手動記決議；無完整 stage 執行或強制阻擋 |
| FR-06 | SN-03 | 05 §4 FR-06；06 Case state machines | Quality/case service、類型欄位與獨立複驗 | AT-15…18 | DEC-03/08/18；嚴重度、期限與根因門檻 | PROTOTYPE-PARTIAL：共用簡化事項欄位；無型別狀態機／CAPA 驗證 |
| FR-07 | SN-01/04/05 | 05 §4 FR-07；06 §5 指標 | Metrics service／可追溯報表和資料字典 | AT-24, business validation | DEC-09；業務核定基線、目標、樣本和指標接受者 | PROTOTYPE-PARTIAL：本機計數；無真實試點基線／成效 |
| FR-08 | SN-01/04 | 05 §4 FR-08；07 §3 Connector contract | 具權限的 connector workers、outbox、source registry | AT-06/07/21/31 | DEC-10/11/12/13；租戶 SKU、授權及優先級 | NOT-IMPLEMENTED：目前無 Microsoft／Google/API 連線 |
| FR-09 | SN-02/06 | 05 §4 FR-09；07 §5 AI；09 §5 | AI gateway、草稿 diff／來源標記／人工審查 | AT-23 | DEC-15；供應者、資料範圍、模型與量測門檻 | NOT-IMPLEMENTED：無 AI 服務 |
| FR-10 | SN-01/03/06 | 05 §4 FR-10；06 AuditEvent；07 §1.1/§6 | Event/audit service、外部封存／完整性監控 | AT-05, AT-19–22, AT-27/30/31 | DEC-12/13/14；稽核保存、權限和外部封存 | PROTOTYPE-PARTIAL：本機追加式 hash chain；無企業身份/WORM/備份服務 |
| FR-11 | SN-03 | 02 QC-01…QC-29；05 §4 FR-11；06 QualityInspection | QC service、條件式必填、缺失與證據連結 | AT-10/11/15/17 | DEC-03/07/08；N/A 核准及高風險獨立覆核 | PROTOTYPE-PARTIAL：29 項表單及保存；無角色型獨立審核 |

### 非功能需求

| Requirement ID | 上游 Need | 正式要求位置 | 實作／設計配置 | 驗證案例 | 業務確認／依賴 |
| --- | --- | --- | --- | --- | --- |
| NFR-01 UX | SN-05 | 05 §3/§6；09 §4 | Task-based navigation、clear states/errors | AT-25/26；pilot tasks | DEC-01/02/17 |
| NFR-02 Accessibility | SN-05 | 05 §6；09 §1/§4 | WCAG 2.2 AA semantics/focus/responsive | AT-25/26 | DEC-17 + local law review |
| NFR-03 Identity/Authorization | SN-01/03/06 | 07 §2；05 §6 | Enterprise IdP, RBAC + project/tenant scope | AT-11/19/20 | DEC-02/12/13 |
| NFR-04 Records/Integrity | SN-01/03 | 06 Evidence lifecycle; 07 §6; 05 §6 | Version pinning, preservation state, append-only audit | AT-07/08/22/27/31 | DEC-05/13/14 |
| NFR-05 Privacy/AI | SN-06 | 07 §5; 05 §4 FR-09/§6 | Approved data fields, provider/model log, AI no approval capability | AT-23/30 | DEC-13/15/18 |
| NFR-06 Interoperability | SN-01/04 | 07 §3/§4; 05 §6 | Canonical references, contract adapters, export | AT-06/07/21/31 | DEC-10/11 |
| NFR-07 Operability | SN-04 | 07 §1.1/§6; 05 §6 | Reproducible deployment, monitoring, restore/rollback | AT-21/27/30 | DEC-14/16 |
| NFR-08 Performance/Capacity | SN-05 | 05 §6; 09 §4 | Scalable API/search/report design | AT-29 | DEC-19 |
| NFR-09 Reliability/Recovery | SN-01/03 | 07 §1.1/§6; 05 §6 | Atomic business/audit writes; idempotent outbox recovery | AT-21/27/29 | DEC-14/16 |
| NFR-10 Security Engineering | SN-01/03/06 | 07 §2/§4/§5; 09 §1/§5 | Threat model, ASVS/SSDF, SBOM, secure review | AT-19–23/30 | DEC-12/13/15/20 |
| NFR-11 Maintainability/Change | SN-02/04 | 05 §6; 06 §4; 09 §3/§6 | Immutable workflow versions, migration and rollback | AT-13/14/27/30 | DEC-06/16/19 |

## 4. 可推導的端到端證據樣例

**文件來源版本變更**

```text
SN-01 (要避免錯用文件版本)
  → FR-02 (保存外部文件版次，版本改變建立新參照)
  → NFR-04 (不得靜默破壞歷史決策／保存責任)
  → EvidenceReference.version_locator + source_status + preservation_status
  → Evidence service：新舊版關聯，舊版審查保留但失效於新版本
  → AT-08 (v1 已核准，來源變 v2) + AT-31 (移動／刪除／撤權／保留期)
  → PASS evidence: build ID + connector mock/tenant trace + decision history + reviewer
  → validated_by: 文件控制者與 QA 以真實案件核對抽樣結果
```

**AI 流程草稿人工決定**

```text
SN-06 (AI 提升起草效率且不取代負責人)
  → FR-09 (只生成草稿、標來源／未知／矛盾、逐階段真人覆核)
  → NFR-05 (資料範圍、模型來源、無發布權)
  → AI gateway + WorkflowTemplateVersion draft + ReviewerDecision
  → AT-23 (不足／矛盾／注入／受限資料／接受全部繞過／保留集品質)
  → PASS evidence: prompt/model/input/output versions + review diffs + eval dataset report
  → validated_by: 流程擁有者證明草稿需時下降且漏掉強制步驟不增加
```

測試名稱出現於矩陣不等同 `PASS`。`evidenced_by` 在每次實際執行時須補 commit/build、測試資料版本、操作者、時間、原始結果、缺陷、修正和重測；業務驗證需另外存受訪角色、試點樣本、限制與簽收。

## 5. 需求變更和發版閉合檢查

每次受控變更輸出差異清單：來源 Need、受影響 FR/NFR、頁面/API/資料表/狀態／事件、在途流程、連接器／租戶設定、AI 評估集、測試、操作手冊、訓練、保存／法規影響。只有逐項決策後才能建立發版候選。

發版追溯報表至少列：

- 必要需求總數、來源可追溯數、設計分配數、測試關聯數、已執行數、PASS／FAIL／BLOCKED／NOT RUN 數。
- 每一項 FAIL 有缺陷與修正 build；每一項 BLOCKED 有阻擋責任人／日期；任何 `NOT RUN` 必要需求使退出 gate 不通過。
- 適用標準版號、控制適用性／例外決議；試點資料／指標字典版本與核准人。
- 對外宣稱限於已驗證環境、指定用途、樣本與日期；不使用「全市場最好／超越所有軟體」等無法證據支持的絕對說法。
