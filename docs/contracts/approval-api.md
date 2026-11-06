# 核准工作流 API 契約 v1.0（`CTR-API-001`）

- **狀態：** Active（S4；只讀端點已接線，寫入端點維持誠實 501）
- **Owner：** Engineering
- **最後審閱：** 2026-11-06
- **上位：** [data formats](data-formats.md)、[BDD 場景](../quality/bdd-scenarios-approval.md)、[再設計總計畫 S4](../product/redesign-v2-master-plan.md)

## 1. 共通規則

- 認證：與工作台同一本機 session（cookie）；未登入一律 `401` 結構化錯誤；**無** Bearer/API key（不存在的能力不得宣稱）。
- CSRF：寫入類請求由 `ekpi_csrf` cookie 或 `X-CSRF-Token` header 綁定 session（與 HTML 表單同一 middleware）。
- 錯誤：一律結構化 JSON `{"error":{code,type,message,timestamp,request_id,...}}`。
- 誠實性：未接線端點一律 `501 NOT_IMPLEMENTED`；**絕不**回傳佔位假資料。

## 2. 已接線（唯讀）

### GET /api/v1/workflows

`200`：

```json
{
  "schema": "ekpi-approval-workflows-v1",
  "workflows": [
    {
      "id": "wf-purchase",
      "version": 1,
      "name": "採購簽核",
      "active": false,
      "stages": [
        { "id": "s1", "name": "s1", "sla_seconds": 3600, "urgency": "normal" }
      ]
    }
  ]
}
```

### GET /api/v1/approvals

`200`：

```json
{
  "schema": "ekpi-approval-tasks-v1",
  "tasks": [
    {
      "id": "TSK-000001",
      "workflow_id": "wf-purchase",
      "workflow_version": 1,
      "stage_id": "s1",
      "state": "pending",
      "urgency": "normal",
      "amount_cents": 15000,
      "platform": "momo",
      "classification": "internal",
      "risk_score": 20,
      "created_at": 1000000,
      "due_at": 1036000,
      "escalated_to": null,
      "escalation_count": 0
    }
  ]
}
```

欄位語義同 [BDD 場景規格](../quality/bdd-scenarios-approval.md)之領域語彙；`state` 為 snake_case 終態/進行態列舉。

## 3. 維持 501（未接線，契約屬後續切片）

| 端點 | 理由 |
|---|---|
| POST /api/v1/workflows、…/publish | 寫入經 HTML 工作台已可用；API 寫入契約（冪等鍵、重放防護）待 SDD 後實作 |
| POST /api/v1/approvals、…/decide、…/escalate | 同上 |
| POST /api/v1/events/replay | 重放 API 契約未定義 |
| GET /api/v1/channels、/health | 通道註冊表尚未存在 |
| GET /api/v1/users | 使用者清單涉及身分治理邊界（DEC-12 未決），不洩露帳號存在性 |
| GET /api/v1/settings | 設定暴露面未經安全審查 |
| GET /api/v1/search | 檢索 API 參數/權限契約未定義（S19 權限語義需先定） |

## 4. 變更規則

- 新增欄位＝向後相容（同步更新本檔與測試）；移除/改名/語義變更＝破壞性（需 ADR＋版本升級 `*-v2`）。
- 任何端點從 501 轉為 200 前，必須先有契約、測試與誠實資料來源；禁止先上線後補文檔。
