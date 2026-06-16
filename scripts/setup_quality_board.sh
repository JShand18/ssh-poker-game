#!/usr/bin/env bash
# One-off: add quality milestones (unit testing, CI/CD, integration, security)
# and their issues, attaching each to GitHub Project #2.
set -euo pipefail

REPO="JShand18/ssh-poker-game"
OWNER="JShand18"
PROJECT_NUMBER=2

echo "==> Labels"
create_label() { gh label create "$1" --color "$2" --description "$3" --force >/dev/null 2>&1 || true; }
create_label "area:testing"  "0e8a16" "Unit / integration testing"
create_label "area:security" "b60205" "Security and hardening"

echo "==> Milestones"
create_milestone() {
  gh api "repos/$REPO/milestones" -f title="$1" -f description="$2" >/dev/null 2>&1 \
    && echo "    created: $1" || echo "    exists/skip: $1"
}
create_milestone "M9: Unit testing"        "Unit test coverage for the Rust engine and Go gateway."
create_milestone "M10: CI/CD"              "Pipelines: multi-language build/test, codegen drift, releases."
create_milestone "M11: Integration testing" "End-to-end gRPC and SSH multiplayer test harnesses."
create_milestone "M12: Security"           "SSH hardening, dependency scanning, fuzzing, auth, boundaries."

echo "==> Issues"
mk_issue() {
  local url
  url=$(gh issue create --repo "$REPO" --title "$1" --body "$2" --milestone "$3" --label "$4")
  echo "    $url"
  gh project item-add "$PROJECT_NUMBER" --owner "$OWNER" --url "$url" >/dev/null
}

# --- M9: Unit testing ---
mk_issue "Expand engine-service unit tests (rule transitions via gRPC surface)" \
  "Unit-test each RPC handler: NewHand, ApplyAction (valid + invalid), GetValidActions, CompleteHand, including blob round-trip and TableView correctness." \
  "M9: Unit testing" "enhancement,area:testing,area:rust"
mk_issue "poker-engine edge-case tests (side pots, all-in run-out, heads-up)" \
  "Add focused tests/property tests for tricky paths: multi-way all-in side pots, all-in run-out to showdown, heads-up blinds, split pots." \
  "M9: Unit testing" "enhancement,area:testing,area:rust"
mk_issue "Go gateway unit tests (table manager, view mapping, input parsing)" \
  "Unit-test the table manager (seat/leave/turn), proto<->view mapping, and key/action parsing in the Bubble Tea models." \
  "M9: Unit testing" "enhancement,area:testing,area:go"
mk_issue "Code coverage reporting (cargo-llvm-cov for Rust, go test -cover)" \
  "Wire coverage for both languages and surface a combined report (e.g., upload to CI artifacts or Codecov)." \
  "M9: Unit testing" "enhancement,area:testing"

# --- M10: CI/CD ---
mk_issue "Codegen drift check in CI (fail if generated code is stale)" \
  "Run protoc/tonic codegen in CI and fail if committed generated code differs (git diff --exit-code)." \
  "M10: CI/CD" "enhancement,area:ci,area:proto"
mk_issue "Unified CI: Rust + Go build/test with dependency caching" \
  "Extend the workflow to build/test both the Rust workspace and the Go gateway with caching; run clippy -D warnings and go vet." \
  "M10: CI/CD" "enhancement,area:ci"
mk_issue "Release pipeline: build & publish Docker images (gateway + engine) on tag" \
  "On version tags, build and push gateway and engine-service container images (GHCR)." \
  "M10: CI/CD" "enhancement,area:ci,area:deploy"
mk_issue "CD: deploy to host on release (DigitalOcean) [stretch]" \
  "Optional continuous deployment: ship new images to the target host on release." \
  "M10: CI/CD" "enhancement,area:ci,area:deploy"

# --- M11: Integration testing ---
mk_issue "gRPC integration test: drive a full hand against a running engine-service" \
  "Spin up engine-service and play a complete hand through the gRPC API, asserting state/views and payouts." \
  "M11: Integration testing" "enhancement,area:testing,area:rust"
mk_issue "SSH end-to-end test harness (scripted client against the gateway)" \
  "Automate an SSH client (expect-style or Go SSH client) against the gateway and assert TUI flows (connect -> name -> lobby -> table)." \
  "M11: Integration testing" "enhancement,area:testing,area:go"
mk_issue "Multiplayer integration test: two sessions at one table" \
  "Drive two concurrent SSH sessions seated at one table and assert each sees the other's actions in real time." \
  "M11: Integration testing" "enhancement,area:testing,area:multiplayer"
mk_issue "docker-compose integration environment for tests" \
  "Provide a compose file that brings up gateway + engine for integration runs locally and in CI." \
  "M11: Integration testing" "enhancement,area:testing,area:deploy"

# --- M12: Security ---
mk_issue "SSH hardening: auth/idle timeouts, max sessions, connection limits" \
  "Configure Wish for sane inactivity timeouts, per-IP connection limits, and a max concurrent session cap." \
  "M12: Security" "enhancement,area:security,area:go"
mk_issue "Rate limiting / abuse protection on connections" \
  "Add connection/action rate limiting to mitigate abuse and DoS." \
  "M12: Security" "enhancement,area:security,area:go"
mk_issue "Dependency scanning: cargo-audit + govulncheck + Dependabot" \
  "Add cargo-audit and govulncheck to CI and enable Dependabot for Cargo and Go modules." \
  "M12: Security" "enhancement,area:security,area:ci"
mk_issue "Fuzz the engine action processing (cargo-fuzz)" \
  "Add a cargo-fuzz target that feeds arbitrary actions/states to the engine to surface panics or invariant violations." \
  "M12: Security" "enhancement,area:security,area:rust"
mk_issue "Re-enable authentication securely (Argon2 + lockout) [design]" \
  "When auth is un-deferred: design DB-backed auth using Argon2 and lockout, carrying forward the existing secure_auth logic; decide where it lives (gateway vs engine)." \
  "M12: Security" "enhancement,area:security"
mk_issue "Secure the gRPC boundary and secrets handling" \
  "Bind engine-service to localhost by default (or mTLS if exposed); ensure no secrets in the repo and document env/secret handling." \
  "M12: Security" "enhancement,area:security,area:infra"
mk_issue "Threat model / security review doc" \
  "Write a short threat model (assets, entry points, trust boundaries) and a security review checklist for releases." \
  "M12: Security" "enhancement,area:security"

echo "==> Done"
