# ADR-0008: Security posture for a public SSH service

- Status: Accepted
- Date: 2026-06-16
- Related issues: M12 "Security" (board #2): SSH hardening, rate limiting,
  dependency scanning, fuzzing, secure auth re-enable (design), gRPC boundary,
  secrets handling, threat model

## Context

The product is, by design, a service you reach by typing `ssh` at a stranger's
machine. The moment it is deployed (M8) it presents a **publicly reachable SSH
listener** to the open internet, which means automated scanners and brute-force
bots will find it within minutes. That is a fundamentally different threat
surface from a local game.

Two facts from earlier decisions shape the security picture:

- **Authentication is deferred** ([ADR-0002](./0002-go-frontend-rust-engine.md)):
  the first playable milestone uses guest identity only (prompt for a display
  name; no password/auth). The original codebase's `secure_auth` (Argon2,
  lockout) is left in place but **not** wired into the Go gateway yet.
- **The engine is a separate gRPC service**
  ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)) that already holds the
  authoritative state and decides, per seat, who can see which cards.

[ADR-0001](./0001-stabilize-the-rust-baseline.md) is also relevant in a security
light: the original engine could be driven into an **infinite loop** by a normal
all-in situation. Against untrusted input, a reachable hang is a denial-of-
service primitive, not just a bug.

## Decision

Adopt a defense-in-depth security posture, tracked under M12, organized by trust
boundary. Items are a mix of "build now" and "design now, enable later":

**SSH front door (the public attack surface):**

- **Harden the listener:** sane inactivity/idle timeouts, a maximum concurrent
  session cap, and per-IP connection limits, configured on the Wish server.
- **Rate limiting / abuse protection** on connections and actions to blunt
  brute-force and DoS attempts.
- **A persistent host key** (M8) so clients get a stable identity and are not
  trained to ignore host-key warnings.

**The gRPC boundary (internal, must stay internal):**

- **Bind `engine-service` to localhost by default.** It is an internal rules
  service; it must not be exposed to the internet. If it ever must cross hosts,
  require **mTLS** rather than an open port.

**Engine robustness against hostile input:**

- **Fuzz the engine's action processing** (`cargo-fuzz`): feed arbitrary
  actions/states and assert no panics or invariant violations. The all-in hang
  is the canonical reason this matters.

**Supply chain:**

- **Dependency scanning:** `cargo-audit` (Rust) and `govulncheck` (Go) in CI,
  plus Dependabot for Cargo and Go modules (runs in the pipeline -
  [ADR-0007](./0007-ci-cd-pipeline.md)).

**Authentication (deferred; design now):**

- When auth is un-deferred, **re-enable it securely**: DB-backed Argon2 with
  lockout, carrying forward the existing `secure_auth` logic, with an explicit
  decision on where it lives (gateway vs. engine). Until then, the system is
  honestly documented as guest-only.

**Secrets and process:**

- **No secrets in the repository**; document environment/secret handling for the
  host key and any future credentials.
- **A written threat model** (assets, entry points, trust boundaries) and a
  per-release security review checklist.

## Rationale (why this is necessary)

- **A public SSH listener is a magnet for automated abuse.** Brute-force and
  scan traffic against SSH is constant and indiscriminate. Idle timeouts, session
  caps, per-IP limits, and rate limiting are the well-established countermeasures
  that keep one host from being overwhelmed and keep logs readable enough to spot
  real incidents. These are table stakes for anything listening on the SSH port.
- **The engine must never be a back door.** `engine-service` holds authoritative
  state and renders hole cards per seat; exposing it publicly would hand an
  attacker the authoritative interface directly. Binding it to localhost (or
  mTLS if it must be remote) keeps the SSH gateway as the *only* front door, so
  all the front-door controls actually mean something.
- **Untrusted input plus a possible hang equals DoS.** Because clients drive the
  engine through their actions, any unhandled panic or loop on a crafted action
  is reachable from the internet. Fuzzing the action processor turns "we think
  it's robust" into "we tried hard to break it," directly extending the lesson of
  the [ADR-0001](./0001-stabilize-the-rust-baseline.md) hang.
- **Most breaches start in dependencies.** Two language ecosystems mean two
  supply chains; `cargo-audit`, `govulncheck`, and Dependabot make known-vuln
  detection automatic rather than a thing someone remembers to check.
