# ADR-0005: Keep one monorepo with a shared proto contract

- Status: Accepted
- Date: 2026-06-16
- Related issues: M2 "Proto and structure" (board #2): Restructure repo (add
  `proto/`, `gateway/`, `crates/engine-service`)

## Context

[ADR-0002](./0002-go-frontend-rust-engine.md) splits the system into two halves
in two languages, and [ADR-0004](./0004-grpc-contract-and-opaque-state.md)
defines a gRPC contract that both halves generate code from. That raises an
organizational question: **do the Go gateway, the Rust engine, and the shared
contract live in one repository or several?**

The repository already contains a Rust **workspace** with several crates
(`poker-engine`, `poker-tui`, `ssh-poker-server`, `data-store`, `ai-bot`,
`hybrid-metrics`). The migration adds a new Rust crate (`engine-service`) and an
entirely new **Go module** (`gateway/`), plus a top-level `proto/` directory
that is the single source of truth both sides build from. Two of the existing
crates - `ssh-poker-server` (russh) and `poker-tui` (ratatui) - are the broken
SSH/TUI layer that the migration exists to replace.

## Decision

Keep **one monorepo** with a clear internal layout:

```
ssh-poker-game/
  proto/poker/v1/poker.proto   # the shared gRPC contract (source of truth)
  crates/                      # Rust workspace
    poker-engine/              #   KEEP: the rules engine (the asset)
    engine-service/            #   NEW: tonic gRPC server wrapping poker-engine
    data-store/ ai-bot/ hybrid-metrics/   # deferred, left in place untouched
    ssh-poker-server/ poker-tui/          # RETIRE after Go reaches parity
  gateway/                     # NEW Go module (Wish + Bubble Tea + gRPC client)
    cmd/poker-gateway/main.go
    internal/pokerpb/          #   generated Go code from proto
  Makefile                     # codegen + build for both halves
  .devcontainer/               # the one toolchain (ADR-0003)
```

The Rust side stays a Cargo workspace; the Go side is a self-contained module
under `gateway/`. The contract lives once, at `proto/`, and **both** the
`tonic-build` step (Rust) and the `protoc-gen-go` step (Go) read from it.

The two broken crates, `crates/ssh-poker-server` and `crates/poker-tui`, are
explicitly **scheduled for retirement** - archived then removed - once the Go
front end reaches parity. They remain in the workspace until then so the repo
keeps building and so their behavior is available for reference during the port.
The deferred crates (`data-store`, `ai-bot`, `hybrid-metrics`) are left in place,
untouched.

## Rationale (why this is necessary)

- **A shared contract wants a shared repository.** The whole point of
  [ADR-0004](./0004-grpc-contract-and-opaque-state.md) is that one `.proto` file
  drives both languages. In a monorepo, a change to that file and the matching
  regeneration on both sides land in **one atomic commit and one pull request** -
  the change and both generated halves are reviewed and merged together. Split
  repositories would force a multi-repo dance (publish the proto, bump a
  dependency, regenerate, coordinate releases) for what is logically one change,
  and would make the codegen-drift CI check (see
  [ADR-0007](./0007-ci-cd-pipeline.md)) far harder to express.
- **One environment, one build entry point.** The Dev Container
  ([ADR-0003](./0003-dev-container-environment.md)) provisions Go, Rust, and
  `protoc` together; a single top-level `Makefile` (`make proto`, `make build`,
  `make test`) drives both halves. That coherence only makes sense if both halves
  live together.
- **Atomic cross-cutting changes.** Many migration steps touch both sides at
  once (add an RPC, add its Go caller, add its Rust handler). A monorepo lets one
  PR keep the system internally consistent at every commit, which is exactly the
  "one issue = one branch = one PR" workflow the roadmap relies on.
- **The asset and its wrapper stay adjacent.** `engine-service` is a thin tonic
  shell around `poker-engine`; keeping them in the same Cargo workspace means
  `engine-service` depends on `poker-engine` by path, with no publishing step,
  and they are tested and linted together.
- **Retirement needs to be visible and reversible.** Marking
  `ssh-poker-server`/`poker-tui` for retirement *inside* the repo - rather than
  deleting them immediately - keeps the build green, preserves the old behavior
  for reference during the port, and makes the eventual removal a clean,
  reviewable deletion once parity is proven.

## Alternatives considered

- **Polyrepo (separate Go, Rust, and proto repositories).** The "clean
  microservice" instinct, and appropriate when teams and release cadences
  diverge. Here it is pure overhead: a solo/small project, one logical product,
  and a contract that changes in lockstep with both consumers. It would turn
  every contract change into cross-repo version coordination. Rejected.
- **A shared proto repository consumed by the other two.** A middle ground that
  still imposes publish/version/bump cycles on the contract and breaks atomic
  changes. Rejected for the same reason.
- **Delete the broken crates immediately.** Tempting, but it would remove the
  reference implementation mid-migration and risk breaking the workspace build
  before the replacement exists. Scheduling retirement after parity is safer.
- **Fold the Go module into the Cargo workspace tooling.** Not meaningful - Cargo
  and Go modules are independent toolchains. They coexist as siblings, unified
  only by the `Makefile` and the Dev Container, which is the right level of
  coupling.

## Consequences

- Positive: contract changes are atomic across both languages; one environment
  and one `Makefile` build everything; the engine and its service wrapper stay
  adjacent; cross-cutting migration steps land coherently; retirement is staged
  and reversible.
- Negative: the repo houses two toolchains and two dependency systems (Cargo and
  Go modules), which can confuse newcomers; a naive CI must build both halves on
  every change unless paths are scoped; generated Go code is committed to the
  repo (a deliberate trade-off that enables the drift check).
- Risk: a monorepo can accrete dead and deferred code (it already carries
  several deferred crates). Mitigation: explicitly label deferred vs. retiring
  crates, and actually remove `ssh-poker-server`/`poker-tui` once parity lands,
  rather than letting them linger indefinitely.

## Learning notes

The monorepo-vs-polyrepo choice is usually decided by **how tightly things
change together**, not by how many languages or services are involved. When two
components share a contract that evolves in lockstep, co-locating them makes the
contract a first-class, atomically-versioned artifact and removes a whole class
of cross-repo coordination bugs. The same reasoning that put the *environment*
in version control ([ADR-0003](./0003-dev-container-environment.md)) puts the
*contract and its consumers* in one repository: keep the things that must agree
with each other in a place where they cannot silently disagree.

## Further reading

- [gRPC and Protocol Buffers references](../references.md#grpc-and-protocol-buffers-contract-first)
  - on treating the `.proto` as a shared, versioned contract.
- [Architecture Decision Records references](../references.md#architecture-decision-records)
  - on keeping decisions and the code they govern together.
- [Architecture overview](../architecture/overview.md) - the resulting
  component layout.
- [Development guide](../development.md) - the one-issue/one-branch/one-PR
  workflow this structure enables.
