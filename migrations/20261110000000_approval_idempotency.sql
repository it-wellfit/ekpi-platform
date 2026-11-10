-- Approval task idempotency (S10, additive).
-- Same Idempotency-Key returns the original task without a second row.

ALTER TABLE approval_tasks
    ADD COLUMN IF NOT EXISTS idempotency_key TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_approval_tasks_idempotency_key
    ON approval_tasks(idempotency_key)
    WHERE idempotency_key IS NOT NULL;