- **Honesty about deferred auth is itself a security property.** The original
  README claimed "Enhanced Security" while authentication was effectively
  bypassed - a dangerous gap between claim and reality
  ([ADR-0001](./0001-stabilize-the-rust-baseline.md)). Documenting the system as
  guest-only *now*, and designing the secure re-enable *before* shipping auth,
  prevents repeating that exact mistake.
- **Server-authoritative rendering is already a structural defense.** Because Go
  only holds an opaque blob and renders from per-seat `TableView`s built in the
  engine ([ADR-0004](./0004-grpc-contract-and-opaque-state.md)), a player
  cannot read another player's hole cards even by tampering with the client -
  the bytes never reach them. The posture here protects *that* boundary rather
  than re-implementing it.

## Threat model (sketch)

- **Assets:** game integrity (no cheating, no seeing others' hole cards),
  service availability, the host key, and (once auth lands) user credentials.
- **Entry points:** the public SSH listener (primary); the gRPC port (must be
  internal); the supply chain (dependencies); the CI/CD pipeline.
- **Trust boundaries:** untrusted SSH client -> gateway (untrusted input
  crossing into our process); gateway -> engine (internal, localhost/mTLS);
  repo/CI -> released images.
- **Primary risks:** brute-force/DoS on SSH, a crafted action hanging or
  panicking the engine, an exposed engine port, a vulnerable dependency, a leaked
  secret, and (future) weak auth.

## Alternatives considered

- **Ship guest-only and harden later.** Rejected as a *posture* (though auth
  itself is deferred): the front-door hardening, the localhost binding, and
  dependency scanning are cheap and must exist the day we are reachable. Only
  full authentication is deferred, and it is deferred *with* a design.
- **Expose `engine-service` directly (no gateway in front).** Rejected: it makes
  the authoritative interface internet-facing and discards every SSH-layer
  control. The gateway must be the sole front door.
- **Skip fuzzing; rely on unit tests.** Unit tests check the cases we thought
  of; fuzzing checks the ones we did not, which is precisely where the
  internet-facing risk lives. Rejected as sufficient on its own.
- **Roll a custom auth scheme when auth returns.** Rejected in advance:
  re-use the vetted Argon2 + lockout approach already present, rather than
  inventing credential handling.

## Consequences

- Positive: the public surface is hardened from day one; the engine stays
  internal; hostile input is actively tested against; the supply chain is
  watched; the system's security claims match reality.
- Negative: hardening adds configuration and operational surface (timeouts,
  limits, key management, mTLS if used); fuzzing and scanning add CI cost; a
  threat model is a living document that needs upkeep.
- Risk: deferred authentication means that, until it lands, there is no user
  identity beyond a self-chosen display name - acceptable for a guest poker game,
  but it must be stated plainly and revisited before any feature that assumes
  real identity. Misconfiguration (e.g. accidentally binding the gRPC port
  publicly) would be serious; the localhost default and a release checklist
  mitigate it.

## Learning notes

Security is **boundary-by-boundary reasoning**, not a feature you bolt on.
Every arrow in the architecture diagram is a trust boundary, and the useful
question at each is "what does the other side get to do, and what happens if it
is hostile?" The same boundary-placement instinct that drove the architecture
([ADR-0002](./0002-go-frontend-rust-engine.md)) and the data model
([ADR-0004](./0004-grpc-contract-and-opaque-state.md)) drives the security
posture: keep the front door singular and hardened, keep the engine internal,
distrust client input, and - above all - never let the documentation claim a
protection the code does not actually provide.

## Further reading

- [SSH server security references](../references.md#ssh-server-security) -
  hardening guidance, rate limiting, and brute-force mitigation.
- [Rust async and tonic references](../references.md#rust-async-and-tonic) -
  on the tonic transport and its TLS options for the gRPC boundary.
- [ADR-0004: gRPC contract and opaque state](./0004-grpc-contract-and-opaque-state.md)
  - the server-authoritative design that protects hole cards.
- [ADR-0007: CI/CD pipeline](./0007-ci-cd-pipeline.md) - where dependency
  scanning and fuzzing run.
- [Roadmap](../roadmap.md) - M12 in context.
