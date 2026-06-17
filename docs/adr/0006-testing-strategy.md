# ADR-0006: Layered testing strategy across two languages

- Status: Accepted
- Date: 2026-06-16
- Related issues: M9 "Unit testing", M11 "Integration testing" (board #2);
  drift check tracked under M10 "CI/CD"

## Context

[ADR-0001](./0001-stabilize-the-rust-baseline.md) is the cautionary tale that
motivates this one: the original all-Rust project advertised itself as
"production-ready," yet `cargo test` did not compile, one engine test hung
forever, and three more tests were silently failing behind the hang. **The test
suite had never passed.** A claim of quality with no green suite behind it is
just a claim.

The migration makes the testing problem strictly harder. We now have:

- A **Rust** rules engine (`poker-engine`) and a thin **Rust** gRPC service
  wrapping it (`engine-service`).
- A **Go** gateway (Wish + Bubble Tea + a gRPC client + a table manager).
- A **gRPC** boundary between them, with code generated on both sides from a
  shared `.proto` ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)).
- An **SSH** boundary in front of the gateway, where real multiplayer behavior
  actually happens.

Bugs can hide at every one of those layers, and a single test layer cannot catch
all of them: a perfect engine unit test says nothing about whether Go decodes a
`TableView` correctly, and a passing Go unit test says nothing about whether the
two halves agree on the wire.

## Decision

Adopt a **layered test pyramid that spans both languages**, with each layer
owning what only it can verify:

1. **Rust unit / engine tests (the wide base).** Pure-function tests of
   `poker-engine` transitions and `engine-service` handlers: hand setup, betting
   rounds, side pots, all-in run-out to showdown, heads-up blinds, split pots,
   invalid-action rejection, and `GameStateBlob` serde round-trips. Fast,
   deterministic, no network. (M9)
2. **Go unit tests.** The table manager (seat/leave/turn enforcement), the
   proto-to-view mapping, and key/action parsing in the Bubble Tea models. Fast,
   no network, no SSH. (M9)
3. **gRPC integration tests.** Stand up a real `engine-service` and drive a full
   hand through the four RPCs, asserting state transitions, per-seat `TableView`
   correctness, and payouts. This is the layer that proves the two
   independently-tested halves actually agree across the boundary. (M11)
4. **SSH end-to-end tests (the narrow top).** A scripted SSH client against the
   running gateway asserting real flows (connect -> name -> lobby -> table), and
   the headline multiplayer case: **two concurrent sessions seated at one table
   see each other's actions in real time** (M7's definition of "playable",
   exercised as an automated test in M11).

Two cross-cutting gates support the pyramid:

- **Codegen-drift check.** CI regenerates the Go and Rust code from
  `proto/poker/v1/poker.proto` and fails if the committed generated code differs
  (`git diff --exit-code`). Stale generated code is a correctness bug the
  ordinary tests cannot see. (M10; see
  [ADR-0007](./0007-ci-cd-pipeline.md).)
- **Coverage reporting.** `cargo-llvm-cov` for Rust and `go test -cover` for Go,
  surfaced as a combined signal - used to find untested paths, not as a vanity
  percentage. (M9)

## Rationale (why this is necessary)

- **Each boundary needs a test that crosses it.** The migration's defining
  feature is two new boundaries (gRPC and SSH). Unit tests on either side are
  necessary but structurally blind to boundary bugs - a mismatched enum mapping,
  a view assembled for the wrong seat, a fan-out that never fires. Only
  integration and end-to-end layers exercise those, so they must exist even
  though they are slower and fewer.
- **The pyramid shape keeps the suite fast and trustworthy.** Most logic lives
  in the engine, so most tests should be cheap engine tests; a handful of
  integration and e2e tests cover the seams. Inverting that (an "ice-cream cone"
  of mostly slow end-to-end tests) yields a suite too slow to run on every
  change and too flaky to believe - the opposite of the safety net
  [ADR-0001](./0001-stabilize-the-rust-baseline.md) demands.
- **Codegen drift is a uniquely sneaky failure.** Because both sides build from
  generated code, a contract change that is not regenerated produces code that
  compiles and passes unit tests while silently disagreeing on the wire. A drift
  gate is the only thing that catches it mechanically, which is why it is part of
  the testing strategy and not just a CI nicety.
- **The all-in hang is the proof.** A bounded, automated suite with a job
  timeout would have caught the original infinite loop on the first run instead
  of letting it mask three other failures. The layered strategy plus the CI
  timeout ([ADR-0007](./0007-ci-cd-pipeline.md)) is the direct, concrete
  response to that incident.

## Alternatives considered

- **Unit tests only.** Fast and cheap, but blind to exactly the new failure
  modes the migration introduces (the gRPC and SSH seams). Rejected as the whole
  strategy; kept as the base layer.
- **End-to-end tests only.** They feel reassuring ("it really plays!") but are
  slow, flaky, and terrible at localizing a failure - a red e2e run tells you
  *something* broke, not *what*. Rejected as the whole strategy; kept as a thin,
  high-value top layer.
- **Skip the drift check, trust discipline.** Relies on every contributor
  remembering to regenerate. The history in
  [ADR-0001](./0001-stabilize-the-rust-baseline.md) is a direct argument against
  trusting discipline over automation. Rejected.
- **Property-based / fuzz testing everywhere now.** Valuable - and a fuzz target
  for engine action processing is planned under
  [security](./0008-security-posture.md) (M12) - but as a complement to the
  pyramid, not a replacement. Not part of the baseline strategy.

## Consequences

- Positive: each layer catches the class of bug only it can; the suite stays
  fast where it is wide and thorough where it is deep; drift is caught
  mechanically; the project earns the right to call itself tested.
- Negative: maintaining tests in two languages plus integration harnesses is
  more work than a single-language suite; the e2e/SSH layer needs scaffolding
  (scripted clients, a way to run both services together).
- Risk: the slow top layers tempt teams to skip them under time pressure, which
  is exactly how the original suite rotted. Mitigation: wire every layer into CI
  (M10) so "skipping" is a visible, deliberate act rather than quiet neglect.

## Learning notes

Testing is **risk-driven, not ritual.** You do not write tests to hit a number;
you write them where a failure would hurt and where nothing else would catch it.
The migration moved the risk to the boundaries, so the test strategy follows the
risk to the boundaries. And the meta-lesson from
[ADR-0001](./0001-stabilize-the-rust-baseline.md) stands above all of it: a test
suite is only a safety net if it is **run automatically and proven green** - an
unrun test is documentation at best and a false sense of security at worst.

## Further reading

- [Test-Driven Development references](../references.md#test-driven-development)
  - the test pyramid (Martin Fowler / Mike Cohn) and TDD foundations.
- [Test-driven development, as practiced here](../practices/test-driven-development.md)
  - the red-green-refactor workflow that feeds this pyramid.
- [ADR-0007: CI/CD pipeline](./0007-ci-cd-pipeline.md) - the gates that run
  these tests on every change, including the drift check and job timeouts.
- [Roadmap](../roadmap.md) - M9 (unit), M10 (CI/CD), and M11 (integration) in
  context.
