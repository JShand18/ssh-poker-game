# Roadmap

This is the milestone-by-milestone plan for the migration, tied to **GitHub
Project board #2**. The board has twelve milestones (M1-M12) and issues #2-#41;
this page tells the story they encode - what each phase delivers, *why it matters
in a deployed system*, and how you know it is done.

Two earlier docs set the frame: the
[Architecture Decision Records](./adr/README.md) explain the reasoning behind
these phases, and the [architecture overview](./architecture/overview.md)
describes the system they build toward. A note on status: **M1 and M2 are
materially complete** (Dev Container verified; proto, codegen wiring, repo
structure, and scaffolds in place), the Rust engine is stabilized and green
(Phase 0, [ADR-0001](./adr/0001-stabilize-the-rust-baseline.md)), and M3 onward
is the work ahead.

## How the phases fit together

M1-M8 march from "an environment exists" to "two humans play a hand over SSH and
it is deployable." M9-M12 are the quality and hardening milestones that make it
trustworthy in production: testing, CI/CD, integration testing, and security.
They are numbered later but are not afterthoughts - several (CI gates, the
codegen-drift check, SSH hardening) must come online as soon as the code they
guard exists.

Execution order: **M1 -> board population -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 ->
M8**, with M3 (Rust service) and a stub of M4 (Go skeleton) briefly overlapping
once the proto exists. The quality milestones (M9-M12) layer on alongside.

The workflow for every issue is the same: **one issue = one branch = one PR**,
branch named `m<phase>/<slug>`, the PR says `Closes #N` and must pass CI. See the
[development guide](./development.md) for the mechanics.

For this same plan as a concrete week-by-week schedule (S1-S21, sized at 3-5
hours per sprint and sequenced to reach a playable game first), see the
[sprint plan](./sprint-plan.md).

---

## M1 - Dev environment

**Deliver:** a reproducible Dev Container (`.devcontainer/`) providing Go, Rust
1.82.0, `protoc`, `protoc-gen-go`, `protoc-gen-go-grpc`, and `buf`.

**Why it matters in production:** a polyglot project lives or dies on a
reproducible toolchain. If "it builds" depends on one person's laptop, every
later phase inherits that fragility, and CI can never mean the same thing as
local. Pinning the environment first eliminates an entire class of
version-mismatch bugs before any are written. See
[ADR-0003](./adr/0003-dev-container-environment.md).

**Definition of done:** the container opens; `go`, `cargo`, `protoc`, and the
plugins all run; a sample codegen succeeds for both languages. *(Verified: Go
codegen + `go build` pass inside the container.)*

---

## M2 - Proto and structure

**Deliver:** the repo restructure (`proto/`, `gateway/`, `crates/engine-service`)
and the shared contract `proto/poker/v1/poker.proto` (Action, Card, SeatView,
TableView, GameStateBlob, and the `EngineService` RPCs), with codegen wired for
both languages (`tonic-build` for Rust, `protoc-gen-go`/`buf` for Go). Includes
the cross-cutting **"Add Go CI job"** item.

**Why it matters in production:** the contract is the boundary between the two
halves. Defining it - and generating both sides from one source - is what lets
front-end and back-end work proceed in parallel without drifting apart. The shape
of the contract (opaque state, per-seat views) is itself a security and
maintainability decision. See
[ADR-0004](./adr/0004-grpc-contract-and-opaque-state.md),
[ADR-0005](./adr/0005-monorepo-structure.md), and
[spec-driven development](./practices/spec-driven-development.md).

**Definition of done:** `proto/poker/v1/poker.proto` compiles to both Rust and
Go; an empty build passes in both halves. *(Scaffolds in place: `engine-service`
and `poker-gateway` build and exercise codegen.)*

---

## M3 - Rust engine-service

**Deliver:** the tonic gRPC server wrapping `poker-engine`: a serde blob
round-trip, then `NewHand`, `ApplyAction`, `GetValidActions`, and `CompleteHand`,
with domain<->proto mapping and per-seat `TableView` construction, plus service
unit tests. (Issues span scaffolding, the four RPCs, the mapping, and tests.)

