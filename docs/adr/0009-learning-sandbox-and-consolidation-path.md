# ADR-0009: A learning sandbox, with a consolidation exit path

- Status: Accepted
- Date: 2026-06-16
- Related: [ADR-0002](./0002-go-frontend-rust-engine.md) (the architecture this
  reframes)

## Context

The other ADRs justify each decision on its own engineering merits: keep the
Rust engine because it is the asset ([ADR-0002](./0002-go-frontend-rust-engine.md)),
draw a tiny gRPC contract ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)),
pin the toolchain ([ADR-0003](./0003-dev-container-environment.md)), and so on.
Read together, though, they describe a system that is **more architecturally
involved than turn-based poker strictly requires** - a point made plainly in the
[architecture overview's complexity section](../architecture/overview.md#complexity-and-trade-offs):
the problem is low-to-moderate complexity, the architecture is moderate-to-high,
and the gap is accidental complexity taken on by choice.

That gap is not an accident of drift or over-engineering-by-neglect. It is
intentional, and the honest reason has not yet been written down in one place:
**this project is, on purpose, a learning sandbox.** It exists to practice and
compare non-trivial architectures and designs, and it uses a simple,
well-understood domain as the stable backdrop for doing so. This ADR records that
intent, and - because intent can change - records the exit path too.

## Decision

State explicitly that **the project is a deliberate learning vehicle**, and that
some accidental complexity is accepted *because experimenting with that
complexity is part of the point*:

- **The simple domain is a feature, not a coincidence.** Texas Hold'em is
  well-specified, finite, and familiar. A stable, well-understood domain **will
  not fight you** while you experiment with the structure around it - you are
  never debugging the rules *and* the architecture at the same time. That makes
  poker an ideal backdrop for practicing things like a gRPC service boundary,
  contract-first codegen, a polyglot build, and a server-authoritative state
  model.
- **This reframes [ADR-0002](./0002-go-frontend-rust-engine.md)'s motivation.**
  That ADR justifies the Go + Rust split mainly by engine reuse and the
  maintainer's wish to keep using Rust. The fuller truth is that the split is
  **also chosen as an architecture-learning exercise** - a real reason to build a
  cross-language, cross-process system on a problem that does not, by itself,
  demand one. Both motivations are valid; this ADR adds the one that was implicit.
- **Record an explicit exit path.** The polyglot, two-process shape is a
  *current* choice, not a permanent commitment. The project **may later
  consolidate to a single language - most likely Go** - and that would be a
  success outcome, not a failure, if the conditions below are met.

### Triggers that would justify consolidating

Consolidation should be considered when one or more of these becomes true:

- **The goal shifts from learning to shipping/operating.** Once the priority is a
  reliable, low-maintenance running service rather than a study of patterns, the
  boundary's learning value stops paying for its cost.
- **The boundary's maintenance cost outweighs its learning value.** When the
  contract, codegen, and two-toolchain upkeep feel like pure tax rather than
  practice, the trade has inverted.
- **A single deploy artifact is wanted.** If "one binary / one container to run"
  becomes a real operational requirement, the two-service surface
  ([ADR-0007](./0007-ci-cd-pipeline.md),
  [ADR-0008](./0008-security-posture.md)) is working against you.
- **Contributor-onboarding friction dominates.** If needing both Go and Rust (and
  the Dev Container around them) is the main thing keeping contributors out, the
  polyglot tax is being paid in people, not just tooling.

### What carries over cleanly

Consolidation would not be a rewrite from zero. The design choices made for the
*boundary* turn out to be exactly the choices that make the boundary *removable*:

- **The gRPC contract ports well.** The message and view types in
  `proto/poker/v1/poker.proto` are a clean, language-neutral description of the
  game's surface; they translate naturally into in-process Go types if the RPC is
  collapsed.
- **The pure engine transitions port well.** Because the engine is a stateless
  pure function over `(state, action) -> (new state, views)`
  ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)), its logic is
  straightforward to re-express in Go (or to keep in Rust via FFI) - there is no
  hidden, stateful coupling to unwind.

