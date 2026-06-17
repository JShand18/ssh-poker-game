# ADR-0003: A Dev Container is the canonical environment

- Status: Accepted
- Date: 2026-06-16
- Related issues: M1 "Dev environment" (board #2): Add Dev Container (Go + Rust +
  protoc + protobuf Go plugins + buf)

## Context

[ADR-0002](./0002-go-frontend-rust-engine.md) commits us to a polyglot system:
a Go gateway (Wish + Bubble Tea) and a Rust `engine-service`, bridged by a gRPC
contract that both languages generate code from. That decision quietly changed
the cost of "set up the project." A contributor now needs, all at once and at
compatible versions:

- A **Go** toolchain (for the gateway and the Go protobuf code).
- A **Rust** toolchain pinned to **1.82.0** - the version the host used and the
  one `engine-service` builds against (see `.devcontainer/Dockerfile`).
- The **Protocol Buffers compiler** (`protoc`) plus the Go plugins
  `protoc-gen-go` and `protoc-gen-go-grpc`, and optionally `buf`.

There was an additional, concrete constraint: **Go was not installed on the
maintainer's host at all**, and there was no desire to pollute the host with a
second language toolchain just to compile generated code. The all-Rust history
of this repo (`cargo` everywhere) meant the environment had silently been
"whatever was on the one machine that worked" - exactly the kind of implicit
state that [ADR-0001](./0001-stabilize-the-rust-baseline.md) showed we cannot
trust. A test suite that had never passed and a CI that could not have been
green both trace back, in part, to an environment nobody could reproduce.

We needed one definition of "the environment" that is checked into the repo,
identical for every contributor, and identical to the one CI will eventually
use.

## Decision

Make a **Dev Container** (`.devcontainer/`) the canonical development
environment. The repository carries a `Dockerfile` and a `devcontainer.json`
that together provision the full toolchain:

- Base image `mcr.microsoft.com/devcontainers/go:1-bookworm` (Go + Debian).
- `protobuf-compiler` from apt.
- Rust **1.82.0** via `rustup` (minimal profile) plus `clippy` and `rustfmt`,
  installed for the non-root `vscode` user.
- `protoc-gen-go`, `protoc-gen-go-grpc`, and `buf` via `go install`.
- A `postCreateCommand` that prints `go`, `cargo`, `protoc`, `protoc-gen-go`,
  and `buf` versions so a broken environment fails loudly at creation time.

This container has been **verified end to end**: Go codegen (`make proto-go`)
and `go build ./...` both pass inside it, and the Rust workspace builds green.

## Rationale (why this is necessary)

- **"Works on my machine" is a production failure in disguise.** The single
  most expensive class of bug in a polyglot project is the one that only
  reproduces in one environment. Pinning the toolchain in an image deletes that
  class of bug: there is exactly one Go, one Rust (1.82.0), and one `protoc`,
  and they live in version control next to the code they build.
- **A polyglot toolchain is genuinely hard to assemble by hand.** Go, Rust at a
  specific version, `protoc`, two Go plugins, and `buf` - installed in the right
  order, on the right user, with the right `PATH` - is a multi-step setup that
  is easy to get subtly wrong (a mismatched `protoc-gen-go` quietly emits
  different code). Encoding it once in a `Dockerfile` means it is assembled
  correctly every time, not re-derived from memory by each new contributor.
- **CI parity is the real prize.** Generated code that compiles locally but not
  in CI (or vice versa) is the classic drift trap. When the same image backs
  both local development and the pipeline (see
  [ADR-0007](./0007-ci-cd-pipeline.md)), "green on my laptop" and "green in CI"
  mean the same thing, which is the precondition for the codegen-drift check to
  be meaningful (see [ADR-0006](./0006-testing-strategy.md)).
- **Onboarding collapses to one step.** "Open the folder in a Dev Container"
  replaces a page of install instructions. For a project that doubles as a
  learning resource, low friction to first build matters.
- **The host stays clean.** The maintainer keeps a Rust-only host and still gets
  a full Go environment, with no global installs and no version conflicts.

## What it provides

- Reproducible, pinned versions of every tool both halves need.
- A single verification command (`postCreateCommand`) that proves the toolchain
  on creation.
- Forwarded ports `2222` (SSH gateway) and `50051` (gRPC engine) so the running
  system is reachable from the host during development.
- Editor parity via recommended extensions (Go, rust-analyzer, proto3, TOML).

## Alternatives considered

- **Document manual setup in the README.** Cheapest to write, most expensive to
  live with: instructions rot, versions drift, and every contributor's machine
  becomes a slightly different environment. This is the status quo that produced
  an unreproducible baseline in the first place.
- **Install Go directly on the host.** Rejected: it pollutes a deliberately
  Rust-only machine, pins nothing for anyone else, and gives no CI parity.
- **Nix / devbox.** A strong reproducibility story, arguably stronger than
  Docker, but a steeper learning curve and weaker out-of-the-box editor
  integration. For a project whose goal includes approachability, the Dev
  Container's first-class IDE support won.
- **A plain Dockerfile / docker-compose dev image (no Dev Container spec).**
  Workable, but it gives up the standardized `devcontainer.json` lifecycle
  (`postCreateCommand`, port forwarding, editor wiring) that makes onboarding a
  single click. We still expect a separate runtime compose file for deployment
  (M8); that is a different artifact from this development image.

## Consequences

- Positive: one reproducible environment for everyone; the host stays clean; CI
  can reuse the exact same toolchain; onboarding is a single step; an entire
  category of version-mismatch bugs disappears.
- Negative: contributors need Docker and an editor (or CLI) that understands the
  Dev Container spec; the first build pays an image-build cost; the pinned
  versions are now something we must consciously bump over time.
- Risk: the image can drift from production if we are careless - the dev image
  is for *building and testing*, while deployment images (M8) are separate and
  must be kept consistent in their toolchain versions. Pinning Rust to 1.82.0
  and a specific Go base image mitigates silent drift, at the cost of periodic
  deliberate upgrades.

## Learning notes

The deeper lesson is that **the environment is part of the source code.** A
build is only reproducible if the toolchain that produces it is itself
versioned and shared. In a single-language project you can often get away with
"install the latest and go"; the moment you add a second language and a code
generator, the combinatorial space of "which versions of which tools" becomes
the dominant source of friction. Capturing that space once, declaratively, is
the same instinct as pinning dependencies or writing a lockfile - it just
operates one level up, on the tools rather than the libraries.

## Further reading

- [Dev Containers references](../references.md#dev-containers) - the
  specification (`containers.dev`) and the VS Code Dev Containers guide.
- [gRPC and Protocol Buffers references](../references.md#grpc-and-protocol-buffers-contract-first)
  - why a shared `protoc` toolchain matters for code generation.
- [Development guide](../development.md) - how to actually open and use the
  container day to day.
