# Further reading (curated references)

This is the project's external-reading hub: the sources that the architecture,
the practices, and the ADRs are built on. It is grouped by topic so you can go
deep on whichever part you are touching. The ADRs and practice docs link
directly into the sections below.

Each topic mixes three kinds of source on purpose:

- **Docs / engineering posts** - the canonical, authoritative material.
- **Books** - for depth, cited with author and the relevant chapter.
- **Video** - a specific talk or tutorial, with the channel and a note on the
  moment worth watching.

> Every link here has been checked to resolve to a real page, and every book and
> video has been verified to exist. Where a precise video timestamp could not be
> confirmed, the relevant *moment* is described instead of inventing a time.

---

## Charm.sh: Wish, Bubble Tea, and Lip Gloss

The Go stack that replaces the old SSH/TUI layer
([ADR-0002](./adr/0002-go-frontend-rust-engine.md)).

**Docs / source:**

- [charmbracelet/wish](https://github.com/charmbracelet/wish) - the SSH server.
  The README's "Bubble Tea" section is the key bit: its `bubbletea` middleware
  gives each SSH session its own `tea.Program` with PTY input/output and resize
  handling - exactly the bridge this project stopped hand-rolling.
- [charmbracelet/bubbletea](https://github.com/charmbracelet/bubbletea) - the TUI
  framework, based on The Elm Architecture (model + `Init`/`Update`/`View`).
- [charmbracelet/lipgloss](https://github.com/charmbracelet/lipgloss) - the
  styling library used for the casino look.
- [Wish v2 release notes](https://github.com/charmbracelet/wish/releases/tag/v2.0.0)
  - useful context on the current Wish/Bubble Tea v2 line and the `charm.land`
  import paths.
- [Wish `bubbletea` example](https://github.com/charmbracelet/wish/blob/main/examples/bubbletea/main.go)
  - a minimal `wish.NewServer` + `bubbletea.Middleware(teaHandler)` server; the
  shape the gateway's M4 skeleton follows.
- [charmbracelet/soft-serve](https://github.com/charmbracelet/soft-serve) - a real
  production app built on Wish; evidence the middleware approach is battle-tested.
- [The Elm Architecture](https://guide.elm-lang.org/architecture/) - the
  model/update/view pattern Bubble Tea borrows; reading the original clarifies why
  Bubble Tea code is structured the way it is.

**Video:**

- "Golang TUI Project Basics - Shopping List | Part I" (Charm channel) -
  <https://www.youtube.com/watch?v=5lxQJS3b38w>. Watch the segment where they
  implement the `View` method and explain that the view is re-rendered after every
  `Update` - the mental model you need for the game view's fan-out re-renders.

---

## gRPC and Protocol Buffers (contract-first)

The boundary between Go and Rust
([ADR-0004](./adr/0004-grpc-contract-and-opaque-state.md),
[spec-driven development](./practices/spec-driven-development.md)).

**Docs:**

- [Introduction to gRPC](https://grpc.io/docs/what-is-grpc/introduction/) - what
  gRPC is and how it uses protobuf as its IDL and wire format.
- [gRPC core concepts](https://grpc.io/docs/what-is-grpc/core-concepts/) - service
  definition, stubs, and the client/server lifecycle; the "define a service, the
  compiler generates both sides" model this project relies on.
- [Protocol Buffers overview](https://protobuf.dev/overview/) - the `.proto`
  definition language and why protobuf is a good fit for a small, stable contract.

**Book:**

- *gRPC: Up and Running* by Kasun Indrasiri and Danesh Kuruppu (O'Reilly, 2020).
  Chapter 1 ("Introduction to gRPC") and Chapter 2 ("Getting Started with gRPC")
  cover the service-definition-first workflow; Chapter 4 ("gRPC: Under the Hood")
  explains the wire protocol; Chapter 6 ("Secured gRPC") covers TLS/mTLS relevant
  to keeping the engine boundary private
  ([ADR-0008](./adr/0008-security-posture.md)).

**Video:**

- "Tonic makes gRPC in Rust stupidly simple" (Dreams of Code) -
  <https://www.youtube.com/watch?v=kerKXChDmsE>. The portion explaining how a
  `build.rs` step compiles the `.proto` into Rust types mirrors exactly how
  `engine-service/build.rs` works here.

---

## Test-Driven Development

The practice behind [test-driven development](./practices/test-driven-development.md)
and the [testing strategy](./adr/0006-testing-strategy.md).

**Docs / engineering posts:**

- [Test Driven Development](https://www.martinfowler.com/bliki/TestDrivenDevelopment.html)
  (Martin Fowler) - the concise definition of red-green-refactor and why the test
  list comes first.
- [The Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)
  (Martin Fowler) - the layered-testing metaphor (unit -> service -> UI) that the
  project's cross-language pyramid follows, including the "ice-cream cone"
  anti-pattern to avoid.

**Books:**

- *Test-Driven Development: By Example* by Kent Beck (Addison-Wesley, 2003) - the
  foundational text. Part I ("The Money Example") walks the red-green-refactor loop
  end to end; Part III ("Patterns for Test-Driven Development") is the reference
  for *how* to write tests well.
- *Succeeding with Agile* by Mike Cohn (Addison-Wesley, 2009) - the origin of the
  test pyramid concept that Fowler's article revisits.

---

## Architecture Decision Records

Why this repo keeps [ADRs](./adr/README.md) at all.

**Docs / engineering posts:**

- [Documenting Architecture Decisions](https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
  (Michael Nygard, 2011) - the original post that defined the ADR format
  (Title / Status / Context / Decision / Consequences) this project uses.
- [Architecture Decision Record](https://martinfowler.com/bliki/ArchitectureDecisionRecord.html)
  (Martin Fowler) - a short overview and history, with the key idea that an ADR is
  "a conversation with a future developer."
- [adr.github.io](https://adr.github.io/) - a hub of ADR tooling, templates, and
  variations.

---

## Texas Hold'em rules and hand evaluation

The domain the Rust `poker-engine` implements.

**Docs / source:**

- [Texas hold 'em (Wikipedia)](https://en.wikipedia.org/wiki/Texas_hold_%27em) -
  the rules: blinds, betting rounds (pre-flop/flop/turn/river), and showdown.
- [List of poker hands (Wikipedia)](https://en.wikipedia.org/wiki/List_of_poker_hands)
  - the canonical hand rankings the evaluator must reproduce.
- [Cactus Kev's Poker Hand Evaluator](http://suffe.cool/poker/evaluator.html) - the
  classic write-up mapping any 5-card hand to one of 7,462 distinct values using
  bitmasks and prime-product lookups; essential background for fast hand
  evaluation.
- [tangentforks/TwoPlusTwoHandEvaluator](https://github.com/tangentforks/TwoPlusTwoHandEvaluator)
  - the well-known 7-card lookup-table evaluator (a caching layer over Cactus
  Kev's idea).
- [HenryRLee/PokerHandEvaluator](https://github.com/HenryRLee/PokerHandEvaluator)
  - a perfect-hash 5-to-7-card evaluator with a clear explanation of why brute
  force over 21 combinations is too slow.

**Video:**

- "Neat AI does Cactus Kevs Poker Hand Evaluator Complete" (Neat AI) -
  <https://www.youtube.com/watch?v=TM_sMACxSzY>. The walk-through of building the
  flush lookup table and the `unique5` straight/high-card table is a good visual
  companion to Cactus Kev's text.

---

## SSH server security

Hardening the public front door
([ADR-0008](./adr/0008-security-posture.md)).

**Docs / source:**

- [Mozilla OpenSSH security guidelines](https://infosec.mozilla.org/guidelines/openssh)
  - a maintained, opinionated reference for secure `sshd`/`ssh` configuration
  (modern ciphers/KEX/MACs, logging, MFA, agent-forwarding caution).
- [`sshd_config` manual (OpenBSD)](https://man.openbsd.org/sshd_config) - the
  authoritative reference for settings like `MaxAuthTries`, `ClientAliveInterval`,
  `MaxSessions`, and `MaxStartups` (the limits the Wish server's hardening mirrors
  conceptually).
- [fail2ban/fail2ban](https://github.com/fail2ban/fail2ban) - log-based
  brute-force mitigation; the canonical example of the rate-limiting/ban approach
  M12 calls for.
- [jtesta/ssh-audit](https://github.com/jtesta/ssh-audit) - a tool to audit an SSH
  server's configuration against current best practices; useful for verifying
  hardening.

---

## Rust async and tonic

The Rust side of the gRPC service
([ADR-0004](./adr/0004-grpc-contract-and-opaque-state.md)).

**Docs / source:**

- [hyperium/tonic](https://github.com/hyperium/tonic) - the gRPC implementation
  used by `engine-service` (tonic + prost + tonic-build), with `helloworld` and
  `routeguide` tutorials.
- [tonic on docs.rs](https://docs.rs/tonic) - API reference, including the
  `transport` module and the `tls` feature relevant to securing the boundary.
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) - the async runtime tonic is
  built on; start here if `async`/`await` and the runtime are new.
- [The Rust Async Book](https://rust-lang.github.io/async-book/) - the official,
  deeper treatment of `async`/`await`, futures, and executors.

**Book:**

- *The Rust Programming Language* by Steve Klabnik and Carol Nichols
  ([free online](https://doc.rust-lang.org/book/)). Chapter 11 ("Writing Automated
  Tests") is the reference for the engine's unit tests; Chapters 7 and 10 ground
  the module and trait patterns the service uses.

---

## Go concurrency patterns

The Go gateway's in-process fan-out and session handling
([architecture overview](./architecture/overview.md)).

**Docs / engineering posts:**

- [Go Concurrency Patterns: Pipelines and cancellation](https://go.dev/blog/pipelines)
  (The Go Blog) - channels, fan-in/fan-out, and the `done`-channel cancellation
  idiom; directly relevant to fanning out `TableView`s and cleaning up on
  disconnect.
- [Go Concurrency Patterns: Context](https://go.dev/blog/context) (The Go Blog) -
  request-scoped cancellation and deadlines, the right tool for bounding gRPC
  calls and per-session goroutines.
- [Effective Go: Concurrency](https://go.dev/doc/effective_go#concurrency) - the
  "share memory by communicating" philosophy and goroutine/channel basics.

**Books:**

- *The Go Programming Language* by Alan A. A. Donovan and Brian W. Kernighan
  (Addison-Wesley, 2015). Chapter 8 ("Goroutines and Channels") and Chapter 9
  ("Concurrency with Shared Variables") are the definitive treatment of the
  primitives the table manager uses.
- *Concurrency in Go* by Katherine Cox-Buday (O'Reilly, 2017). Chapter 4
  ("Concurrency Patterns in Go") covers fan-out/fan-in, pipelines, and the
  `or`-channel - the patterns behind multiplayer fan-out.

---

## Dev Containers

The reproducible toolchain
([ADR-0003](./adr/0003-dev-container-environment.md)).

**Docs / source:**

- [containers.dev](https://containers.dev/) - the Development Container
  Specification's home: overview, schema, and supporting tools.
- [devcontainers/spec](https://github.com/devcontainers/spec) - the specification
  itself, including the rationale for a single, un-orchestrated container usable
  for both development *and* CI - exactly this project's use case.
- [Create a Dev Container (VS Code)](https://code.visualstudio.com/docs/devcontainers/create-dev-container)
  - the practical guide to `devcontainer.json`, Dockerfile-based containers, and
  `postCreateCommand` (the verification hook this repo uses).