In other words, the thin-boundary discipline that
[ADR-0002](./0002-go-frontend-rust-engine.md) adopted for *today* also keeps the
*door open* for tomorrow.

## Rationale (why this is necessary)

- **Unstated intent is a maintenance hazard.** A future reader who assumes the
  goal was "minimal correct poker server" will quite reasonably conclude the
  architecture is over-built, and may "simplify" it without understanding that the
  complexity *was the curriculum*. Writing the intent down makes the design
  legible on its own terms.
- **Honesty completes the argument.** The other ADRs make the strongest case for
  each decision; a learning project owes its readers the meta-context that some of
  those decisions are also there to be *practiced*. This is the same value -
  documentation that matches reality - that drove
  [ADR-0001](./0001-stabilize-the-rust-baseline.md).
- **A pre-committed exit path prevents two opposite mistakes.** Without it, the
  project risks either clinging to the boundary out of sunk cost long after it
  stops teaching anything, or ripping it out impulsively the first time the
  two-toolchain tax stings. Naming the triggers in advance turns "should we
  consolidate?" into a checklist rather than an argument.
- **It protects the complexity budget.** The overview warns that every re-added
  deferred feature lands on both sides of the boundary
  ([architecture overview](../architecture/overview.md#complexity-and-trade-offs)).
  Tying that budget to an explicit learning purpose - and an explicit exit - is
  what keeps "we're learning" from becoming a blanket excuse for unbounded
  complexity.

## Alternatives considered

- **Stay single-language from the start (all-Go or all-Rust).** Simpler to build,
  run, and onboard, and the right call for a pure shipping goal. Rejected *for
  now* because it forgoes exactly the polyglot and distributed-systems learning
  this project exists to get. It remains the most likely consolidation target -
  which is the point of recording the exit path rather than pretending the choice
  is permanent.
- **Commit permanently to the polyglot, two-process architecture.** Rejected as
  premature: declaring the boundary forever optimizes for a future that may not
  arrive and discards a cheap, valuable option. Keeping consolidation explicitly
  on the table costs nothing today and preserves freedom later.

## Consequences

- Positive: the architecture is legible *as a learning choice*, so it will not be
  misread as accidental over-engineering; the exit path is decided in calm
  conditions rather than under pressure; the thin-boundary discipline is
  reinforced because it is what keeps consolidation cheap.
- Negative: stating "this is partly for learning" invites the fair critique that
  some complexity is not strictly necessary - which is true, and now openly owned
  rather than hidden.
- Risk: "it's a learning sandbox" could be misused to wave through *any* added
  complexity. Mitigation: the complexity budget and consolidation triggers above
  are the guardrails - new complexity must still earn its place, and the exit
  path must stay genuinely open (boundary thin, contract CI-enforced).

## Learning notes

The deeper lesson is that **a project's goals are themselves an architectural
input, and deserve to be documented like one.** The "right" amount of complexity
is not absolute; it is relative to what you are trying to get out of the work.
The same system can be over-engineered for one goal (ship the smallest poker
server) and appropriately engineered for another (practice production patterns on
a safe domain). Writing the goal down - and the conditions under which it would
change - is what lets a future maintainer judge the design against the intent
that produced it, instead of against an intent they merely assumed.

## Further reading

- [ADR-0002: Go front end with a Rust rules engine](./0002-go-frontend-rust-engine.md)
  - the architecture this ADR reframes as (also) a learning vehicle.
- [Architecture overview: Complexity and trade-offs](../architecture/overview.md#complexity-and-trade-offs)
  - the essential-vs-accidental breakdown this intent explains.
- [ADR-0004: gRPC contract and opaque state](./0004-grpc-contract-and-opaque-state.md)
  - why the engine transitions and contract port cleanly if consolidated.
- [Architecture Decision Records references](../references.md#architecture-decision-records)
  - on documenting decisions (and intent) for future maintainers.
