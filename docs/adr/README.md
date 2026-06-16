# Architecture Decision Records (ADRs)

An Architecture Decision Record captures a single significant decision: the
context that forced the decision, the decision itself, the alternatives we
weighed, and the consequences we accepted. ADRs are a widely used production
practice because they answer the question every future maintainer eventually
asks: *"why is it like this?"* - without requiring an archaeology dig through
chat logs and commit history.

We use them here for two reasons that map directly to this project's goals:

- **Maintainability.** When you return to this code in six months, or a third
  party picks it up, the reasoning is preserved next to the code it governs.
- **Learning.** Each record explains the trade-off in real-world terms, so the
  repository doubles as a study of how production systems are reasoned about.

## Conventions

- ADRs are numbered sequentially and never renumbered.
- An ADR is immutable once `Accepted`. If we change our minds, we add a new ADR
  that supersedes the old one (and mark the old one `Superseded by ADR-XXXX`).
- Status values: `Proposed`, `Accepted`, `Superseded`, `Deprecated`.
- Each ADR follows the same shape: Context, Decision, Rationale, Alternatives,
  Consequences, Learning notes.

## Index

| ADR | Title | Status |
| --- | --- | --- |
| [0001](./0001-stabilize-the-rust-baseline.md) | Stabilize the Rust baseline before migrating | Accepted |
| [0002](./0002-go-frontend-rust-engine.md) | Go (Wish + Bubble Tea) front end with a Rust rules engine | Accepted |
| [0003](./0003-dev-container-environment.md) | A Dev Container is the canonical environment | Accepted |
| [0004](./0004-grpc-contract-and-opaque-state.md) | gRPC contract with opaque state and per-seat views | Accepted |
| [0005](./0005-monorepo-structure.md) | Keep one monorepo with a shared proto contract | Accepted |
| [0006](./0006-testing-strategy.md) | Layered testing strategy across two languages | Accepted |
| [0007](./0007-ci-cd-pipeline.md) | CI/CD gates, codegen drift, and releases | Accepted |
| [0008](./0008-security-posture.md) | Security posture for a public SSH service | Accepted |
| [0009](./0009-learning-sandbox-and-consolidation-path.md) | A learning sandbox, with a consolidation exit path | Accepted |
