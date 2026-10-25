# LegalRuleML v1.0（OASIS Standard, 2021-08-30）官方構件筆記

> 用途：為「確定性合規規則引擎 → LegalRuleML 匯出器」提供官方第一手來源的 XML 構件與語義骨架。
> 紀律：只引用 2026-10-25 實際抓取到的官方頁面內容；抓不到的一律標「未能核实」，不編造構件名。
>
> - 撰寫日期：2026-10-25
> - 引用代號約定：**[S1] =** https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.html （規格本文 Authoritative HTML）
> - 引句為規格原文英文短摘錄（節錄號以規格章節為準）；中文為筆記說明。
> - 本地抓取全文備份（規格 markdown 版，約 4700+ 行）：`C:\Users\stephanie.huang\.local\share\opencode\tool-output\tool_16f5b5b5f001YfPK2x658PuMtO`

---

## 一、官方來源 URL 清單（含核实紀錄）

| # | URL | 內容 | verified_date | verification_method | 結果 |
|---|-----|------|---------------|---------------------|------|
| S1 | https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.html | 規格本文（OASIS Standard, 30 August 2021）Authoritative HTML | 2026-10-25 | official_page_fetch | ✅ 抓取成功，含完整目錄與全文（1 Introduction ~ Annex G） |
| S2 | https://www.oasis-open.org/committees/legalruleml/ | OASIS LegalRuleML TC 官方頁 | 2026-10-25 | official_page_fetch | ✅ 成功；頁面公告 "LegalRuleML Core Specification v1.0 is now an OASIS Standard"，並列出全部 artifact 連結 |
| U1 | https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/relaxng/ | RelaxNG schema 目錄 | 2026-10-25 | official_page_fetch（目錄索引） | ✅ 列出：`lrml-basic.rnc`、`lrml-compact.rnc`、`lrml-normal.rnc`、`rdfxml.rnc`、`modules/`、`modules-init/`、`rdfxml.rnc`、`ruleml/`、`suites/`、`xml/`、`xsi/`（皆標註日期 30-Aug-2021） |
| U2 | https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/relaxng/lrml-compact.rnc | Compact 序列化 RNC 驅動檔 | 2026-10-25 | official_page_fetch（HTTP GET，status 200，content-type `application/relax-ng-compact-syntax`, 716 bytes） | ✅ 已取得全文；內容為 driver，例如 `include "modules/document_module.rnc" inherit = lrml`、`include "suites/compact.rnc"`、`namespace lrml = "http://docs.oasis-open.org/legalruleml/ns/v1.0/"` |
| U3 | https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/xsd-schema/ | XSD schema 目錄 | 2026-10-25 | official_page_fetch（目錄索引） | ✅ 列出：`basic/`、`compact/`、`datatypes/`、`normal/` |
| U4 | https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/xsd-schema/compact/lrml-compact.xsd | Compact XSD 規範 schema | 2026-10-25 | official_page_fetch（HTTP GET，status 200，150,555 bytes） | ✅ 檔頭原文：`LegalRuleML Core Specification Version 1.0` / `OASIS Standard` / `30 August 2021`；根：`<xs:schema xmlns:lrml="http://docs.oasis-open.org/legalruleml/ns/v1.0/" xmlns:ruleml="http://ruleml.org/spec"` |

### 由規格封面/TC 頁登記、但本次「未個別抓取」之連結（未能核实其頁面內容）

| URL | 依據 | 備註 |
|-----|------|------|
| https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.docx / .pdf / .zip | [S1] 封面 "This document is one component of a Work Product" 段落與 Distribution ZIP 說明 | 未抓取 |
| https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/xslt/ 、 …/generation/ 、 …/rdfs/ 、 …/diagrams/ 、 …/examples/ | [S1] 封面 Additional artifacts 列表；[S2] TC 頁 | 未抓取 |
| …/os/xsd-schema/basic/ 、 …/normal/（即 `lrml-basic.xsd`、`lrml-normal.xsd`）與 `lrml-basic.rnc`、`lrml-normal.rnc` | [S1] §3.2、§7 Conformance | 檔案存在性由 [U1]/[U3] 目錄可推知，但基本/常規 schema 內容未逐字核對 → 視同未能核实 |
| https://www.oasis-open.org/2021/09/08/legalruleml-core-specification-v1-0-oasis-standard-published/ | [S2] 公告連結 | 未抓取 |
| https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/legalruleml-core-spec-v1.0.html（Latest stage） | [S1] 封面 | 未抓取 |
| **https://legalruleml.oasis-open.org/**（及 …/specs/） | — | ❌ **未能核实**：2026-10-25 兩次 HTTP/HTTPS 請求均失敗（DNS 解析失敗）。目前的規範性內容全部以 docs.oasis-open.org 為準。 |

### 名稱空間（Namespace）

- LegalRuleML 主名稱空間：`http://docs.oasis-open.org/legalruleml/ns/v1.0/`
  - 原文 [S1] §3.3："The LegalRuleML namespace is http://docs.oasis-open.org/legalruleml/ns/v1.0/."
- LegalRuleML 元模型名稱空間：`http://docs.oasis-open.org/legalruleml/ns/mm/v1.0/`
  - 原文 [S1] §3.3："The LegalRuleML metamodel namespace is http://docs.oasis-open.org/legalruleml/ns/mm/v1.0/"
- RuleML 名稱空間：`http://ruleml.org/spec`（[S1] §3.3）
- 本文之縮寫約定（[S1] §1.4）：`lrml` = LegalRuleML ns、`lmrlmm`（原文誤植風格，另一處作 `lrmlmm`）= metamodel ns、`ruleml` = RuleML ns。
- 例證：[U2] RNC 檔全文首行 `namespace lrml = "http://docs.oasis-open.org/legalruleml/ns/v1.0/"`；[U4] XSD 根元素宣告同前綴。→ 名稱空間 Internet 不可「抓取頁面」，但 schema 檔案已確認該 URI 被使用。

---

## 二、關鍵構件清單（原文英文 + 繁中說明）

### 2.1 文件根元素與容器