**Why it matters in production:** this is where the trustworthy engine becomes a
trustworthy *service*. Getting the blob round-trip and per-seat view construction
right - especially "only the viewer sees their own hole cards" - is the core
correctness and security work of the back end. The engine stays stateless, which
keeps it easy to test and safe to call concurrently.

**Definition of done:** the service serves all four RPCs and round-trips a full
hand (deal -> betting -> showdown -> payout) in tests.

---

## M4 - Go gateway skeleton

**Deliver:** the Wish SSH server with Bubble Tea middleware (connect shows a
minimal TUI) and a gRPC client wrapper, proven by a keypress -> Rust call ->
render smoke test.

**Why it matters in production:** this validates the single riskiest assumption
of the whole architecture - that an SSH keystroke can drive a real engine RPC and
render the result - on a tiny surface, before any game UI exists. Proving the
plumbing end to end early means later phases build on something known to work.

**Definition of done:** SSH into the gateway shows a TUI, and a keypress triggers
a real engine RPC whose result is rendered.

---

## M5 - TUI views

**Deliver:** the Bubble Tea views with Lip Gloss styling - a name prompt (guest
identity), a lobby (list/create/join tables), and the game view (poker table:
community cards, pot, seats, action bar), porting the old casino theme
conceptually.

**Why it matters in production:** the views are the product. This is where the
project stops being plumbing and becomes a game someone wants to look at. Doing it
in Bubble Tea + Lip Gloss is the entire point of
[ADR-0002](./adr/0002-go-frontend-rust-engine.md) - the SSH/TUI bridge we kept
failing to build is now handled by Wish, so effort goes into the UI itself.

**Definition of done:** lobby and game views render with Lip Gloss styling and
you can navigate between them.

---

## M6 - Multiplayer

**Deliver:** the Go table manager - one authoritative `GameStateBlob` per table
plus seated players - with turn enforcement via the engine and fan-out to seated
sessions via `tea.Program.Send`, including disconnect/leave handling.

**Why it matters in production:** this is the trickiest Go piece and the heart of
"multiplayer." Owning state in Go and fanning out in-process is precisely why
Option A was chosen; this phase proves that bet. Disconnect handling is not
optional polish - players *will* drop, and a leaked seat or a wedged table is a
real outage.

**Definition of done:** two SSH sessions seated at one table see each other's
actions in real time.

---

## M7 - Playable

**Deliver:** the milestone validation - two humans complete a full hand over SSH
(deal -> betting rounds -> showdown -> payout), exercised as an end-to-end test.

**Why it matters in production:** this is the definition of "usable." Everything
before it is means to this end. It is also the first moment the whole stack - SSH,
TUI, gRPC, engine, fan-out - is proven working together against real human input.

**Definition of done:** two humans play a complete hand, start to finish, over
SSH.

---

## M8 - Deploy

**Deliver:** a persistent Wish host key, config flags (ports, engine address),
Dockerfile + docker-compose for both services, a Makefile, and run docs;
deployable to a host (e.g. DigitalOcean).

**Why it matters in production:** a game nobody can reach is not a product. A
*persistent* host key matters specifically: without it, every restart changes the
server's identity and trains users to click through host-key warnings - a real
security anti-pattern. Compose makes "bring up gateway + engine together at
compatible versions" a single command. See
[ADR-0008](./adr/0008-security-posture.md) for the host-key reasoning.

**Definition of done:** `docker compose up` starts gateway + engine, and the
documented deploy works on a fresh host.

---

## M9 - Unit testing

**Deliver:** expanded `engine-service` unit tests across the RPC surface;
`poker-engine` edge-case tests (side pots, all-in run-out, heads-up, split pots);
Go gateway unit tests (table manager, view mapping, input parsing); and code
coverage reporting (`cargo-llvm-cov`, `go test -cover`).

