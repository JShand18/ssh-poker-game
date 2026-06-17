# ADR-0002: Go (Wish + Bubble Tea) front end with a Rust rules engine

- Status: Accepted
- Date: 2026-06-16
- Supersedes the all-Rust SSH/TUI approach
- Long-form analysis: [MIGRATION_PLAN.md](../../MIGRATION_PLAN.md)

## Context

The original architecture was all-Rust: `russh` for SSH and `ratatui` for the
TUI, bridged by a hand-written layer. That bridge was the project's recurring
failure point - in the reviewed state it never actually streamed rendered frames
to the client, so the "casino TUI over SSH" did not function. Building a correct
SSH-to-terminal bridge (PTY handling, resize, per-session isolation, ANSI
rendering) is genuinely hard, and we kept paying that cost.

Meanwhile, the Go Charm.sh ecosystem solves exactly this problem as a product:

- **Wish** is an SSH server whose middleware turns each SSH session into a
  program. Its `bubbletea` middleware wires an SSH session directly to a
  **Bubble Tea** TUI - the bridge we kept failing to build is provided for free
  and is battle-tested (it powers tools like Soft Serve).

The maintainer wants to keep using and learning Rust. The `poker-engine` is also
the asset most worth preserving (see [ADR-0001](./0001-stabilize-the-rust-baseline.md)).
So the real question was not "Go or Rust" but "where is the boundary between
them, and how do they communicate?"

We evaluated three shapes (full detail in the migration plan):

- **Option A - Rust as a stateless rules library.** Go owns SSH sessions and the
  authoritative table state; Rust is a pure function over gRPC: given a state and
  an action, return the next state plus rendered views.
- **Option B - Rust as the authoritative game server.** Rust owns table state and
  streams updates; Go is a thin SSH/TUI gateway.
- **Option C - cgo/FFI.** One Go binary with Rust linked in-process.

## Decision

Adopt the hybrid: **Go (Wish + Bubble Tea) front end, Rust `poker-engine` behind
a gRPC boundary, using Option A** (Rust as a stateless rules library; Go owns
sessions and authoritative table state).

Authentication, AI bots, persistence, and metrics are explicitly deferred so the
first milestone is a playable multiplayer game.

## Rationale (why this is necessary, and why Option A)

- **Delete the part that never worked.** Wish's middleware removes the custom
  SSH/TUI bridge entirely. We stop maintaining the exact code that has failed
  repeatedly and adopt a maintained, widely used implementation.
- **Keep the asset, keep learning Rust.** The rules engine - the hard, valuable,
  well-tested code - stays in Rust. A gRPC service is also a genuinely useful
  Rust learning surface (tonic, prost, type mapping).
- **Option A makes multiplayer simplest.** The thing that must push updates to
  players is the SSH session, and sessions always live in Go (Wish owns the
  connection). If Go also owns the authoritative state, then when one player
  acts, Go updates state and fans out to the others over in-process channels -
  no streaming RPC, no cross-process state synchronization. Option B would split
  "who owns state" (Rust) from "who owns connections" (Go), forcing a streaming
  or polling channel between them. For turn-based poker, Option A is less moving
  parts for the same result.
- **Performance is explicitly not the reason.** Poker is turn-based with tiny
  payloads; Rust is not here for speed. It is here because the engine exists, is
  trustworthy, and the maintainer values Rust. Naming this prevents us from
  over-engineering a boundary in pursuit of a non-existent performance need.

## Alternatives considered

- **Full Go rewrite (drop Rust).** Simplest single codebase, but discards the
  most valuable, most-tested component and the maintainer's learning goal.
- **Option B (Rust authoritative server).** A "truer" backend and richer Rust,
  but streaming/resync complexity for no functional gain in a turn-based game.
- **Option C (cgo/FFI).** One binary, no network hop, but fiddly cross-language
  builds, an unsafe boundary, and painful cross-compilation. Worst effort-to-
  benefit ratio for this project.

## Consequences

- Positive: the broken bridge is gone; multiplayer fan-out is simple; the engine
  is reused; two clean, independently testable halves.
- Negative: two languages and two processes to build, run, and deploy; a proto
  contract to maintain; full `GameState` is serialized across the boundary each
  action (negligible for poker, but verbose - addressed in
  [ADR-0004](./0004-grpc-contract-and-opaque-state.md)).
- Risk: contributors now need both toolchains - mitigated by the Dev Container
  ([ADR-0003](./0003-dev-container-environment.md)).

## Learning notes

The core lesson is **buy the commodity, build the differentiator.** The SSH/TUI
plumbing is a solved, commodity problem - adopt a mature implementation (Wish)
instead of rebuilding it. The game rules are the project's actual substance -
own them. Also note how the *boundary placement* (who holds state) drove the
entire design: in distributed systems, "where does the source of truth live?"
is usually the first and most consequential question.

One motivation is deliberately understated above: the Go + Rust split is *also*
chosen as an architecture-learning vehicle, not only for engine reuse. That
intent - and the explicit path to consolidating back to a single language if the
goals change - is recorded in
[ADR-0009](./0009-learning-sandbox-and-consolidation-path.md).
