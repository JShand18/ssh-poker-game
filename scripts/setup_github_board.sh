#!/usr/bin/env bash
# One-off: create milestones, labels, issues for the Go/Rust migration and
# attach each issue to GitHub Project #2 (JShand18/ssh-poker-game).
set -euo pipefail

REPO="JShand18/ssh-poker-game"
OWNER="JShand18"
PROJECT_NUMBER=2

echo "==> Creating area labels"
create_label() {
  gh label create "$1" --color "$2" --description "$3" --force >/dev/null 2>&1 || true
}
create_label "area:infra"       "5319e7" "Dev environment / repo structure"
create_label "area:proto"       "1d76db" "gRPC / protobuf contract"
create_label "area:rust"        "b7410e" "Rust engine-service"
create_label "area:go"          "00add8" "Go gateway"
create_label "area:tui"         "0e8a16" "Bubble Tea / Lip Gloss views"
create_label "area:multiplayer" "fbca04" "Table orchestration / sessions"
create_label "area:deploy"      "555555" "Build / deploy / ops"
create_label "area:ci"          "c5def5" "Continuous integration"

echo "==> Creating milestones"
create_milestone() {
  # $1 title, $2 description
  gh api "repos/$REPO/milestones" -f title="$1" -f description="$2" >/dev/null 2>&1 \
    && echo "    created: $1" \
    || echo "    exists/skip: $1"
}
create_milestone "M1: Dev environment"        "Reproducible Dev Container with Go + Rust + protoc toolchain."
create_milestone "M2: Proto and structure"    "Repo restructure and shared gRPC contract + codegen."
create_milestone "M3: Rust engine-service"    "Stateless tonic gRPC service wrapping poker-engine."
create_milestone "M4: Go gateway skeleton"    "Wish + Bubble Tea SSH gateway talking to the engine."
create_milestone "M5: TUI views"              "Name, lobby, and game views with Lip Gloss."
create_milestone "M6: Multiplayer"            "Table manager, turn enforcement, session fan-out."
create_milestone "M7: Playable"               "Two humans complete a full hand over SSH."
create_milestone "M8: Deploy"                 "Host key, config, Docker compose, Makefile, docs."

echo "==> Creating issues + adding to project #$PROJECT_NUMBER"
mk_issue() {
  # $1 title, $2 body, $3 milestone, $4 comma-separated labels
  local url
  url=$(gh issue create --repo "$REPO" \
    --title "$1" --body "$2" --milestone "$3" --label "$4")
  echo "    $url"
  gh project item-add "$PROJECT_NUMBER" --owner "$OWNER" --url "$url" >/dev/null
}

mk_issue "Add Dev Container (Go + Rust + protoc + plugins + buf)" \
  "Create .devcontainer with Go, Rust 1.82, protobuf-compiler, protoc-gen-go, protoc-gen-go-grpc, and buf. Verify go/cargo/protoc and codegen run inside the container." \
  "M1: Dev environment" "enhancement,area:infra"

mk_issue "Restructure repo: add proto/, gateway/, crates/engine-service" \
  "Add top-level proto/ dir, a Go module under gateway/, and a new Rust crate crates/engine-service in the workspace. Mark crates/ssh-poker-server and crates/poker-tui for retirement after parity." \
  "M2: Proto and structure" "enhancement,area:infra"

mk_issue "Define proto/poker.proto (Action, TableView, GameStateBlob, EngineService)" \
  "Define the shared contract: Action, Card, PlayerView, TableView, GameStateBlob (opaque serde bytes), and the EngineService with NewHand/ApplyAction/GetValidActions/CompleteHand RPCs." \
  "M2: Proto and structure" "enhancement,area:proto"

mk_issue "Wire codegen: tonic-build (Rust) + protoc-gen-go/buf (Go)" \
  "Set up tonic-build in engine-service build.rs and protoc-gen-go/protoc-gen-go-grpc (optionally via buf) for the Go module. Add Make targets to regenerate." \
  "M2: Proto and structure" "enhancement,area:proto"