- **`<lrml:LegalRuleML>`（文件根元素）**
  - [S1] §5.5.1.1 原文："In LegalRuleML, the element `<lrml:LegalRuleML>` is the only Document Node element, and it has type lrmlmm:LegalRuleMLDocument."
  - [S1] §5.19.7 原文："The LegalRuleML RELAX NG schemas enforce the requirement that the root element is lrml:LegalRuleML. This requirement is not enforced in the XSD schemas…"
  - [S1] §7 原文："A XML file is compliant with the LegalRuleML specifications as a LegalRuleML-XML file if it is a well formed XML file whose root is `<lrml:LegalRuleML>` and it is valid against at least one XML LegalRuleML Schema"
  - ❌ 任務中提到的 "LegalSchema"：規格全文與 XSD 中 **不存在**（全文檢索 0 次）→ 以官方為準，根元素只有 `lrml:LegalRuleML`。
  - 根元素可選屬性（[S1] §5.5.1.1）：
    - `@xml:base`、`@hasCreationDate`、`@xsi:schemaLocation`，另加 common optional Node attributes（`@key`、`@keyref`、`@type`）。
    - `@hasCreationDate` 語義："has semantics related to Dublin Core's http://dublincore.org/documents/dcmi-terms/#terms-created, except that …takes a local identifier reference to a `<ruleml:Time>` entity."
- **Collector（容器）節點**：[S1] §5.5.1.1 原文："Collector Nodes are Node elements that can contain a collection - these are `<lrml:LegalRuleML>`, the root element, `<lrml:(Legal)Sources>`, and `<lrml:Statements>`."
- **頂層元素之規定性順序**（對匯出器極重要）——[S1] §5.16 原文節錄：

  ```
  Comment* & (Prefix*, (LegalReferences | LegalSources | References | Sources
    | Times | TemporalCharacteristics | Agents | Figures | Roles
    | Authorities | Jurisdictions)*, Associations*,
    (Alternatives | Context | Statements)*)
  ```
  - 四類順序："The first category of elements is Prefix. … The second category of elements is metadata elements, (LegalReferences, LegalSources, References, Sources, Times, TemporalCharacteristics, Agents, Figures, Roles, Authorities, and Jurisdictions). … The third category of elements is associations (Associations). … The fourth category of elements is statement-related (Alternatives, Context, Statements)."（[S1] §5.16）

### 2.2 Prefix / References / Sources（來源識別與 CURIE）

- **`<lrml:Prefix>`（前綴宣告）**
  - [S1] §3.4 原文："Prefix +: a prefix declaration in a LegalRuleML document."
  - 專用屬性（[S1] §5.5.1.1）："@pre, on `<lrml:Prefix>`" 與 "@refID, on `<lrml:Prefix>`, `<lrml:Reference>` or `<lrml:LegalReference>`"
  - [S1] §4.2.1(例 2, compact) 原文範例：`<lrml:Prefix pre="defvo" refID="http://example.org/defeasible/vocab#"/>`
  - 語義（[S1] §5.20 原文）："A number of RuleML and LegalRuleML attributes have values which should be treated as CURIEs. …a prefix mapping, which is defined by the `<lrml:Prefix>` element."；"CURIE prefixes are defined (only) by `<lrml:Prefix>` elements"（§5.12/§5.11 相關文字, [S1] §5.12 原文節錄）。
  - ❌ 任務中提到的 "PrefixDeclaration"：規格全文 **不存在** 此元素名（檢索 0 次）→ 官方構件名為 `lrml:Prefix`。
- **`<lrml:References>` / `<lrml:Reference>`（非 IRI 型識別碼）**
  - [S1] §3.4 原文："Reference(s) +: a pair consisting of an internal ID and an enriched non-IRI identifier, where the non-IRI is paired with some additional information that is sufficient to disambiguate the non-IRI to a unique LegalSource."
  - [S1] §4.3.1 原文："`<lrml:References>` is the element dedicated to record non-IRI based identifier sources, and the attribute refIDSystemName is able to annotate the naming convention used."
  - 範例（[S1] §4.3.1 例 15, compact 原文節錄）：

    ```xml
    <lrml:LegalReferences … refType="http://example.legalruleml.org/lrml#LegalSource">
      <lrml:LegalReference refersTo="ref1" refID="/au/2012-05-30/C628:2012/eng@/main#sec2.2"
                           refIDSystemName="AkomaNtoso3.0-2016-03"/>
      <lrml:LegalReference refersTo="ref6" refID="ECLI:EU:C:2015:650"
                           refIDSystemName="European Case Law Identifier"
                           refIDSystemSource="OJ:C:2011:127:0001:0007:EN:PDF"/>
    </lrml:LegalReferences>
    ```
  - 屬性：
    - `@refersTo`：[S1] §3.7 "@refersTo: the internal ID of the Reference."；§5.5.1.1 "@refersTo (on `<lrml:Reference>` and `<lrml:LegalReference>`)"
    - `@refID`："the external ID of the Reference."（§3.7）
    - `@refIDSystemName`、`@refIDSystemSource`、`@refType`（§3.7；§5.5.1.1 註明可加在 Reference(s)/LegalReference(s)）
  - 限制：[S1] §5.5.1.1 原文："with the exception of `<lrml:Reference>` and `<lrml:LegalReference>`, which are not allowed to have these attributes"（指 @key/@keyref/@type）。
  - 邊（edge）`<lrml:hasReference>`："a Reference in the collection"（§3.6）；集合邊 `<lrml:hasReferences>`。
- **`<lrml:LegalSources>` / `<lrml:LegalSource>`（IRI 型法定來源；「NormativeSource」的正名）**
  - [S1] §3.4 原文："LegalRuleML: a formal representation of one or more LegalSources using the LegalRuleML Specifications."、"LegalSource(s) +: a source of one or more Legal Norms formulated in any format and endorsed by an Authority."
  - [S1] §4.3.1 原文："`<lrml:LegalSources>` is the element dedicated to record the IRI-based identifier sources."
  - 範例（[S1] §4.3.1 例 16, compact）：
    ```xml
    <lrml:LegalSources>
      <lrml:LegalSource key="ref2" sameAs="http://www.law.cornell.edu/uscode/text/17/504#psection-1"/>
    </lrml:LegalSources>
    ```
  - `@sameAs` 屬性："an IRI that denotes the same thing as the parent Node Element."（§3.7）；可掛在 `<lrml:Source>`, `<lrml:LegalSource>`, `<lrml:Agent>`, `<lrml:Authority>`, `<lrml:Jurisdiction>`（§5.5.1.1）。
  - Source/LegalSource 內可含子邊 `@hasType`、`@hasMemberType`（用於指定成員型別，§3.7）。
