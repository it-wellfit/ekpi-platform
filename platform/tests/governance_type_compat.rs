// ADR-0004 compatibility lock: the governance event payloads are a single type
// family. These fixtures are the on-disk JSONL format written by earlier
// revisions, so every variant must keep deserializing and re-serializing
// byte-for-byte. The test lives in `platform` because that crate owns the
// ledger file format; it also proves the platform and domain definitions are
// interchangeable before the re-export migration.

fn assert_wire_format(json: &str) {
    let platform: ekp_platform::governance::EventPayload = serde_json::from_str(json)
        .unwrap_or_else(|e| panic!("platform deserialization failed for {json}: {e}"));
    let domain: ekp_domain::governance::EventPayload = serde_json::from_str(json)
        .unwrap_or_else(|e| panic!("domain deserialization failed for {json}: {e}"));

    let platform_json = serde_json::to_string(&platform).expect("serialize platform payload");
    let domain_json = serde_json::to_string(&domain).expect("serialize domain payload");

    assert_eq!(
        platform_json, json,
        "platform round-trip changed the wire format"
    );
    assert_eq!(
        domain_json, json,
        "domain round-trip changed the wire format"
    );
    assert_eq!(
        platform_json, domain_json,
        "the two definitions must serialize identically"
    );
}

#[test]
fn every_event_payload_keeps_its_jsonl_wire_format() {
    let fixtures = [
        r#"{"project_created":{"id":"P-1","name":"專案","department":"品保","owner":"owner","risk_tier":"high","created_at":1700000000}}"#,
        r#"{"evidence_linked":{"id":"E-1","project_id":"P-1","title":"證據","source_system":"ERP","source_url":"https://example.invalid/e1","source_version":"v1","evidence_type":"record","owner":"owner","created_at":1700000001}}"#,
        r#"{"evidence_reviewed":{"evidence_id":"E-1","result":"accepted","reviewer":"reviewer","note":"ok","reviewed_at":1700000002}}"#,
        r#"{"workflow_saved":{"id":"W-1","version":1,"title":"流程","department":"品保","owner":"owner","description":"desc","stages":["a","b"],"status":"draft","created_at":1700000003,"published_at":null,"published_by":null}}"#,
        r#"{"workflow_published":{"id":"W-1","version":1,"reviewer":"reviewer","published_at":1700000004}}"#,
        r#"{"quality_case_created":{"id":"C-1","project_id":"P-1","kind":"CAPA","title":"案例","description":"desc","owner":"owner","due_date":"2026-11-30","status":"open","created_at":1700000005,"updated_at":1700000005,"closure_note":null}}"#,
        r#"{"quality_case_updated":{"id":"C-1","status":"closed","actor":"approver","note":"done","updated_at":1700000006}}"#,
        r#"{"quality_inspection_recorded":{"id":"I-1","project_id":"P-1","reviewer":"reviewer","created_at":1700000007,"results":[{"item_id":"QC-01","result":"pass","note":"ok","evidence_reference":"E-1","issue_id":""}]}}"#,
        r#"{"gate_decision_recorded":{"id":"G-1","project_id":"P-1","gate_id":"GATE-1","decision":"approved","actor":"approver","reason":"ok","evidence_reference":"E-1","risk_acceptor":"","remediation_due":"","non_waivable":true,"created_at":1700000008}}"#,
    ];

    for fixture in fixtures {
        assert_wire_format(fixture);
    }
}
