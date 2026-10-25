# Configuration Reference

## Engine Parameters (config/engine-params.json)

| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| retrieval_threshold | 0.70 | 0.0-1.0 | Minimum retrieval score for evidence inclusion |
| invariants | (locked) | — | Fail-closed invariant set |

## Data Files

| File | Description |
|------|-------------|
| data/docs.json | Versioned documents with clauses, anchors, injection directives |
| data/rules.json | Active rules with deontic/scope/sequencing/exceptions/temporal/evidence_requirements |
| data/processes.json | Process models with precedence pairs, repeat policy, forbidden pairs |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| DATABASE_URL | — | PostgreSQL connection string (R4) |
| RUST_LOG | info | Log level |