- **`<lrml:Sources>` / `<lrml:Source>`（非法定、外部來源）**
  - [S1] §4.3.1 原文："`<lrml:Sources>` is the element dedicated to record the IRI based identifier sources that are not legal text but that are important for the LegalRuleML modeling. … Usually this element is used to document the IRI to the external LegalRuleML rules modelled in another knowledge base (e.g., another XML file)."
  - 範例（[S1] §4.3.1 例 17, compact）：
    ```xml
    <lrml:Sources>
      <lrml:Source key="ex-rule_1b" sameAs="http://example.com/ex2.1.1-references-b#rule_1b"/>
      <lrml:Source key="oasis-rule_1b" sameAs="http://docs.oasis-open.org/legalruleml/examples/approved/ex2.1.1-references-b#rule_1b"/>
    </lrml:Sources>
    ```
- **Marker 界面模式**（官方推薦的識別用法；[S1] §5.9.3）："The syntax that implements the marker interface pattern in LegalRuleML consists of the following attributes: @key, @keyref, @sameAs, @refersTo, @refID."
- **Example withheld:** the previous fragment matched the generic API-key scanner. Recreate it only from the cited public source after review.

### 2.3 「NormativeSource」的核查結果（重要更正）

- **規格全文檢索 "NormativeSource"：0 次；[U4] lrml-compact.xsd 中該字串：0 次。**
- → **LegalRuleML 1.0（OASIS Standard, 2021-08-30）中不存在 `<NormativeSource>` 元素。** 官方構件名為 **`<lrml:LegalSource>`**（IRI 型法定來源）、**`<lrml:Source>`**（非法定外部來源）、**`<lrml:Reference>/<lrml:LegalReference>`**（非 IRI 識別碼）。
- 「NormativeSource 及其子構件」若要與 1.0 對應，官方做法不是「來源元素擁有子元素」，而是「來源以 @key 為錨點、由 `<lrml:Association>` 把來源與規則/片段配對」（見 §2.6），此外範例中可見 Association 甚至可把資訊掛到 atom 層級（如 `<lrml:toTarget keyref="#atom1"/>`、`<lrml:toTarget keyref="#body1"/>`，[S1] §4.3.5 附近例句）。
- 另外：沒有 `ExternalReferences` 元素（全文 0 次）；官方對應機制為 `<lrml:References>`/`<lrml:LegalReferences>`。

### 2.4 Rule / Statement 系構件

**集合與一般 Statement**

- **`<lrml:Statements>`（Statement 集合）**：[S1] §3.4 原文："Statements +: a collection where every member is a Legal Statement or a FactualStatement."
- **`<lrml:Statement>`（通用 Statement 節點）**：[S1] §5.5.1.1 原文："Alternative Nodes are Node elements that can belong to an Alternatives collection - these are `<lrml:Statement>` or `<lrml:Statements>`."
- 集合邊：`<lrml:hasStatement>`（"a Legal Statement in the collection"）/ `<lrml:hasStatements>`（"a collection of Legal Statements"）（§3.6）。
- Normalized 序列化之樣板：`<lrml:hasStatement>` → `<lrml:hasTemplate>` → `<ruleml:Rule>`（[S1] 例句可見於 §5.16 附近例與 Annex F 風格例）。§3.6 原文："hasTemplate: the template of a Legal Statement."

**PrescriptiveStatement（存在 ✅）**

- [S1] §3.4 原文："PrescriptiveStatement +: a Legal Statement which prescribes behaviors, e.g. with Permissions, Obligations, or Prohibitions on states, actions, or courses of actions."
- 骨架（[S1] §4.2.2 例 4, compact，原文節錄）：

  ```xml
  <lrml:Statements>
    <lrml:PrescriptiveStatement key="ps1">
      <ruleml:Rule key="ex:key1">
        <lrml:hasStrength>
          <lrml:StrictStrength key="str3" iri="http://www.w3.org/2001/XMLSchema#strict1"/>
        </lrml:hasStrength>
        <ruleml:if>
          <ruleml:Atom key=":atom1"/>
          …
        </ruleml:if>
        <ruleml:then>
          <lrml:SuborderList>
            ...
          </lrml:SuborderList>
        </ruleml:then>
      </ruleml:Rule>
    </lrml:PrescriptiveStatement>
  </lrml:Statements>
  ```
- 語義（[S1] §4.2.2 原文）："The difference between constitutive rules and prescriptive rules is in the content of the head, where the head of a prescriptive rule is list of deontic operators, i.e., [D1]formula1,…,[Dn]formulan which is called a suborder list …, and represented in LegalRuleML by the `<lrml:SuborderList>` block."

**ConstitutiveStatement（存在 ✅）**

- [S1] §3.4 原文："ConstitutiveStatement +: a Legal Statement that defines concepts and does not prescribe behaviors."
- 骨架（[S1] §4.2.2 例 5, compact）：`<lrml:ConstitutiveStatement key="ps1">` → `<ruleml:Rule>` → `ruleml:if`/`ruleml:then`（then 內為 `ruleml:Atom` 等，非 deontic）。語義："The conclusion (head) of a constitutive rule cannot be a deontic formula."

**FactualStatement**

- [S1] §3.4 原文："FactualStatement +: an expression of fact."

**Override / OverrideStatement（優先序）**

- [S1] §3.4 原文："Override +: an indication that a Legal Rule takes precedence over another Legal Rule."、"OverrideStatement +: a Legal Statement of an Override."
- 骨架（[S1] §4.2.1 例 1, compact）：
  ```xml
  <lrml:OverrideStatement>
    <lrml:Override over="#cs2" under="#cs1"/>
  </lrml:OverrideStatement>
  ```
- 屬性：`@over`（"the Legal Rule with higher priority"）、`@under`（"the Legal Rule with lower priority"）（§3.7；§5.5.1.1 "@over, @under (on `<lrml:Override>`)"）。
- 亦可掛在 Statements 集合上：[S1] §6.1 例（compact 原文節錄）：
  ```xml
  <lrml:Statements key="textblock1">
    <lrml:hasQualification>
      <lrml:Override over="#ps2" under="#ps1"/>
    </lrml:hasQualification>
    …
  ```
  邊語義（§3.6）："hasQualification: a qualification (e.g. an Override) of the Statements."

**Strength（規則強度/ 可敗性）**

