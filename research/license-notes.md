# 授權合規登記表 —— compliance-critical 鏈路依賴第一手核驗

- **核驗日（抓取日）**：2026-10-25（所有 URL 皆於當日抓取）
- **核驗範圍**：5 個 Rust 依賴：serde、serde_json、sha2、regex（候選）、jsonschema（候選）
- **紀錄原則**：授權一律以官方來源為準（crates.io 官方資料、GitHub repo LICENSE 原文）；抓取失敗如實標註、不推斷；雙授權如實記錄兩個選項。
- **總結論**：5/5 核驗成功（verified = yes）；全數無 GPL／AGPL／LGPL 類授權，無 copyleft 阻塞項。

---

## 一、核驗記錄總表

| crate | 鎖定版本（或候選） | license（官方表述原文） | 官方來源 URL | verified_date | verification_method | 備註 |
|---|---|---|---|---|---|---|
| serde | 1.0.229（鎖定版；2026-07-18 發布，publisher=dtolnay，非 yanked） | `MIT OR Apache-2.0`<br>（雙授權：MIT 或 Apache-2.0，二擇一） | crates.io API v1：https://crates.io/api/v1/crates/serde/1.0.229<br>crate 頁：https://crates.io/crates/serde/1.0.229<br>repo LICENSE 原文：https://github.com/serde-rs/serde/blob/master/LICENSE-MIT ；https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE | 2026-10-25 | crates.io 官方 API v1 版本级 license 欄＋ GitHub repo LICENSE-MIT／LICENSE-APACHE 全文抓取核對（註：crates.io HTML 頁面抓取無內容，改以 API v1 取得，同屬 crates.io 官方來源；詳見第二節） | MIT 條文與 Apache-2.0 條文均已取得全文核對。雙授權需二選一：選 MIT 須保留原著作權與許可聲明即可；選 Apache-2.0 為明示專利授權＋NOTICE 等義務。無 copyleft，不阻塞合規鏈路，放行。 |
| serde_json | 1.0.154（鎖定版；2026-10-11 發布，publisher=dtolnay，非 yanked） | `MIT OR Apache-2.0`<br>（雙授權：MIT 或 Apache-2.0，二擇一） | crates.io API v1：https://crates.io/api/v1/crates/serde_json/1.0.154<br>crate 頁：https://crates.io/crates/serde_json/1.0.154<br>repo LICENSE 原文：https://github.com/serde-rs/json/blob/master/LICENSE-MIT ；https://github.com/serde-rs/json/blob/master/LICENSE-APACHE | 2026-10-25 | 同 serde：crates.io 官方 API v1 版本级 license 欄＋ repo（serde-rs/json）LICENSE-MIT／LICENSE-APACHE 全文抓取核對 | 與 serde 同一生態（作者 dtolnay、governance 相同）。兩個授權選項均已全文核對。無 copyleft，不阻塞合規鏈路，放行。 |
| sha2 | 0.10.9（鎖定版；2025-04-30 發布，publisher=newpavlov，非 yanked） | `MIT OR Apache-2.0`<br>（雙授權：MIT 或 Apache-2.0，二擇一） | crates.io API v1：https://crates.io/api/v1/crates/sha2/0.10.9<br>crate 頁：https://crates.io/crates/sha2/0.10.9<br>repo LICENSE 原文：https://github.com/RustCrypto/hashes/blob/master/sha2/LICENSE-MIT ；https://github.com/RustCrypto/hashes/blob/master/sha2/LICENSE-APACHE<br>交叉佐證：https://github.com/RustCrypto/hashes/blob/master/sha2/Cargo.toml（`license = "MIT OR Apache-2.0"`） | 2026-10-25 | crates.io 官方 API v1 版本级 license 欄＋ repo sha2/ 子目錄 LICENSE-MIT／LICENSE-APACHE 全文＋ master Cargo.toml license 欄三方交叉比對 | 授權檔放於 sha2/ 子目錄（repo 根目錄無 LICENSE、GitHub license 偵測=null，已如實記錄——首次對 repo 根目錄抓 LICENSE-MIT／LICENSE-APACHE 得 404，屬路徑定位失敗而非授權缺失）。MIT 條文含多行著作權行（RustCrypto Project Developers、Artyom Pavlov、Mozilla Foundation、Graydon Hoare），均為標準 MIT 授權表述。密碼學組件，授權面無障礙；master 已在開發 0.11.1（2026-10-23 仍有 push），授權宣告不變。放行。 |
| regex | 候選最新版：1.13.1（2026-07-15 發布；max_version=最新版皆 1.13.1，非 yanked） | `MIT OR Apache-2.0`<br>（雙授權：MIT 或 Apache-2.0，二擇一） | crates.io API v1：https://crates.io/api/v1/crates/regex<br>版本頁：https://crates.io/crates/regex/1.13.1<br>repo LICENSE 原文：https://github.com/rust-lang/regex/blob/master/LICENSE-MIT ；https://github.com/rust-lang/regex/blob/master/LICENSE-APACHE | 2026-10-25 | crates.io 官方 API v1（crate 總覽含各版本 license）＋ repo LICENSE-MIT（「Copyright (c) 2014 The Rust Project Developers」起之 MIT 條文）與 LICENSE-APACHE（Apache-2.0 全文）抓取核對 | MIT 條文與 Apache-2.0 條文均已全文核對。附屬授權檔 `LICENSE-UNICODE` 於 master 抓取返回 404（2026-10-25）——如實記錄、未推斷其存否；正式採用前應比對正式發行包內的授權檔清單後再定案。無 copyleft，作為 JSON Schema pattern 校驗之候選依賴無授權阻塞。 |
| jsonschema | 候選最新版：0.60.1（2026-10-23 發布，非 yanked） | `MIT`<br>（crates.io 版本資料原文；repo LICENSE 原文首行為「MIT License」，著作權行「Copyright (c) 2020-2026 Dmitry Dygalo」） | crates.io API v1：https://crates.io/api/v1/crates/jsonschema<br>版本頁：https://crates.io/crates/jsonschema/0.60.1<br>repo LICENSE 原文：https://github.com/Stranger6667/jsonschema/blob/master/LICENSE<br>repo 維護狀態（GitHub API）：https://api.github.com/repos/Stranger6667/jsonschema | 2026-10-25 | crates.io 官方 API v1 版本级 license 欄＋ repo LICENSE 全文＋ GitHub repo API（license 偵測=MIT、archived=false、最後 push=2026-10-24、stars=829、forks=135、open issues=17） | **建議採用：是**——MIT 單授權無 copyleft 風險，且維護極活躍（2026-10 內即有 6 個 release 0.58.4→0.60.1、repo 前一日仍有 push）。範圍聲明：以上核驗僅及 crate 本體；其遞移依賴（referencing 等）授權未在本次範圍，採用前須以同標準另行核驗。文件站：https://jsonschema.dygalo.dev/ 。 |

