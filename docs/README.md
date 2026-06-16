# SSH Poker Game - Documentation

This is the documentation hub. It is written to give you (and any third party)
a single, cohesive narrative of *what* this project is, *why* it is built the
way it is, and *how* to work on it. It is intended to be leaned on for future
changes and as a learning resource, so decisions are explained in plain terms
and tied back to real production concerns rather than just described.

## Start here (the story so far)

This project began as an all-Rust, SSH-accessible multiplayer Texas Hold'em
game. A review found that the core game engine was solid but the SSH/TUI layer
was effectively non-functional (it never rendered to clients, gameplay was not
wired in, and authentication was bypassed). Rather than keep hand-rolling the
hardest part (the SSH-to-terminal bridge), we are moving the front end to the
Go Charm.sh stack (Wish for SSH, Bubble Tea for the TUI) while keeping the
proven Rust game engine as a stateless rules service behind a gRPC boundary.

The full reasoning for every major fork in that story lives in the
[Architecture Decision Records](./adr/README.md). Read those in order and you
will understand the whole project.

## How the documentation is organised

| Document | What it answers |
| --- | --- |
| [Architecture Decision Records](./adr/README.md) | Why we made each major decision, the alternatives we rejected, and the trade-offs we accepted. The narrative backbone. |
| [Architecture overview](./architecture/overview.md) | What the system looks like today: components, responsibilities, data flow, and diagrams. |
| [Roadmap](./roadmap.md) | The milestone-by-milestone plan (M1-M12) tied to the GitHub Project board, with the production rationale for each phase. |
| [Development guide](./development.md) | How to set up the environment, regenerate code, build, test, and follow the branch/PR workflow. |
| [Spec-driven development](./practices/spec-driven-development.md) | How `poker.proto` is treated as the contract-first source of truth that both languages generate from. |
| [Test-driven development](./practices/test-driven-development.md) | The red-green-refactor loop and the cross-language test pyramid, grounded in the Phase 0 cautionary tale. |
| [References](./references.md) | Curated external reading - Charm, gRPC/protobuf, tonic, TDD, ADRs, poker hand evaluation, SSH security, Go concurrency, and Dev Containers. |

## Reading order for a newcomer

1. This page, for orientation.
2. [ADR-0001](./adr/0001-stabilize-the-rust-baseline.md) through
   [ADR-0009](./adr/0009-learning-sandbox-and-consolidation-path.md), in order -
   the decisions (ending with the learning-sandbox intent and exit path).
3. [Architecture overview](./architecture/overview.md) - the resulting system.
4. [Development guide](./development.md) - to start contributing.
5. [Spec-driven development](./practices/spec-driven-development.md) and
   [Test-driven development](./practices/test-driven-development.md) - the two
   working practices that shape day-to-day contributions.
6. [Roadmap](./roadmap.md) - to see where we are and what is next.
7. [References](./references.md) - to go deeper on any topic via verified
   external sources.

## A note on the older documents

Several files in this directory (for example `executive_summary.md`,
`simplified_architecture.md`, `current_state_summary.md`, and the
`book_mappings/` set) predate the migration described above. They are retained
for historical context but are **not** authoritative. Where they conflict with
the ADRs or the architecture overview, the ADRs win. The
[migration plan](../MIGRATION_PLAN.md) at the repository root is the long-form
analysis that led to [ADR-0002](./adr/0002-go-frontend-rust-engine.md).