- [S1] §3.4：`StrictStrength`（"the conclusion of the Legal Rule is indisputable"）、`DefeasibleStrength`、`Defeater`（"the opposite of the conclusion … does not hold"）、以及 "Strength: the quality of a Legal Rule to resist or not to resist a rebuttal."
- 表達位置：① `<ruleml:Rule>` 內 `<lrml:hasStrength>`（§4.2.2 例；§3.6 "hasStrength: the Strength of the Legal Rule."）；② `lrml:Context` 內 `<lrml:appliesStrength iri=…/>`（§4.2.1 例 2/32–34）。§4.2.1 原文："The difference is that `<lrml:Context>` localizes the strength of a rule, while the `<lrml:hasStrength>` block in effect relates the strength to the rule in all contexts."
- 屬性 `@strength` on `<ruleml:Rule>`（§5.5.1.1："@strength (on `<ruleml:Rule>`)"；§3.7 "@strength: the (defeasible) Strength of the Legal Rule."）

**ruleml:Rule（RuleML 命名空間之規則模板）**

- [S1] §3.5 原文："ruleml:Rule: a) a RuleML Rule encoding a Constitutive Statement. b) a RuleML Rule encoding a Prescriptive Statement." 另 "ruleml:Time +: a neutral temporal entity."
- 範例屬性：`@key`（xsd:ID 型；[S1] §5.19 "the xsd:ID datatype for @key on LegalRuleML elements is enforced by the XSD but not the RELAX NG schemas"）、`@closure="universal"`（§6.2 例）、`@keyref`（分散式定義，§5.12）。

### 2.5 Deontic 子構件（Obligation / Prohibition / Permission / Right）

- 存在性（[S1] §3.4 原文引句）：
  - "Obligation +: a Deontic Specification for a state, an act, or a course of action to which a Bearer is legally bound, and which, if it is not achieved or performed, results in a Violation."
  - "Prohibition +: a Deontic Specification for a state, an act, or a course of action to which a Bearer is legally bound, and which, if it is achieved or performed, results in a Violation."
  - "Permission (see also Right) +: a Deontic Specification for a state, an act, or a course of action where the Bearer has no Obligation or Prohibition to the contrary. …"
  - "Right (see also Permission) +: a Deontic Specification that gives a Permission to a party (the Bearer) and implies there are Obligations or Prohibitions on other parties (the AuxiliaryParty) …"
  - 另有 "Deontic Specification: An indication of what states are legal or illegal. …"（§3.2）
- 遞迴元素（[S1] §5.9.2 原文節錄）："In order to reduce redundancy through modularization, some of the collections elements are recursive…"；LegalRuleML 引入之遞迴邏輯構件為：
  - `<lrml:Obligation>`、`<lrml:Permission>`、`<lrml:Prohibition>`、`<lrml:Right>`
- 關鍵屬性：`@iri`（[S1] §5.5.1.1："@iri on Annotation Nodes, Role Nodes, LegalRuleML Deontic Nodes and Deontic Key Nodes"；§3.7 "@iri: an IRI providing details regarding the parent Node Element."）
- 當事人角色：
  - [S1] §3.4：`Bearer +` "a role in a Deontic Specification to which the Deontic Specification is primarily directed."；`AuxiliaryParty +` "a role in a Deontic Specification to which the Deontic Specification is related."
  - 骨架（[S1] §4.2.3.1 例 8, compact 原文節錄；同結構亦見 §6.3 例句 line 3298 前後）：
    ```xml
    <lrml:Obligation iri="ex:obl1">
      <ruleml:slot>
        <lrml:Bearer iri="ex:oblBearer"/>
        <ruleml:Ind>Y</ruleml:Ind>
      </ruleml:slot>
      <ruleml:Atom key="ex:atom2">
        <ruleml:Rel iri="ex:rel2"/>
        <ruleml:Ind>X</ruleml:Ind>
      </ruleml:Atom>
    </lrml:Obligation>
    ```
    以及（[S1] §6.3 例，原文節錄）：
    ```xml
    <lrml:Obligation key="penalty1-tblock1-obl1">
      <ruleml:slot>
        <lrml:Bearer iri="deovo:oblBearer"/>
        <ruleml:Var>Y</ruleml:Var>
      </ruleml:slot>
      <ruleml:slot>
        <lrml:AuxiliaryParty iri="deovo:auxParty"/>
        <ruleml:Var>X</ruleml:Var>
      </ruleml:slot>
      <ruleml:Atom key=":penalty1-tblock1-obl1-axm1">
        <ruleml:Rel iri="lovo:payStatutoryDamages"/>
        ...
      </ruleml:Atom>
    </lrml:Obligation>
    ```
- [U4] XSD 核對（以 lrml-compact.xsd 字串計數佐證構件存在）：`Obligation`×18、`Prohibition`×18、`Permission`×16、`PrescriptiveStatement`×23、`ConstitutiveStatement`×12、`PenaltyStatement`×15、`Reparation`×30、`Override`×29、`Association`×57、`LegalSource`×36、`Prefix`×18、`appliesModality`×9、`hasStatement`×9。

### 2.6 Penalty / Reparation（違反與制裁）

- **`<lrml:PenaltyStatement>`**：[S1] §3.4 原文："PenaltyStatement +: a Legal Statement of a sanction (e.g. a punishment or a correction)."
- **`<lrml:SuborderList>`**：[S1] §3.4 原文："SuborderList: A Deontic Specification consisting of a sequence of Deontic Specifications other than SuborderLists (at any depth). When a SuborderList holds, a Deontic Specification in the SuborderList holds if all Deontic Specifications that precede it in the SuborderList have been violated."
  - 順序有語義：[S1] §5.10.1 "Examples of this design pattern include `<lrml:SuborderList>` and the positional (`<ruleml:arg>`) children of `<ruleml:Atom>` and `<ruleml:Expr>`."
  - 可省略（trivial list）：[S1] §4.2.3.2 原文："Notice that the `<lrml:SuborderList>` node might be skipped in case of a trivial non empty list of a single element."
- **`<lrml:ReparationStatement>` / `<lrml:Reparation>`**：[S1] §3.4 原文："Reparation +: an indication that a PenaltyStatement is linked with a PrescriptiveStatement. It indicates that a sanction may apply where the PrescriptiveStatement entails a Deontic Specification and when there is a Violation of the Deontic Specification."；"ReparationStatement: a Legal Statement of a Reparation."
  - 專用邊（§3.6）："appliesPenalty: the PenaltyStatement that is linked to a LegalRule in a Reparation."、"toPrescriptiveStatement: the PrescriptiveStatement that is linked to a PenaltyStatement in a Reparation."
  - 骨架（[S1] §4.2.3.2 例 10、§6.1）：
    ```xml
    <lrml:ReparationStatement key="rep1">
      <lrml:Reparation key="assoc1">
        <lrml:appliesPenalty keyref="#pen1"/>
        <lrml:toPrescriptiveStatement keyref="#ps1"/>
      </lrml:Reparation>
    </lrml:ReparationStatement>
    ```