**Why it matters in production:** the wide base of the test pyramid. Most logic
lives in the engine, so most tests should be cheap, fast engine tests. This is the
direct answer to [ADR-0001](./adr/0001-stabilize-the-rust-baseline.md)'s
cautionary tale - a suite that is actually run and actually green. See
[ADR-0006](./adr/0006-testing-strategy.md) and
[test-driven development](./practices/test-driven-development.md).

**Definition of done:** each RPC handler and the tricky engine paths are
unit-tested; the Go gateway's core logic is unit-tested; coverage is reported for
both languages.

---

## M10 - CI/CD

**Deliver:** a unified pipeline building and testing both the Rust workspace and
the Go gateway (with caching), running `clippy -D warnings` and `go vet`; a
**codegen-drift check** (#27: regenerate and fail on `git diff`); release image
publishing for both services on tag; and optional CD to a host (stretch).

**Why it matters in production:** CI is executable policy - the gate that makes
"no warnings," "tests pass," and "generated code is fresh" enforceable instead of
aspirational. Job timeouts (carried over from stabilization) turn a future hang
into a fast failure. The drift check catches the uniquely sneaky bug where
generated code silently disagrees across the boundary. See
[ADR-0007](./adr/0007-ci-cd-pipeline.md).

**Definition of done:** both halves build/test in CI with caching; the drift
check fails on stale generated code; tagged releases publish images for gateway
and engine.

---

## M11 - Integration testing

**Deliver:** a gRPC integration test driving a full hand against a running
`engine-service`; an SSH end-to-end harness (scripted client asserting connect ->
name -> lobby -> table); a multiplayer integration test (two sessions at one
table); and a docker-compose environment for integration runs.

**Why it matters in production:** unit tests are blind to the seams the migration
introduced - the gRPC and SSH boundaries. These are the tests that prove the two
independently-tested halves actually agree, and that real multiplayer works over
real SSH. They are few and slower by design (the narrow top of the pyramid). See
[ADR-0006](./adr/0006-testing-strategy.md).

**Definition of done:** a full hand passes through the live gRPC API in a test;
the SSH harness drives the TUI flow; two concurrent sessions see each other's
actions in an automated run.

---

## M12 - Security

**Deliver:** SSH hardening (idle timeouts, max sessions, per-IP connection
limits); connection/action rate limiting; dependency scanning (`cargo-audit`,
`govulncheck`, Dependabot); engine action-processing fuzzing (`cargo-fuzz`); a
*design* for re-enabling authentication securely (Argon2 + lockout); securing the
gRPC boundary (localhost by default, mTLS if exposed) and secrets handling; and a
written threat model / security review checklist.

**Why it matters in production:** the day this is deployed it is a public SSH
listener, which means constant automated abuse. Hardening, rate limiting, keeping
the engine internal, and fuzzing untrusted input are table stakes. Authentication
is *deferred but designed* - and the system is documented honestly as guest-only
until it lands, so the docs never claim a protection the code lacks. See
[ADR-0008](./adr/0008-security-posture.md).

**Definition of done:** the SSH front door is hardened and rate-limited; the
engine is bound internally; dependency scanning and a fuzz target run in CI; a
threat model and the secure-auth design are written down.

---

## Retired and deferred

- **Retire after parity:** `crates/ssh-poker-server` (russh) and
  `crates/poker-tui` (ratatui) - the broken SSH/TUI layer the migration replaces
  ([ADR-0005](./adr/0005-monorepo-structure.md)).
- **Deferred (left in place, untouched):** authentication, AI bots
  (`ai-bot`), persistence (`data-store`), metrics (`hybrid-metrics`), and the
  streaming upgrade to Option B.

## Related reading

- [Architecture Decision Records](./adr/README.md) - the reasoning behind the
  phases.
- [Architecture overview](./architecture/overview.md) - the system being built.
- [Development guide](./development.md) - the branch/PR workflow used per issue.
