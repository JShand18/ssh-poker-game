# ADR-0004: gRPC contract with opaque state and per-seat views

- Status: Accepted
- Date: 2026-06-16
- Related issues: M2 "Proto and structure" (board #2): Define
  `proto/poker/v1/poker.proto`; Wire codegen (tonic-build + protoc-gen-go)

## Context

[ADR-0002](./0002-go-frontend-rust-engine.md) places a network boundary between
the Go gateway and the Rust engine and adopts **Option A**: Go owns SSH sessions
and the authoritative table state; Rust is a stateless pure function over
`(state, action) -> (new state, views)`. That decision leaves two concrete
questions open, and this ADR answers them:

1. **What transport and contract format** connects the two halves?
2. **How does the authoritative `GameState` cross that boundary** without
   leaking the engine's internals into Go - or, worse, leaking one player's hole
   cards to another?

The Rust `poker-engine` already derives `serde::Serialize`/`Deserialize` on its
`GameState`. The Go side needs *nothing* from inside that state except a way to
hold it between turns and a way to render each player's view of the table.

## Decision

Define a small **gRPC contract** in `proto/poker/v1/poker.proto`, generated for
Rust by `tonic-build` (via `engine-service/build.rs`) and for Go by
`protoc-gen-go`/`protoc-gen-go-grpc` (via `make proto-go`). The contract has two
deliberate properties:

- **The authoritative state is opaque.** `GameStateBlob { bytes data = 1; }`
  carries the serde-encoded `poker-engine` `GameState`. Go stores and
  round-trips this blob without ever interpreting it; only the engine
  understands its contents.
- **Rust returns typed, per-seat views.** A `TableView` is the table rendered
  from one viewer's perspective (phase, community cards, pot, current/dealer
  seat, and `SeatView`s). Crucially, a `SeatView.hole_cards` field is populated
  **only for the viewing player's own seat**; every other seat carries public
  information only.

The service is exactly four RPCs, each taking the current state (or hand config)
and returning the next state plus rendered views:

```
service EngineService {
  rpc NewHand(NewHandRequest) returns (HandState);
  rpc ApplyAction(ApplyActionRequest) returns (ApplyActionResult);
  rpc GetValidActions(ValidActionsRequest) returns (ValidActions);
  rpc CompleteHand(CompleteHandRequest) returns (CompleteHandResult);
}
```

`HandState`, `ApplyActionResult`, and `CompleteHandResult` each bundle the new
`GameStateBlob` **and** a `repeated TableView views`, so the gateway can fan out
to every seated player without a second round-trip per viewer.

## Rationale (why this is necessary)

- **A typed contract is the only honest description of a network boundary.**
  Once the engine lives across a process boundary, the boundary *is* an API. A
  `.proto` file makes that API explicit, versioned, and code-generated on both
  sides, so the compiler - not a code review - catches a mismatch. gRPC over
  HTTP/2 with Protocol Buffers gives us that for free, in both Go and Rust, with
  mature tooling (tonic, prost, protoc). See
  [spec-driven development](../practices/spec-driven-development.md) for how this
  contract becomes the single source of truth.
- **Opaque state keeps the proto surface tiny and hides engine internals.** The
  engine's `GameState` is rich and will evolve (side pots, betting bookkeeping,
  deck state). If we mirrored all of that in the proto, every internal change
  would ripple into a proto change and a Go change. By shipping it as `bytes`,
  the contract stays small and stable: Go depends on *behavior* (apply an
  action, get views), not on the engine's data layout. The engine can be
  refactored freely as long as it can still decode its own blob.
- **Server-authoritative rendering is a security property, not just tidiness.**
  If Go received the full game state, a malicious or buggy client path could in
  principle observe data it should not - notably other players' hole cards.
  Because Go only ever holds an opaque blob and renders from per-seat
  `TableView`s built *inside the engine*, the gateway physically cannot leak a
  card it never receives in plaintext for that viewer. The trust boundary is
  drawn so that "who can see what" is decided by the authoritative engine, once,
  per seat. This dovetails with the broader
  [security posture](./0008-security-posture.md).
- **Bundling views with state removes a fan-out round-trip.** Multiplayer means
  one action must update many screens. Returning all seats' `TableView`s
  alongside the new blob lets the Go table manager push each player their view
  via in-process `program.Send` (the Option A fan-out) without calling the
  engine once per viewer.
- **Four RPCs match the actual lifecycle.** `NewHand` starts a hand,
  `ApplyAction` advances it, `GetValidActions` drives the UI's action bar, and
  `CompleteHand` resolves the showdown and reports `Winning`s. There is no fifth
  thing the gateway needs from the rules engine.

## Alternatives considered

- **Fully typed state in the proto (mirror `GameState`).** Maximum
  transparency, but it couples the contract to engine internals: every
  refactor of the engine becomes a breaking proto change, and Go gains the
  ability (and temptation) to reason about - and accidentally expose - private
  game data. Rejected.
- **REST/JSON over HTTP/1.1.** Familiar and debuggable, but no schema-enforced
  contract, no generated stubs on both sides, and weaker typing. We would
  reinvent half of what protobuf gives us. Rejected for the service boundary.
- **A different serialization for the blob (JSON instead of a compact binary
  format).** The choice of *what serde format* fills `GameStateBlob.data`
  (e.g. bincode vs JSON) is an engine-internal detail and remains open (noted in
  the migration plan). It does not affect the contract, which only sees `bytes`.
- **Streaming RPCs for live updates.** Considered and explicitly deferred: in
  Option A the SSH sessions and state both live in Go, so fan-out is in-process
  and needs no server-streaming channel. A future streaming upgrade (Option B)
  would revisit this.

## Consequences

- Positive: a tiny, stable contract; engine internals stay private; the design
  is server-authoritative so clients cannot see other players' hole cards;
  codegen on both sides catches drift at compile time; fan-out needs no extra
  round-trips.
- Negative: the full `GameState` is serialized and shipped on every action
  (negligible for turn-based poker with tiny payloads, but it is real bytes on
  the wire); Go cannot inspect the state for debugging without going through the
  engine; the opaque blob means the proto cannot, by itself, document the game's
  data model.
- Risk: the blob format becomes an implicit compatibility surface - a persisted
  or in-flight blob produced by one engine build must be decodable by another.
  As long as state is never persisted across versions (persistence is deferred),
  this risk stays small; it must be revisited if/when persistence lands.

## Learning notes

The pattern here is **"opaque token, typed projection."** The producer (Rust)
hands the consumer (Go) a token it cannot read, and separately a typed
projection of exactly the slice the consumer is allowed to act on. You see the
same shape in opaque pagination cursors, encrypted session tokens, and
capability handles: the boundary deliberately exposes *behavior and a safe
view*, never the raw internal state. It is also a clean illustration of letting
the *trust boundary* drive the data model - the reason hole cards are safe is
not a Go code review, it is that the bytes never cross the wire to a viewer who
should not have them.

## Further reading

- [gRPC and Protocol Buffers references](../references.md#grpc-and-protocol-buffers-contract-first)
  - the official gRPC and protobuf documentation and contract-first material.
- [Rust async and tonic references](../references.md#rust-async-and-tonic) -
  tonic and prost, which generate and serve this contract on the Rust side.
- [Spec-driven development](../practices/spec-driven-development.md) - how the
  `.proto` file is treated as the authoritative spec for both languages.
- [Architecture overview](../architecture/overview.md) - where this contract
  sits in the end-to-end data flow.
