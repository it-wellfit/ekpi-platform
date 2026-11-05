-- Audit chain sequence.
--
-- The sequence hands out the event number. Every governance record identifier
-- is derived from it, so identifiers are unique even when two writers race.

CREATE SEQUENCE governance_audit_sequence START 1 INCREMENT BY 1;

ALTER TABLE governance_audit_events
    ALTER COLUMN sequence SET DEFAULT nextval('governance_audit_sequence');