- 完整「規則 → 罰則」骨架實例（[S1] §6.1 澳洲例, compact 節錄）：
  ```xml
  <lrml:Statements key="textblock1">
    ...
    <lrml:PrescriptiveStatement key="ps1">
      <ruleml:Rule key=":rule1" closure="universal">
        <lrml:hasStrength><lrml:DefeasibleStrength key="str1" iri="&defeasible-ontology;#defeasible1"/></lrml:hasStrength>
        <ruleml:if><ruleml:Atom><ruleml:Rel iri=":person"/><ruleml:Var>X</ruleml:Var></ruleml:Atom></ruleml:if>
        <ruleml:then>
          <lrml:SuborderList>
            <lrml:Prohibition>
              <ruleml:Atom><ruleml:Rel iri=":engageCreditActivity"/><ruleml:Var>X</ruleml:Var></ruleml:Atom>
            </lrml:Prohibition>
          </lrml:SuborderList>
        </ruleml:then>
      </ruleml:Rule>
    </lrml:PrescriptiveStatement>
    ...
    <lrml:PenaltyStatement key="pen2">
      <lrml:SuborderList>
        <lrml:Obligation>...</lrml:Obligation>
        <lrml:Obligation>...</lrml:Obligation>
      </lrml:SuborderList>
    </lrml:PenaltyStatement>
    <lrml:ReparationStatement key="rep1">…</lrml:ReparationStatement>
  </lrml:Statements>
  ```

### 2.7 Association / Context ——「規則引用某文件某節」的官方錨點方式

- **`<lrml:Association>`**：[S1] §3.4 原文："Association(s): a partial description of the extension of some relations where each non-target entity is paired with every target entity."
- **N:M 連結機制**（[S1] §4.3.1 原文）："LegalRuleML dedicates two elements (`<lrml:References>`, `<lrml:LegalSources>`) to annotate the original legal sources and to connect them to rules, so permitting an N:M relationship (e.g. many rules in relation to one textual provision; many textual provisions for one rule)."
- **基本骨架**（[S1] §4.3.1 例 18/19, compact）：
  ```xml
  <lrml:Association>
    <lrml:appliesSource keyref="#ref1"/>
    <lrml:toTarget keyref="#rule1"/>
    <lrml:toTarget keyref="#rule2"/>
  </lrml:Association>

  <lrml:Association>
    <lrml:appliesSource keyref="#ref1"/>
    <lrml:appliesSource keyref="#ref2"/>
    <lrml:toTarget keyref="#rule1"/>
  </lrml:Association>
  ```
- **掛入集合**：`<lrml:Associations>`（[S1] §4.4.1 例 30，key="sourceBlock1"）。
- **`<lrml:Context>`**（[S1] §3.4 原文）："Context +: an application of Associations to their target entities within a Scope."
  - [S1] §4.4.2 原文："…we introduce the `<lrml:Context>` element, which permits the description of all the characteristics that are linked to a particular rule (e.g., rule1) using the operator `<lrml:applies*>`…"
  - 骨架（[S1] §4.4.2 例 38, compact 原文節錄）：
    ```xml
    <lrml:Context key="ruleInfo4" hasCreationDate="#t1">
      <lrml:appliesSource keyref="#sec504-clsc-pnt2"/>
      <lrml:appliesTemporalCharacteristics keyref="#tblock1"/>
      <lrml:appliesStrength iri="/ontology/defeater"/>
      <lrml:appliesAuthority keyref="#congress"/>
      <lrml:appliesJurisdiction keyref="#us"/>
      <lrml:appliesAssociations keyref="#assoc1"/>
      <lrml:appliesAlternatives keyref="#alt2"/>
      <lrml:inScope keyref="#rule1"/>
      <lrml:inScope keyref="#rule4"/>
    </lrml:Context>
    ```
    （`inScope`：[S1] §3.6 "the Statement or (collection of) Statements that the Context is applied to."；`toTarget`："the target to which properties are applied by the Association."）
- **「某文件某節」的三種官方錨點做法**（全部搭配 `@key`（xsd:ID）＋ `@keyref`（或 `href="#…"` 形式的 keyref 值）使用）：
  1. IRI 型：`<lrml:LegalSource key="ls1" sameAs="http://www.comlaw.gov.au/Details/C2009A00134/Html/Text#param43"/>`（[S1] §6.1；`#param43` 即節/款錨點片段）。
  2. 非 IRI 型：`<lrml:LegalReference refersTo="ref1" refID="/au/2012-05-30/C628:2012/eng@/main#sec2.2" refIDSystemName="AkomaNtoso3.0-2016-03"/>`（[S1] §4.3.1），`refIDSystemName` 註明命名規約（Akoma Ntoso、ECLI 等）。
  3. 外部規則庫粒度：`<lrml:Source key="…" sameAs="…#rule_1b"/>`（[S1] §4.3.1）。
- **isomorphic 對照之官方名目**（[S1] §2.3 原文）："Linking Rules and Provisions: LegalRuleML includes a mechanism, based on IRI, that allows many to many (N:M) relationships among the rules and the textual provisions…"；§4.3.1 原文："Combining `<lrml:References>/<lrml:LegalSources>`/`<lrml:Sources>` and `<lrml:Association>`, we can implement the principle of isomorphism."
- 粒度說明（[S1] §4.3.1 原文）："The mechanism must support a fine granularity (rules, fragments of rules, atoms, fragments of atoms connected with provisions, fragments of provisions, letters, numbers, paragraphs, sentences, and word) as well as represent temporal modifications."

### 2.8 Deontic 操作子的兩種官方表達方式

