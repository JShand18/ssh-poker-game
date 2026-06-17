# ADR-0001: Stabilize the Rust baseline before migrating

- Status: Accepted
- Date: 2026-06-16
- Related issues: the Phase 0 stabilization work (pre-board)

## Context

When work resumed on this project after a long pause, a full review of the
existing all-Rust codebase found a dangerous gap between what the documentation
claimed ("production-ready") and reality:

- The core `poker-engine` had an **infinite loop**: when every remaining player
  was all-in, `skip_to_next_active_player` spun forever. A single common poker
  situation could hang a thread permanently.
- `cargo test` did not even compile (a test referenced a private field), and one
  engine test hung. Three more tests were silently failing because the hang hid
  them. In other words, **the test suite had never passed**.
- The advertised CI (`clippy -D warnings`, `cargo fmt --check`, `cargo test`)
  could not have been green.
- Database tests used a multi-connection in-memory SQLite pool, which is a known
  flake (each connection is a separate database).

The team's intent was to migrate the front end to Go (see
[ADR-0002](./0002-go-frontend-rust-engine.md)). The Rust `poker-engine` is the
one asset worth carrying forward, because it encodes the actual game rules.

## Decision

Before any migration work, bring the Rust workspace to a **trustworthy, green
baseline**: fix the correctness bugs, make the entire test suite compile and
pass without hanging, make the linters clean, and fix repository hygiene.

## Rationale (why this is necessary)

- **You cannot port or trust logic you cannot run.** The migration plan keeps
  the Rust engine and wraps it in a service. If the engine can hang or its tests
  never ran, every later phase inherits that risk. Stabilizing first turns the
  engine into a dependable foundation.
- **A hang is a production incident, not a bug.** In a server that holds many
  concurrent games, one all-in hand looping forever ties up a worker and can
  cascade. This had to be fixed regardless of language direction.
- **Green CI is the safety net for everything that follows.** The migration adds
  a second language and a network boundary; we need automated checks we can
  believe in before we start changing the architecture under them.

## What changed

- Bounded the seat-advance scan and added "run the board out to showdown when no
  one can act," so all-in situations resolve instead of hanging.
- Fixed heads-up blind assignment (dealer posts the small blind) and made the
  action validator reject actions from players who cannot act.
- Fixed the test-suite compile error, a wrong assertion, a tautological test,
  and the in-memory SQLite pool configuration.
- Cleaned all `clippy` lints; made `clippy -D warnings` and `cargo test` real CI
  gates with a job timeout (so a future hang fails fast instead of running
  forever); made formatting advisory for now.
- Removed committed database/log binaries and ignored them going forward.

Result: 105 tests passing, `clippy -D warnings` clean.

## Alternatives considered

- **Migrate first, fix later.** Rejected: we would be building on quicksand and
  could not tell whether new bugs came from the migration or pre-existing rot.
- **Rewrite the engine from scratch in Go.** Rejected here (see ADR-0002); the
  engine is the most valuable, most-tested part and is worth preserving.

## Consequences

- Positive: a dependable engine and a CI we can trust; faster, safer iteration.
- Negative: a short delay before visible migration progress.
- Risk retained: the engine's dual phase-advance mechanism is still a little
  fragile; noted for future cleanup but out of scope for stabilization.

## Learning notes

The lesson here is a classic one: **trust must be established before it can be
built upon.** "Production-ready" is a claim that only means something when an
automated suite proves it on every change. A green pipeline is not bureaucracy;
it is the contract that lets you change code quickly without fear.