---

## 二、抓取失敗／例外記錄（如實、不推斷）

| # | 項目 | URL | 結果（2026-10-25） | 處置 |
|---|---|---|---|---|
| 1 | crates.io HTML 頁面 5 份（serde/1.0.229、serde_json/1.0.154、sha2/0.10.9、regex、jsonschema） | 各 https://crates.io/crates/… | 經抓取轉換後**無可讀內容**（頁面為動態渲染，授權欄位未出現在伺服器端 HTML） | 改採 crates.io 官方 **API v1**（同屬 crates.io 官方來源，且直接提供版本級 license 欄位）；已在表中 method 欄如實註明 |
| 2 | RustCrypto/hashes repo 根目錄 LICENSE-MIT／LICENSE-APACHE | https://raw.githubusercontent.com/RustCrypto/hashes/master/LICENSE-MIT 、…/LICENSE-APACHE | **404**（授權檔並非位於 repo 根目錄；GitHub API 對根目錄偵測 license=null） | 經根目錄清單定位後改抓 `sha2/` 子目錄之 LICENSE-MIT／LICENSE-APACHE → 成功驗證；sha2 行 verified = yes |
| 3 | rust-lang/regex `LICENSE-UNICODE` | https://raw.githubusercontent.com/rust-lang/regex/master/LICENSE-UNICODE | **404**（2026-10-25 於 master 分支） | 僅如實記錄 404，不據此推斷該檔於倉庫或發行包中存在與否；regex 核心授權已由 LICENSE-MIT／LICENSE-APACHE＋crates.io license 欄三證確認 |

---

## 三、jsonschema crate 維護狀態評估（license 與活躍度）

- **license**：MIT（單授權），著作權人 Dmitry Dygalo（2020–2026），GitHub license 自動偵測與 LICENSE 原文、crates.io license 欄三方一致。
- **維護活躍度證據**（GitHub API，抓取日 2026-10-25）：
  - repo 最後 push：**2026-10-24**（核驗前一日）
  - 2026 年 10 月內連續發布 **6 個版本**（0.58.4、0.58.5、0.58.6、0.59.0、0.60.0、0.60.1；最新 0.60.1 於 2026-10-23）
  - 未歸檔（archived=false）、open issues=17：規模健康；829 stars／135 forks
- **採用建議**：

> **建議採用：是** —— license 為 MIT（無 copyleft、無 GPL 類阻塞），且維護跡象極活躍（最後 push 2026-10-24、10 月內 6 次發布）；前提條件：對其遞移依賴（referencing、fluent-uri 等）完成與本表同標準的授權核驗後方可進入合規鏈路。

---

## 四、合規風險清單