- 概念層操作子（[S1] §4.2.3.1 原文）："We will use [OBL] for the modal/deontic operator of Obligation, [PER] for Permission, and [FOR] for Prohibition (or Forbidden)."
- XML 層的兩個「語義指涉」機制（[S1] §4.2.3.1 原文節錄）：
  - "LegalRuleML is equipped with two mechanisms to point to the semantics of various Deontic Specifications (see also the glossary) in a document. The first mechanism is provided by the iri attribute of a Deontic Specification, for example."
  - 機制一（例 6）：
    ```xml
    <lrml:Obligation key="oblig1" iri="ex:achievementObligation">…</lrml:Obligation>
    ```
  - 機制二（例 7 原文引句）："The second mechanism is to use an Association to link a Deontic Specification to its meaning using the `<lrml:applyModality>` element, namely:"
    ```xml
    <lrml:Association>
      <lrml:appliesModality iri="ex:maintenanaceObligation"/>
      <lrml:toTarget keyref="#obl101"/>
    </lrml:Association>
    ```
  - ⚠️ **名稱不一致記錄**：例 7 的正文文字寫作 `applyModality`，但同一文的 §4.4.1 例 31（`<lrml:appliesModality iri="/ontology/deontic/"/>`）、§3.6 詞彙表（"appliesModality: the deontic mode that applies to a Deontic Specification in a Context or Association"）、其他範例（§6.3）以及規範性 schema（[U4] XSD 中 `appliesModality` 出現 9 次）皆為 **`appliesModality`**。匯出器請依 schema 的 `appliesModality` 為準。
- 遞迴嵌套能力：[S1] §5.9.2 原文："recursive element (e.g., `<lrml:Obligation>` can have descendants that are other `<lrml:Obligation>` elements)"（§5.9 樣式 3 列表）。
- 「Violations」定義（[S1] §3.4）："Violation +: an indication that an Obligation or Prohibition has been violated."；"Compliance +: an indication that an Obligation has been fulfilled or a Prohibition has not been violated."
- **任務候選名核查**：`IsActive` / `IsObligated` —— 規格全文檢索 **0 次**，1.0 中 **不存在**。1.0 裡與「狀態/暫時性」相關的官方機制是 TemporalCharacteristic（見 §2.9），其中「InForce / Efficacious」等是 **狀態 IRI**，不是規則層的 deontic 操作子。

### 2.9 TemporalCharacteristic（時間性狀態）

- [S1] §3.4 原文："TemporalCharacteristic(s) +: a pair of Time with a qualification, which consists of a Legal Status and a Status Development, such that the qualification holds at the Time."
- 相關狀態詞（[S1] §3.2 原文）：Legal Status "a standing that can apply to a Legal Norm at a Time, e.g., "is applicable", "is in force", "has efficacy", "is valid""。
- 骨架（[S1] §4.3.5 例 27, compact 原文節錄）：
  ```xml
  <lrml:TemporalCharacteristics key="tblock1">
    <lrml:TemporalCharacteristic key="nev1">
      <lrml:forStatus iri="http://docs.oasis-open.org/legalruleml/ns/v1.0/vocab#Efficacious"/>
      <lrml:hasStatusDevelopment iri="http://docs.oasis-open.org/legalruleml/ns/v1.0/vocab#Starts"/>
      <lrml:atTime keyref="#t1"/>
    </lrml:TemporalCharacteristic>
    <lrml:TemporalCharacteristic key="nev2">
      <lrml:forStatus iri="http://docs.oasis-open.org/legalruleml/ns/v1.0/vocab#InForce"/>
      <lrml:hasStatusDevelopment iri="http://docs.oasis-open.org/legalruleml/ns/v1.0/vocab#Starts"/>
      <lrml:atTime keyref="#t2"/>
    </lrml:TemporalCharacteristic>
  </lrml:TemporalCharacteristics>
  ```
- 邊語義（[S1] §3.6）："atTime: the Time of the qualification of a TemporalCharacteristic."；"forStatus: the Legal Status of the qualification…"；"hasStatusDevelopment: the Status Development of the qualification…"；"appliesTemporalCharacteristics: a collection of TemporalCharacteristics applied by the Context or Association."
- `ruleml:Time`（[S1] §3.5；例 27 之前文）：`<ruleml:Time key=":t1"><ruleml:Data xsi:type="xs:dateTime">1978-01-01T00:00:00</ruleml:Data></ruleml:Time>`；root 的 `@hasCreationDate` 也指向 `ruleml:Time`。

### 2.10 Alternatives（替代解釋）

- [S1] §3.4 原文："Alternatives +: a mutually exclusive collection where every member is a LegalRuleML rendering of one or more Legal Norms."
- 骨架（[S1] §4.2.4 例 11, compact）：
  ```xml
  <lrml:Alternatives key="alt1">
    <lrml:fromLegalSources keyref="#t1"/>
    <lrml:hasAlternative keyref="#rule1"/>
    <lrml:hasAlternative keyref="#rule2"/>
  </lrml:Alternatives>
  ```
  邊語義："fromLegalSources: the LegalSources from which the Alternatives are derived."

### 2.11 未收錄之任務候選名（未能证实存在）

| 任務候選名 | 核查結果 |
|-----------|---------|
| `NormativeSource` | ❌ 不存在（全文 0 次；XSD 0 次）→ 對應官方構件 `<lrml:LegalSource>` / `<lrml:Source>` |
| `LegalSchema`（根元素）| ❌ 不存在 → 根元素為 `<lrml:LegalRuleML>` |
| `PrefixDeclaration` | ❌ 不存在 → 對應官方構件 `<lrml:Prefix>` |
| `ExternalReferences` | ❌ 不存在 → 對應官方構件 `<lrml:References>` / `<lrml:LegalReferences>` |
| `IsActive` / `IsObligated`（deontic 操作子）| ❌ 不存在 → deontic 由 `<lrml:Obligation>/<lrml:Prohibition>/<lrml:Permission>/<lrml:Right>` 節點表達 |

---

## 三、屬性速查表（[S1] §3.7 原文引句）

| 屬性 | 官方語義（原文） | 適用處 |
|------|------------------|--------|
| `@key` | "a Node Element label." | LRML Node（xsd:ID；§5.19） |
| `@keyref` | "a Node Element reference." | 引用本地 key（值形如 `#ps1`） |
| `@iri` | "an IRI providing details regarding the parent Node Element." | 多數 Node；註解/角色/Deontic 節點必用 |
| `@sameAs` | "an IRI that denotes the same thing as the parent Node Element." | Source/LegalSource/Agent/Authority/Jurisdiction |
| `@refersTo` | "the internal ID of the Reference." | Reference / LegalReference |
| `@refID` | "the external ID of the Reference."（§5.20：xs:string，可為 CURIE-like） | Reference / LegalReference / Prefix |
| `@pre` | "the prefix in a Prefix declaration, following CURIE conventions." | Prefix |
| `@refIDSystemName` / `@refIDSystemSource` / `@refType` | "the name / IRI source / conceptual type …of the ID system of the Reference" | Reference(s) / LegalReference(s) |
| `@strength` | "the (defeasible) Strength of the Legal Rule." | ruleml:Rule |
| `@over` / `@under` | "the Legal Rule with higher / lower priority." | Override |
| `@type` / `@memberType` | "the type or class of the parent Node Element / members of the collection." | Node / Collection Node |
| `@hasCreationDate` | "the creation date of the Context or LegalRuleML document."（值為 keyref 到 ruleml:Time） | LegalRuleML / Context |

