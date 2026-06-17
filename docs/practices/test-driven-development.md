# Test-driven development, as practiced here

This page explains how the project approaches testing as a *practice* - the
red-green-refactor loop and the cross-language test pyramid - and grounds it in
the one cautionary tale this repo can never forget: the Phase 0 stabilization,
where the test suite had never passed
([ADR-0001](../adr/0001-stabilize-the-rust-baseline.md)).

## Red, green, refactor

Test-driven development (TDD) is a short, repeating loop where the test comes
*before* the code:

1. **Red.** Write a small test for behavior that does not exist yet. Run it; it
   fails (it must - if it passes, it is not testing what you think). The red step
   forces you to state, precisely, what "working" means before you build it.
2. **Green.** Write the simplest code that makes the test pass. Not the elegant
   code - the *simplest* code. The goal of this step is "that works."
3. **Refactor.** Now that a passing test pins the behavior, clean up the code -
   remove duplication, improve names, simplify - running the test continuously to
   prove you did not break it. The goal of this step is "clean code."

The discipline is that you do not skip ahead: you do not write code with no
failing test driving it, and you do not refactor without a green test holding the
behavior in place. Done well, you are never more than a minute or two from a known
state, and every line of production code exists because a test asked for it.

## Why test-first, specifically

Writing the test first changes what you build, not just how you check it:

- It makes you design the interface from the caller's side before the
  implementation can bias you.
- It guarantees the test can actually fail, so a green suite *means* something - a
  test you have never seen fail is not yet evidence.
- It produces a suite as a *byproduct* of building the feature, not a chore bolted
  on afterward (which is the chore that, historically, never gets done - see
  below).

## The cross-language test pyramid

TDD here happens inside a layered pyramid that spans both languages. Each layer
owns the bugs only it can catch; full rationale is in
[ADR-0006](../adr/0006-testing-strategy.md).

```
            ^  fewer, slower, broader
            |   SSH end-to-end        (M11/M7)  two humans/sessions, real SSH
            |   gRPC integration      (M11)     full hand across the live boundary
            |   Go unit               (M9)      table manager, view mapping, input
   broad -> |   Rust unit / engine    (M9)      rules, side pots, all-in, serde
            |  more, faster, narrower
```

- **Rust unit / engine tests (the wide base).** Pure-function tests of
  `poker-engine` and `engine-service`: hand setup, betting rounds, side pots,
  all-in run-out to showdown, heads-up blinds, split pots, invalid-action
  rejection, and `GameStateBlob` serde round-trips. These are where TDD is most
  natural - the engine is a pure function, so a test is just "given this state and
  action, expect this next state." Lives in the Rust crates.
- **Go unit tests.** The table manager (seat/leave/turn enforcement), the
  proto-to-`TableView` mapping, and key/action parsing in the Bubble Tea models.
  Lives in `gateway/`.
- **gRPC integration tests.** Stand up a real `engine-service` and drive a full
  hand through the four RPCs, asserting state, per-seat views, and payouts. This
  is the layer that proves the two independently-tested halves *agree across the
  boundary* - something no unit test on either side can show.
- **SSH end-to-end tests (the narrow top).** A scripted SSH client against the
  running gateway asserting real flows, culminating in the headline case: two
  concurrent sessions seated at one table see each other's actions in real time
  (the M7 "playable" definition, automated in M11).

The shape matters: most logic is in the engine, so most tests are cheap engine
tests; only a handful of slow integration/e2e tests cover the seams. Inverting
that - mostly slow end-to-end tests - gives a suite too slow to run on every
change and too flaky to trust.

## The cautionary tale: Phase 0 (ADR-0001)

When work resumed, a review of the all-Rust codebase found the exact failure mode
TDD exists to prevent. From
[ADR-0001](../adr/0001-stabilize-the-rust-baseline.md):

- `cargo test` **did not even compile** (a test referenced a private field).
- One engine test **hung forever**: when every remaining player was all-in,
  `skip_to_next_active_player` looped infinitely.
- **Three more tests were silently failing**, hidden behind the hang.
- The advertised CI (`clippy -D warnings`, `cargo test`) **could not have been
  green.**

In short: **the test suite had never passed**, while the documentation claimed
"production-ready." That is the precise danger of treating tests as an
afterthought. A test you never ran is not a safety net; it is decoration that
*feels* like safety. And the infinite loop is the sharpest lesson of all - it was
not just a failing test, it was a reachable hang, which in a server holding many
concurrent games is a production incident (one all-in hand could tie up a worker
forever).

Phase 0's fix is the model this practice follows: make the suite *compile and pass
without hanging*, make the linters clean, and turn the checks into **real CI
gates with a job timeout** so a future hang fails fast instead of running forever.
The result - 105 tests passing, `clippy -D warnings` clean - is what "green" is
supposed to mean.

The takeaway for how we work now: **write the test first, run it, and only trust
green that a machine produced on every change.** If the Phase 0 engine had been
built test-first, the all-in case would have been a red test on day one, not a
production hang discovered months later.

## How TDD and spec-first fit together

[Spec-driven development](./spec-driven-development.md) decides *what* the
boundary is (edit `poker.proto` first); TDD decides that *each side does the right
thing* against it (write the failing test first). A typical change touches both:
edit the spec, regenerate, then red-green-refactor the Rust handler and the Go
caller, with an integration test proving they agree. Both practices share one
belief - correctness should be *enforced by something automatic*, not promised in
a README.

## Further reading

- [Test-Driven Development references](../references.md#test-driven-development)
  - Kent Beck's *Test-Driven Development: By Example*, Martin Fowler on TDD, and
    the test pyramid.
- [ADR-0006: Testing strategy](../adr/0006-testing-strategy.md) - the layered
  strategy this loop feeds.
- [ADR-0001: Stabilize the Rust baseline](../adr/0001-stabilize-the-rust-baseline.md)
  - the cautionary tale in full.
- [ADR-0007: CI/CD pipeline](../adr/0007-ci-cd-pipeline.md) - the gates and job
  timeouts that make green trustworthy.
