-- Approval event chain (PG-A): canonical append-only, hash-chained approval
-- store for the approval bounded context. Additive; mirrors the
-- governance_audit_events shape and append-only trigger.
--
-- The event chain is the single source of truth; approval_tasks (see
-- 20261106000000 + 20261110000000) is a queryable projection maintained
-- inside the same transaction as each append.

CREATE TABLE IF NOT EXISTS approval_events (
    sequence        BIGINT PRIMARY KEY,
    occurred_at     BIGINT NOT NULL,
    actor           TEXT NOT NULL,
    payload_jsonb   JSONB NOT NULL,
    previous_sha256 TEXT NOT NULL,
    sha256          TEXT NOT NULL
);

CREATE OR REPLACE FUNCTION approval_events_are_append_only()
RETURNS TRIGGER
LANGUAGE plpgsql
AS
$$
BEGIN
    RAISE EXCEPTION 'approval_events is append-only; % is not allowed', TG_OP;
END
$$;

DROP TRIGGER IF EXISTS approval_events_append_only ON approval_events;

CREATE TRIGGER approval_events_append_only
    BEFORE UPDATE OR DELETE ON approval_events
    FOR EACH ROW
    EXECUTE FUNCTION approval_events_are_append_only();