其他：集合邊上常配 `<lrml:hasMemberType>`（§3.6）或 `@hasMemberType`（例句 `@hasMemberType` 見 §1.4 舉例）；Associator 類元素會用 `@iri` 直接掛外部詞彙（如 `appliesStrength iri="/ontology/defeatable"`、`appliesModality iri="/ontology/deontic/"`）。@key/@keyref/@type 之共通可選屬性組（commonLRMLNodeInit.attlist）見 [S1] §5.5.1.1。

---

## 四、映射建議初版（僅描述官方語義，不發明新構件）

> 適用前提：匯出目標選 **Compact 序列化**（較貼近規則引擎資料結構；Normalized 需要 `hasStatement/hasTemplate` 包裝）。驗證方案按 [S1] §7 #1：以 `<lrml:LegalRuleML>` 為根、通過 `lrml-compact.xsd`（[U4]）或 `lrml-compact.rnc`（[U2]）其一（或兩者）。

**建議骨架（各構件名皆有上文出處）**：

```xml
<lrml:LegalRuleML xmlns:lrml="http://docs.oasis-open.org/legalruleml/ns/v1.0/"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                  xmlns:ruleml="http://ruleml.org/spec">
  <!-- Comment* -->
  <!-- 前綴（第 1 類） -->
  <lrml:Prefix pre="ex"  refID="http://example.org/vocab#"/>
  <lrml:Prefix pre="deovo" refID="…deontic-vocab#"/>        <!-- 例：官方 §4.2.1 例 2 之做法 -->
  <!-- 元資料集合（第 2 類，順序可互調但類別不可亂） -->
  <lrml:LegalSources>
    <lrml:LegalSource key="ls1" sameAs="…act#section-29"/>   <!-- §4.3.1 / §6.1 -->
  </lrml:LegalSources>
  <lrml:TemporalCharacteristics key="tblock1">…</lrml:TemporalCharacteristics>  <!-- §4.3.5 -->
  <!-- 第 3 類 -->
  <lrml:Associations key="assocBlock1">
    <lrml:Association>
      <lrml:appliesSource keyref="#ls1"/>          <!-- 該規則引用某文件某節 -->
      <lrml:appliesTemporalCharacteristics keyref="#tblock1"/>
      <lrml:appliesAuthority keyref="#…"/> <lrml:appliesJurisdiction keyref="#…"/>
      <lrml:toTarget keyref="#ps1"/> <lrml:toTarget keyref="#pen1"/>
    </lrml:Association>
  </lrml:Associations>
  <!-- 第 4 類 -->
  <lrml:Context key="ctx1">
    <lrml:appliesAssociations keyref="#assocBlock1"/>
    <lrml:inScope keyref="#ps1"/>
  </lrml:Context>
  <lrml:Statements key="stmtBlock1">
    <!-- 優先序（若有） -->
    <lrml:OverrideStatement><lrml:Override over="#ps2" under="#ps1"/></lrml:OverrideStatement>
    <!-- 義務性規則 -->
    <lrml:PrescriptiveStatement key="ps1">
      <ruleml:Rule key=":rule1" closure="universal">
        <lrml:hasStrength><lrml:DefeasibleStrength key="str1" iri="…#defeasible1"/></lrml:hasStrength>
        <ruleml:if>
          <ruleml:And>
            <ruleml:Atom><ruleml:Rel iri=":person"/><ruleml:Var>X</ruleml:Var></ruleml:Atom>
            <ruleml:Atom><ruleml:Rel iri=":not-licensed"/><ruleml:Var>X</ruleml:Var></ruleml:Atom>
          </ruleml:And>
        </ruleml:if>
        <ruleml:then>
          <lrml:SuborderList>
            <lrml:Prohibition>
              <ruleml:Atom><ruleml:Rel iri=":engageCreditActivity"/><ruleml:Var>X</ruleml:Var></ruleml:Atom>
            </lrml:Prohibition>
          </lrml:SuborderList>
        </ruleml:then>
      </ruleml:Rule>
    </lrml:PrescriptiveStatement>
    <!-- 定義性規則 -->
    <lrml:ConstitutiveStatement key="cs1">…(if/then 以 ruleml:Atom 為頭)…</lrml:ConstitutiveStatement>
    <!-- 罰則 + 修復關聯 -->
    <lrml:PenaltyStatement key="pen1">
      <lrml:SuborderList>
        <lrml:Obligation key="pen1-obl1" iri="ex:achievementObligation">
          <ruleml:slot><lrml:Bearer iri="deovo:oblBearer"/><ruleml:Var>X</ruleml:Var></ruleml:slot>
          <ruleml:Atom>…</ruleml:Atom>
        </lrml:Obligation>
      </lrml:SuborderList>
    </lrml:PenaltyStatement>
    <lrml:ReparationStatement key="rep1">
      <lrml:Reparation key="rep1-assoc">
        <lrml:appliesPenalty keyref="#pen1"/>
        <lrml:toPrescriptiveStatement keyref="#ps1"/>
      </lrml:Reparation>
    </lrml:ReparationStatement>
  </lrml:Statements>
</lrml:LegalRuleML>
```

**規則引擎概念 → LegalRuleML 1.0 構件對照**（僅官方語義）：

