# EKPI domain glossary

Use these terms consistently. The linked product documents remain authoritative when this summary is incomplete.

| Term | Meaning in this project | Source |
| --- | --- | --- |
| EKPI | Evidence-Grounded Enterprise Knowledge & Process Intelligence Platform | [README](README.md) |
| Project | A governed initiative with an immutable project ID, defined purpose and scope, owners, risk tier, and classification. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| Requirement | A versioned business or system need with a source, acceptance method, owner, and trace links. | [Functional specification](docs/governance/05-functional-spec-v1.md) |
| Workflow template version | An immutable, versioned definition of stages, gates, roles, deliverables, and rules. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| Workflow run | A project-specific execution pinned to one published template version. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| Stage | An ordered lifecycle activity with entry/exit criteria, activities, deliverables, and roles. | [Functional specification](docs/governance/05-functional-spec-v1.md) |
| Gate | A controlled decision point whose required conditions and approver roles are defined by a workflow version. | [Functional specification](docs/governance/05-functional-spec-v1.md) |
| Evidence reference | A pointer to one source document and one version; the platform records a reference and does not copy the authoritative source. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| Evidence link | The explicit relationship connecting an evidence reference to a requirement, deliverable, gate, or quality check. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| Quality case | A typed risk, change, defect, or CAPA record with a controlled lifecycle. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |
| CAPA | Corrective and preventive action; closure requires verification and effectiveness confirmation where applicable policy requires it. | [Functional specification](docs/governance/05-functional-spec-v1.md) |
| Audit event | An append-only event that records a governed action and supports integrity and traceability checks. | [Data and state model](docs/governance/06-data-and-state-model-v1.md) |

Do not use “document uploaded” as a synonym for an evidence reference: the current prototype stores source references, not source-file contents.
