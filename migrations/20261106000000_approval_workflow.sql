-- Approval workflow + event outbox (Phase C, extends existing schema)
-- Additive only: does not modify existing migrations.

CREATE TABLE IF NOT EXISTS approval_workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    definition_json TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS approval_stages (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES approval_workflows(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    approver_rule TEXT NOT NULL,
    sla_seconds BIGINT NOT NULL,
    urgency TEXT NOT NULL CHECK (urgency IN ('low','normal','high','critical')),
    conditions_json TEXT NOT NULL DEFAULT '[]',
    escalation_path_json TEXT NOT NULL DEFAULT '[]',
    on_timeout TEXT NOT NULL DEFAULT 'escalate'
);

CREATE TABLE IF NOT EXISTS approval_tasks (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    workflow_version INTEGER NOT NULL,
    stage_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('pending','approved','rejected','escalated','expired','cancelled')),
    urgency TEXT NOT NULL CHECK (urgency IN ('low','normal','high','critical')),
    amount_cents BIGINT NOT NULL,
    platform TEXT NOT NULL,
    classification TEXT NOT NULL,
    risk_score INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    due_at TIMESTAMPTZ NOT NULL,
    escalated_to TEXT,
    escalation_count INTEGER NOT NULL DEFAULT 0,
    agent_recommendation_json TEXT
);

CREATE TABLE IF NOT EXISTS channel_registry (
    platform_id TEXT PRIMARY KEY,
    config_json TEXT NOT NULL,
    healthy BOOLEAN NOT NULL DEFAULT FALSE,
    last_health_check TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS event_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_jsonb JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    attempt INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_approval_tasks_state ON approval_tasks(state);
CREATE INDEX IF NOT EXISTS idx_approval_tasks_due ON approval_tasks(due_at);
CREATE INDEX IF NOT EXISTS idx_event_outbox_unpublished ON event_outbox(published_at) WHERE published_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_event_outbox_aggregate ON event_outbox(aggregate_type, aggregate_id);