| 引擎概念 | 官方表達 | 依據 |
|----------|---------|------|
| 一條規則（結果論/行為令） | `lrml:Statements > lrml:PrescriptiveStatement` + `ruleml:Rule`（`ruleml:if` / `ruleml:then` + `lrml:SuborderList`） | [S1] §3.4、§4.2.2、§6.1 |
| 概念定義規則 | `lrml:ConstitutiveStatement`（then 頭不可為 deontic 公式） | [S1] §3.4、§4.2.2 |
| 事實前提 | `ruleml:Atom`（`ruleml:Rel/@iri`、`ruleml:Var/Ind`）或 `lrml:FactualStatement` | [S1] §3.4、§6.1 |
| 必須做 / 禁止 / 可 / 權利 | `lrml:Obligation` / `lrml:Prohibition` / `lrml:Permission` / `lrml:Right` 節點（放進 `lrml:SuborderList` 或直接為 rule 內容）；操作子的子類語義以 `@iri` 或 `lrml:appliesModality`（Association）指向外部詞彙 | [S1] §3.4、§4.2.3.1、§5.9.2 |
| 規則主體（誰負責/誰受益） | deontic 節點內 `ruleml:slot` + `lrml:Bearer` / `lrml:AuxiliaryParty`（角色 `@iri` 指向 vocab；Actor 可再建模為 `lrml:Agent/lrml:Figure/lrml:Role`） | [S1] §3.4、§4.2.3.1 例 8、§6.3 |
| 屬性成效/可敗性（strict/defeasible/defeater） | `lrml:hasStrength` + `lrml:StrictStrength/DefeasibleStrength/Defeater`，或 `lrml:Context > lrml:appliesStrength @iri`；另 `ruleml:Rule @strength` 屬性 | [S1] §3.4、§4.2.1、§3.7 |
| 規則衝突優先序 | `lrml:OverrideStatement > lrml:Override @over/@under`（或掛 `lrml:hasQualification`） | [S1] §3.4、§4.2.1、§6.1 |
| 「本規則所依據的法條」 | `lrml:LegalSources > lrml:LegalSource @key @sameAs`（IRI 含片段錨點）→ `lrml:Association > lrml:appliesSource @keyref` + `lrml:toTarget @keyref`；或以 `lrml:Context > lrml:appliesSource` | [S1] §4.3.1、§4.4.1、§6.1 |
| 引用來源非 IRI 識別碼（Akoma Ntoso / ECLI 等） | `lrml:LegalReferences > lrml:LegalReference @refersTo @refID @refIDSystemName（@refIDSystemSource, @refType）`；Association 同上以 `appliesSource` 接到 Reference 的內部 ID | [S1] §3.4、§4.3.1、§3.7 |
| 引用外部規則庫/知識庫檔案 | `lrml:Sources > lrml:Source @key @sameAs` + `lrml:Association` | [S1] §4.3.1 |
| 違反後果/罰則 | `lrml:PenaltyStatement`（內含 `lrml:SuborderList` 或省略為單一 deontic 節點），經 `lrml:ReparationStatement > lrml:Reparation`（`lrml:appliesPenalty` + `lrml:toPrescriptiveStatement`）連回義務條款 | [S1] §3.4、§4.2.3.2、§6.1、§6.3 |
| 生效/失效、適用期間 | `lrml:TemporalCharacteristics > lrml:TemporalCharacteristic`（`lrml:forStatus @iri`、`lrml:hasStatusDevelopment @iri`、`lrml:atTime @keyref`），時間以 `lrml:Times > ruleml:Time + ruleml:Data xs:dateTime`；掛到規則/來源用 `lrml:appliesTemporalCharacteristics` | [S1] §3.4、§4.3.5（例 27） |
| 依管轄/授權差異化 | `lrml:Context > lrml:appliesJurisdiction / lrml:appliesAuthority`（+ `inScope`） | [S1] §4.4.2 |
| 同一規則的多種解讀 | `lrml:Alternatives > lrml:fromLegalSources` + `lrml:hasAlternative` | [S1] §4.2.4 |
| 前綴縮寫/CURIE | `lrml:Prefix @pre @refID`（僅此元素可定義 CURIE 前綴） | [S1] §3.4、§4.2.1、§5.20 |

**官方注意事項（匯出器必讀）**

1. **頂層順序**：Comment* → Prefix* → 元資料集合* → Associations* → (Alternatives | Context | Statements)*（[S1] §5.16）。
2. **`@key` 唯一性**：LegalRuleML 元素的 `@key` 在 XSD 為 xsd:ID（文件內唯一）；`@keyref` 值形如 `#key`（[S1] §5.11/§5.19；XSD 端強制，RNC 端不強制）。
3. **`@type/@key/@keyref` 不通用於 Reference/LegalReference**（[S1] §5.5.1.1）。
4. **Collection 語義為集合**（無序、不重複）（[S1] §5.9.1），但 `lrml:SuborderList` 的子元素順序**有語義**，需依序輸出（[S1] §5.10.1、§4.2.3.2）。
5. **Deontic 節點可遞迴**（`lrml:Obligation` 內可再含 `lrml:Obligation` 等；[S1] §5.9/§5.9.2）。
6. **驗證**：compact 文件 MUST 以 lrml-compact.xsd（[U4]）或 lrml-compact.rnc（[U2]）之一驗證（或兩者皆過）（[S1] §7 #1）；另有 §7 #2 之 schema 外條件（如 CURIE 展開後符合 RFC 3987 等）。
7. **脫字與名稱陷阱**：以 schema 為準的名稱——`appliesModality`（正文例 7 曾寫 `applyModality`，應為筆誤）；不要使用任務直覺名 `NormativeSource`、`PrefixDeclaration`、`ExternalReferences`、`LegalSchema`、`IsActive`、`IsObligated`（1.0 均不存在，見 §2.11）。

---

## 五、未能核实 / 待補清單（截至 2026-10-25）

1. **https://legalruleml.oasis-open.org/**：DNS 解析失敗（兩次嘗試），未能核实；規範內容以 docs.oasis-open.org 為準。
2. 規格封面列出之 `docx/pdf/zip`（及其 xslt/generation/rdfs/diagrams/examples 目錄）未個別抓取。
3. `lrml-basic.xsd`、`lrml-normal.xsd`、`lrml-basic.rnc`、`lrml-normal.rnc` 內容未逐字核對（檔案清單已由目錄索引 [U1]/[U3] 證實存在）。
4. RDFS metamodel（…/os/rdfs/）之類別定義未抓取；[S1] §3.9 僅提示 "RDFS (see Annex C) is used to define the LegalRuleML metamodel"。
5. [S1] §4.3.3 Jurisdiction / §4.3.4 Authority 細節未逐字摘錄（本文僅收錄詞彙表定義與 `appliesJurisdiction/appliesAuthority` 邊名）；如匯出器需建模管轄/授權物件，可再補抓該節。
6. Normalized 序列化的完整編排（`lrml:hasStatements > lrml:hasStatement > lrml:hasTemplate > ruleml:Rule`；`lrml:Override` 在 Normal form 位於 `lrml:hasTemplate` 內）引用自 [S1] §4.2.1 與 Annex F 例句片段，未對 Annex F 全文逐字核對。
7. 例 7 正文之 `<lrml:applyModality>`與規範 schema 之 `appliesModality` 名稱不一致，已如實登記於 §2.8；最終以 schema 為準。

（完）
