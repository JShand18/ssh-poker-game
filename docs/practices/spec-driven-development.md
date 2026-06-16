# Spec-driven development (contract-first), as practiced here

This page explains a working practice the project leans on, in concrete terms:
**the spec is a real artifact in the repo, and changes start by editing it.** It
is written to be learned from, so it explains not just the rule but why the rule
pays off.

## What "spec-driven" means in general

Spec-driven (or *contract-first*) development inverts a common habit. Instead of
writing code and letting the interface emerge from it, you **write the
interface - the spec - first**, agree on it, and then generate or implement both
sides from that single description. The spec is the source of truth; the code
conforms to the spec, not the other way around.

For an API boundary, the spec is an *interface definition*: the messages, the
calls, the types. Both the caller and the callee are derived from it, so they
cannot disagree about the shape of the boundary - the definition is shared by
construction.

## The spec in this project: `poker.proto`

Here, the spec is not a wiki page or a design doc. It is a file the build depends
on:

```
proto/poker/v1/poker.proto
```

This Protocol Buffers file **is the contract** between the Go gateway and the
Rust engine. It defines the messages (`Action`, `Card`, `SeatView`, `TableView`,
`GameStateBlob`, the request/response types) and the service - `EngineService`
with `NewHand`, `ApplyAction`, `GetValidActions`, and `CompleteHand`. Everything
the two halves know about each other, they know through this file.

Both sides **generate code from it**:

- **Rust** generates server stubs and message types via `tonic-build`, run from
  `engine-service/build.rs` during `cargo build`.
- **Go** generates client stubs and message types via `protoc-gen-go` /
  `protoc-gen-go-grpc`, run by `make proto-go` into `gateway/internal/pokerpb/`.

Neither side hand-writes the wire types. They are *projections of the spec*.

## The workflow: a change starts at the spec

The practice is a rule you can follow mechanically:

> To change the boundary, **edit `poker.proto` first**, then regenerate, then
> implement each side against the regenerated code.

Concretely (the same flow as the [development guide](../development.md)):

1. **Edit the spec.** Add the field, message, or RPC to `poker.proto`.
2. **Regenerate both sides.** `make proto` updates the Go stubs and rebuilds the
   Rust codegen. Now both languages have new, matching types - and any code that
   doesn't compile against them is immediately visible.
3. **Implement against the generated code.** Fill in the Rust handler and the Go
   caller. The compiler guides you: a missing field or a renamed method is a
   build error, not a runtime surprise.
4. **Commit the spec change and the regenerated Go together** (see drift,
   below), open the PR, let CI verify.

You never "add a field on the Go side and a matching one on the Rust side and
hope they line up." There is exactly one place the field is defined.

## Why this pays off in a deployed system

- **Single source of truth.** The boundary is described once. There is no second
  copy to keep in sync, so there is no opportunity for two hand-written copies to
  diverge. This is the same instinct as the opaque-state design in
  [ADR-0004](../adr/0004-grpc-contract-and-opaque-state.md): keep the things that
  must agree in one authoritative place.
- **Parallel front-end / back-end work.** Once the spec is agreed, the Go and
  Rust teams (or, here, the same person on different days) can work *at the same
  time* against a stable, generated interface. The back end can build
  `ApplyAction` while the front end builds the action bar that calls it, each
  trusting the generated types. The spec is the handshake that makes the
  parallelism safe.
- **Automatic drift detection.** Because the generated Go code is committed,
  CI can **regenerate and diff** it (`git diff --exit-code`). If someone edits
  the proto but forgets to regenerate - or hand-edits generated code - CI fails.
  This is the codegen-drift check (board issue **#27**), and it is the mechanical
  safety net that makes "the spec is the source of truth" *enforced* rather than
  merely intended. See [ADR-0007](../adr/0007-ci-cd-pipeline.md).
- **The contract stays small on purpose.** Spec-first only works if the spec is
  something humans can actually agree on. By keeping the authoritative state
  opaque (`GameStateBlob { bytes }`) and exposing only typed per-seat views, the
  proto surface stays tiny and stable - the spec changes when *behavior* changes,
  not every time the engine's internals are refactored
  ([ADR-0004](../adr/0004-grpc-contract-and-opaque-state.md)).

## A worked example

Suppose we want each seat to show a "time bank" (seconds left to act).

- **Spec-first (what we do):** add `uint64 time_bank_secs = 9;` to `SeatView` in
  `poker.proto`. Run `make proto`. Both languages now have the field. The Rust
  engine populates it when building each `TableView`; the Go view renders it. One
  definition, two generated projections, drift impossible to merge.
- **Code-first (what we avoid):** add a field to the Go struct, add a field to a
  Rust struct, and serialize them compatibly by hand. Now two hand-maintained
  definitions must agree forever, with nothing checking that they do. The first
  time they drift, a card or a number renders wrong in production and no test
  necessarily catches it.

## Relationship to the other practices and decisions

- It is the *front* of the workflow; [test-driven
  development](./test-driven-development.md) is how you build each side once the
  spec exists. Spec-first decides *what* the boundary is; test-first decides that
  *each side does the right thing* against it.
- It depends on the reproducible toolchain
  ([ADR-0003](../adr/0003-dev-container-environment.md)) so "regenerate" produces
  identical output everywhere, and on the monorepo
  ([ADR-0005](../adr/0005-monorepo-structure.md)) so the spec and both generated
  halves live - and change - together.

## Further reading

- [gRPC and Protocol Buffers references](../references.md#grpc-and-protocol-buffers-contract-first)
  - the official IDL/contract material this practice is built on.
- [ADR-0004: gRPC contract and opaque state](../adr/0004-grpc-contract-and-opaque-state.md)
  - the contract this spec defines.
- [Development guide](../development.md) - the exact commands to regenerate.