1. **GPL 類授權禁項**：本批 5 個 crate 均未發現 GPL／AGPL／LGPL 授權——目前合規鏈路無 copyleft 阻塞項。任何後續新增依賴若屬 GPL 類授權，**不得引入**（會阻塞合規鏈路），本表制度上持續把關。
2. **雙授權（MIT OR Apache-2.0）落實方式**：`OR` 代表使用者二擇一遵循。選 MIT：保留原版授權與著作權聲明即可；選 Apache-2.0：需留意味明示專利授權、修改聲明與 NOTICE 要求。登記時**兩個選項均原樣記錄**（本表已執行），後續由供應鏈合規政策擇一統一（建議與專案內其他 MIT OR Apache-2.0 依賴採同一選擇以簡化 NOTICE）。
3. **版本鎖定核驗方式**：serde 1.0.229、serde_json 1.0.154、sha2 0.10.9 皆為 Cargo.lock 鎖定版本；本次核驗直接針對**該鎖定版本**的 crates.io 版本級資料（非僅 crate 最新頁面），避免「最新版授權≠鎖定版授權」的落差風險。
4. **維護基本面**：serde 與 serde_json 同屬 serde-rs 組織（dtolnay 系）；sha2 屬 RustCrypto 組織（2026-10-23 仍有 push）；regex 屬 rust-lang 官方倉庫——四者維護基本面穩定；jsonschema 見第三節。
5. **未竟事項**（不推斷、待補驗）：
   - regex：正式採用前比對其發行包內授權檔清單（含 LICENSE-UNICODE 存否）。
   - jsonschema：遞移依賴授權核驗（見第三節前提條件）。

---

## 五、證據清單（全部於 2026-10-25 抓取）

| 證據 | URL | 取得內容 |
|---|---|---|
| crates.io API v1 — serde 1.0.229 | https://crates.io/api/v1/crates/serde/1.0.229 | 版本级 JSON：license=`MIT OR Apache-2.0`、發布日 2026-07-18、publisher=dtolnay、repo=serde-rs/serde、非 yanked |
| crates.io API v1 — serde_json 1.0.154 | https://crates.io/api/v1/crates/serde_json/1.0.154 | license=`MIT OR Apache-2.0`、發布日 2026-10-11、publisher=dtolnay、repo=serde-rs/json、非 yanked |
| crates.io API v1 — sha2 0.10.9 | https://crates.io/api/v1/crates/sha2/0.10.9 | license=`MIT OR Apache-2.0`、發布日 2025-04-30、publisher=newpavlov、repo=RustCrypto/hashes、非 yanked |
| crates.io API v1 — regex（總覽） | https://crates.io/api/v1/crates/regex | 最新版 1.13.1（2026-07-15）、license=`MIT OR Apache-2.0`、repo=rust-lang/regex |
| crates.io API v1 — jsonschema（總覽） | https://crates.io/api/v1/crates/jsonschema | 最新版 0.60.1（2026-10-23）、license=`MIT`、repo=Stranger6667/jsonschema |
| serde LICENSE-MIT / LICENSE-APACHE | https://github.com/serde-rs/serde/blob/master/LICENSE-MIT （原文經 raw 同路徑取得） | MIT 條文全文 ／ Apache-2.0 條文全文 |
| serde_json LICENSE-MIT / LICENSE-APACHE | https://github.com/serde-rs/json/blob/master/LICENSE-MIT （原文經 raw 同路徑取得） | MIT 條文全文 ／ Apache-2.0 條文全文 |
| sha2 LICENSE-MIT / LICENSE-APACHE（sha2/ 子目錄） | https://github.com/RustCrypto/hashes/blob/master/sha2/LICENSE-MIT 、…/sha2/LICENSE-APACHE | MIT 條文全文（多行著作權行）／ Apache-2.0 條文全文 |
| sha2 master Cargo.toml（交叉佐證） | https://github.com/RustCrypto/hashes/blob/master/sha2/Cargo.toml | `license = "MIT OR Apache-2.0"`（master 為 0.11.1 開發版） |
| regex LICENSE-MIT / LICENSE-APACHE | https://github.com/rust-lang/regex/blob/master/LICENSE-MIT （原文經 raw 同路徑取得） | MIT 條文全文（Copyright (c) 2014 The Rust Project Developers）／ Apache-2.0 條文全文 |
| jsonschema LICENSE | https://github.com/Stranger6667/jsonschema/blob/master/LICENSE | MIT 條文全文（Copyright (c) 2020-2026 Dmitry Dygalo）；GitHub API `/license` 端點亦回傳同一檔（spdx_id=MIT） |
| jsonschema repo 維護狀態 | https://api.github.com/repos/Stranger6667/jsonschema | archived=false、pushed_at=2026-10-24、stars=829、forks=135、open_issues=17 |

（註：表中「原文經 raw 同路徑取得」指 fetch 時以 `raw.githubusercontent.com/<owner>/<repo>/<ref>/<LICENSE檔>` 直取原檔全文，與前述 GitHub blob 頁為同一檔案之官方內容。）
