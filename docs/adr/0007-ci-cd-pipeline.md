# ADR-0007: CI/CD gates, codegen drift, and releases

- Status: Accepted
- Date: 2026-06-16
- Related issues: M10 "CI/CD" (board #2): unified Rust + Go build/test, codegen
  drift check, release image publishing; cross-cutting "Add Go CI job"

## Context

[ADR-0001](./0001-stabilize-the-rust-baseline.md) established that this project's
entire credibility rests on a pipeline you can believe. During stabilization we
made `clippy -D warnings` and `cargo test` *real* gates and added a **job
timeout** so a future hang fails fast instead of running forever. That covered
the Rust half of a then-all-Rust repo.

The migration changes the shape of CI in three ways:

- There is now a **Go module** (`gateway/`) that must build, vet, and test
  alongside the Rust workspace - the cross-cutting "Add Go CI job" item exists
  for exactly this.
- Both halves consume **generated code** from a shared `.proto`
  ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)), so "did someone forget
  to regenerate?" becomes a first-class failure mode
  ([ADR-0006](./0006-testing-strategy.md)).
- The deployable artifact is no longer one Rust binary but **two services**
  (gateway + engine) that need to be built and shipped as images (M8/M10).

## Decision

Define a CI/CD pipeline whose **merge gates** are the contract for landing any
change, ideally running on the same toolchain as the Dev Container
([ADR-0003](./0003-dev-container-environment.md)) for parity:

**Quality gates (must pass to merge):**

- **Rust:** `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace`, each under a **job timeout** so a hang fails fast.
- **Go:** `go build ./...`, `go vet ./...`, and `go test ./...` in `gateway/`
  (the "Add Go CI job" item), also time-bounded.
- **Codegen-drift check:** regenerate from `proto/poker/v1/poker.proto`
  (`make proto-go` for Go; the `tonic-build` step for Rust) and fail if the
  committed generated code differs (`git diff --exit-code`). Stale generated
  code is a silent correctness bug; this gate makes it loud.
- **Formatting:** `cargo fmt --check` advisory for now (as set during
  stabilization), to be promoted to a hard gate later.

**Release / CD:**

- On version tags, **build and publish container images** for both
  `engine-service` and the Go gateway (e.g. to GHCR), so a release is a set of
  immutable, versioned images.
- Continuous deployment to a host (e.g. DigitalOcean) on release is a documented
  **stretch goal**, kept separate from the merge gates.

**Supporting checks** (detailed in their own ADRs): dependency scanning
(`cargo-audit`, `govulncheck`, Dependabot) and coverage reporting belong to the
pipeline but are specified under
[security](./0008-security-posture.md) (M12) and
[testing](./0006-testing-strategy.md) (M9).

## Rationale (why this is necessary)

- **Green CI is the safety net for a now-polyglot system.** The migration adds a
  second language and a network boundary; the only way to change architecture
  under that much surface area without fear is automated checks on every change.
  This is the direct continuation of
  [ADR-0001](./0001-stabilize-the-rust-baseline.md)'s thesis: a green pipeline is
  the contract that lets you move fast safely.
- **`-D warnings` and friends prevent slow rot.** Treating clippy warnings as
  errors stops the gradual accumulation of "we'll fix it later" that turns a
  clean codebase into a noisy one where real warnings hide. The same logic
  extends to `go vet` on the Go side.
- **Job timeouts turn hangs into failures.** The original engine hung a test
  forever and masked three others. A bounded job converts "runs forever" into a
  fast, visible red - this was added during stabilization precisely so a future
  hang cannot recur silently, and it must cover both languages now.
- **The drift gate is non-negotiable for generated code.** Because both sides
  build from generated stubs, a contract change that is committed without
  regeneration compiles and passes unit tests while the wire formats disagree.
  Only a regenerate-and-diff step catches this mechanically. It is cheap to run
  and the failure it prevents is expensive and confusing.
- **Releasing immutable images makes deployment reproducible.** Two services
  must be deployed together at compatible versions. Publishing versioned images
  on tag (rather than building on the production host) means what you tested is
  exactly what you ship - the deploy-time analog of the reproducibility argument
  in [ADR-0003](./0003-dev-container-environment.md).

## Alternatives considered

- **Keep CI Rust-only.** Leaves the entire Go gateway - half the system, and the
  half that handles untrusted SSH input - unguarded. Rejected; the Go job is a
  named work item.
- **No drift gate; rely on a pre-commit hook or discipline.** Hooks are
  bypassable and discipline is exactly what
  [ADR-0001](./0001-stabilize-the-rust-baseline.md) warns against trusting.
  A server-side gate that cannot be skipped is the point. Rejected.
- **No job timeouts.** A single hang can wedge the runner and hide other
  failures, as already happened once. Rejected.
- **Build images on the production host at deploy time.** Simple, but it means
  "what runs in prod" is rebuilt from source on the host and may differ from
  what was tested. Rejected in favor of publishing tested, immutable images.
- **Make `cargo fmt` a hard gate immediately.** Reasonable, but stabilization
  deliberately left formatting advisory to avoid noise during heavy churn; we
  promote it once the codebase settles. Deferred, not rejected.

## Consequences

- Positive: every change is checked in both languages; hangs fail fast; drift is
  impossible to merge unnoticed; releases are reproducible, versioned images;
  the pipeline mirrors local builds via the shared toolchain.
- Negative: CI does more work and takes longer (two toolchains, codegen,
  image builds); caching and path-scoping become necessary to keep it quick;
  more moving parts to maintain.
- Risk: a slow or flaky pipeline erodes trust and tempts people to bypass it -
  the very failure mode that started this project's troubles. Mitigation:
  dependency caching, sensible job timeouts, and keeping the slow layers (e2e,
  image builds) off the hot path where possible.

## Learning notes

CI/CD is **executable policy.** Every rule that matters - "no warnings," "tests
pass," "generated code is fresh," "we ship what we tested" - is worthless as a
guideline and powerful as a gate, because a gate is enforced by a machine that
never forgets and never makes an exception under deadline pressure. The job
timeout is the sharpest example: it encodes the hard-won lesson that *a process
that never finishes is a failure*, and it encodes it somewhere a tired human
cannot accidentally remove it mid-incident.

## Further reading

- [Test-Driven Development references](../references.md#test-driven-development)
  - the testing philosophy these gates enforce.
- [ADR-0003: Dev Container](./0003-dev-container-environment.md) - the shared
  toolchain that gives CI/local parity.
- [ADR-0006: Testing strategy](./0006-testing-strategy.md) - what the gates run.
- [ADR-0008: Security posture](./0008-security-posture.md) - dependency scanning
  and fuzzing that also live in the pipeline.
- [Roadmap](../roadmap.md) - M10 in context.