mk_issue "Scaffold crates/engine-service tonic server + serde blob round-trip" \
  "Stand up the tonic server skeleton and prove a GameState serde encode/decode round-trip through GameStateBlob bytes." \
  "M3: Rust engine-service" "enhancement,area:rust"

mk_issue "Implement NewHand / ApplyAction / GetValidActions / CompleteHand" \
  "Implement the four RPCs on top of poker-engine's pure transitions (process_action, get_valid_actions, complete_hand)." \
  "M3: Rust engine-service" "enhancement,area:rust"

mk_issue "Map domain <-> proto and build per-seat TableView" \
  "Convert poker-engine types to/from proto and build per-seat TableView (public info for all seats + that viewer's hole cards)." \
  "M3: Rust engine-service" "enhancement,area:rust"

mk_issue "engine-service unit/integration tests" \
  "Test a full hand via the service surface: deal, betting rounds, showdown, payout; invalid-action handling." \
  "M3: Rust engine-service" "enhancement,area:rust"

mk_issue "Wish SSH server + bubbletea middleware (connect -> hello TUI)" \
  "Stand up the Wish SSH server with bubbletea middleware so an SSH connection shows a minimal TUI. Persistent host key wiring stubbed." \
  "M4: Go gateway skeleton" "enhancement,area:go"

mk_issue "gRPC client to engine-service + keypress->Rust->render smoke test" \
  "Add a gRPC client wrapper in the Go gateway and prove a keypress triggers a real engine RPC and renders the result." \
  "M4: Go gateway skeleton" "enhancement,area:go"

mk_issue "Name-prompt view (guest identity)" \
  "Bubble Tea view to capture a guest display name on connect (no auth)." \
  "M5: TUI views" "enhancement,area:tui"

mk_issue "Lobby view (list/create/join tables) with Lip Gloss" \
  "Lobby model: list tables, create a table, join a table, styled with Lip Gloss (port casino theme conceptually)." \
  "M5: TUI views" "enhancement,area:tui"

mk_issue "Game view (poker table render) with Lip Gloss" \
  "Game model: render the poker table, community cards, pot, seats, and the action bar from TableView." \
  "M5: TUI views" "enhancement,area:tui"

mk_issue "Go table manager: authoritative state blob per table + seats" \
  "Implement an in-memory table manager that owns one GameStateBlob per table and tracks seated players." \
  "M6: Multiplayer" "enhancement,area:multiplayer"

mk_issue "Turn enforcement via engine + fan-out via program.Send" \
  "Validate/apply actions through the engine, then push updated TableViews to all seated sessions via tea.Program.Send." \
  "M6: Multiplayer" "enhancement,area:multiplayer"

mk_issue "Disconnect/leave handling" \
  "Handle SSH disconnects and explicit leaves: free the seat, notify table-mates, clean up empty tables." \
  "M6: Multiplayer" "enhancement,area:multiplayer"

mk_issue "Two humans complete a full hand over SSH (end-to-end test)" \
  "Milestone validation: two SSH sessions seated at one table play a complete hand (deal -> betting -> showdown -> payout)." \
  "M7: Playable" "enhancement,area:multiplayer"

mk_issue "Persistent Wish host key + config flags" \
  "Persist the Wish ed25519 host key across restarts; add config flags (ports, engine address, etc.)." \
  "M8: Deploy" "enhancement,area:deploy"

mk_issue "Dockerfile + docker-compose (gateway + engine) + Makefile" \
  "Containerize both binaries and provide docker-compose plus a Makefile for codegen/build/run." \
  "M8: Deploy" "enhancement,area:deploy"

mk_issue "README / run docs" \
  "Document local dev (dev container), codegen, running both services, and SSH connection instructions." \
  "M8: Deploy" "enhancement,area:deploy"

mk_issue "Add Go CI job (build/vet/test)" \
  "Add a GitHub Actions job to build/vet/test the Go module alongside the existing Rust CI." \
  "M2: Proto and structure" "enhancement,area:ci"

echo "==> Done"
