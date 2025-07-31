# SSH-Accessible Multiplayer Poker Game - Project & Task Breakdown

> **Format:** GitHub Issues / Kanban Ready  | **Iterations:** MVP (v0.1) & Production (v1.0)
>
> **How to use:** Copy-paste each table row into a new GitHub *Issue* (or *Task* in Projects v2).  Each row already contains labels (`epic`, `mvp`, etc.), an *estimate* field (hours), acceptance criteria, and curated learning links to enable pair-programming workflows.

---

## 0 Legend
| Field            | Usage                                                          |
|------------------|-----------------------------------------------------------------|
| **Key**          | Unique `E#-T#` identifier (Epic–Task)                           |
| **Summary**      | ≤60 chars headline                                             |
| **Labels**       | GitHub labels comma-separated (`epic`,`mvp`,`backend`, …)       |
| **Est.**         | Raw engineering hours (1 ≙ 1 focused h)                        |
| **Depends On**   | Blocking task keys                                             |
| **Acceptance**   | Bullet list of verifiable completion tests                     |
| **Learn**        | 1-3 links (docs / repo / video) to self-study while coding      |

_Note: All tasks are written to **pair-program**: the junior contributor codes while the senior narrates reasoning, reviews PRs, and answers questions in real time._

---

## Iteration 1 – MVP (Weeks 1-6)

### Epic 1 – Project Setup & CI (28 h)
| Key | Summary | Labels | Est. | Depends On | Acceptance | Learn |
|-----|---------|--------|------|------------|------------|-------|
| E1-T1 | Init mono-repo & Cargo workspace | `epic:setup,mvp` | 4 | — | `README` scaffolding, `.gitignore`, `cargo check` passes | [Cargo Book](https://doc.rust-lang.org/cargo/) |
| E1-T2 | Configure GitHub Actions CI | `ci,mvp` | 6 | E1-T1 | PR triggers run `fmt`,`clippy`,`test` on ubuntu-latest | [GH Actions Docs](https://docs.github.com/en/actions) |
| E1-T3 | Create GitHub Projects v2 board | `pm` | 2 | E1-T1 | Columns: *Backlog→Ready→Dev→Review→Done* auto-moving | [GH Projects Docs](https://docs.github.com/en/issues/trying-out-the-new-projects-experience) |
| E1-T4 | Dev-container / Nix flake (optional) | `devx` | 4 | E1-T1 | `devcontainer.json` boots VS Code with Rust toolchain | [DevContainers](https://containers.dev) |
| E1-T5 | MVP Docker build + push | `ci,infra` | 6 | E1-T2 | `docker build` succeeds, image under 40 MB | [Docker Rust](https://docs.docker.com/language/rust/) |
| E1-T6 | Base architecture ADR | `docs` | 6 | E1-T1 | ADR 001 committed, diagrams rendered via `cargo doc` | [ADR Guide](https://adr.github.io) |

### Epic 2 – Core Game Logic (MVP slice) (36 h)
| Key | Summary | Labels | Est. | Depends On | Acceptance | Learn |
|-----|---------|--------|------|------------|------------|-------|
| E2-T1 | Card + Deck structs | `backend,logic,mvp` | 6 | — | `shuffle()` cryptographically secure; 100% unit tested | rs-poker crate src |
| E2-T2 | 5-card hand evaluator | `backend,logic` | 10 | E2-T1 | Bench: ≥5 M eval/s; tests cover all 2,598,960 hands | [Henri Lee Evaluator](https://github.com/HenryRLee/)** |
| E2-T3 | FSM skeleton: Waiting→Showdown | `backend,logic` | 8 | E2-T2 | State transitions compile-time checked; integration test passes | [Game Loop Gist](https://gafferongames.com/post/fsm/) |
| E2-T4 | Betting rules (no-limit only) | `backend,logic` | 6 | E2-T3 | Pot correct after scenario unit tests; Bet validation strict | THM rules doc |
| E2-T5 | CLI stub to play heads-up vs. self | `cli` | 6 | E2-T4 | Playable hand via `cargo run`, logs actions | clap v4 docs |

### Epic 3 – SSH Server & TUI Stub (36 h)
| Key | Summary | Labels | Est. | Depends On | Acceptance | Learn |
|-----|---------|--------|------|------------|------------|-------|
| E3-T1 | russh server “hello world” | `network,ssh,mvp` | 6 | E1-T5 | `ssh localhost -p2222` prints banner, exits | russh README |
| E3-T2 | PTY allocation + raw-mode echo | `network,ssh` | 6 | E3-T1 | Terminal size negotiated; echo shows keycodes | PTY RFC |
| E3-T3 | ratatui lobby screen | `frontend,tui` | 8 | E3-T2 | Renders table list placeholder; 60 FPS no flicker | ratatui docs |
| E3-T4 | Wire CLI engine to TUI via mpsc | `frontend,ipc` | 8 | E2-T5,E3-T3 | Can play vs. self through SSH in terminal | Tokio mpsc docs |
| E3-T5 | Basic password auth + Argon2 | `security` | 8 | E3-T1 | Register user, login succeeds, hash stored salted | argon2 crate |

**Deliverable v0.1 = MVP:** one table, 2 human seats, no AI, single-node Postgres.

---

## Iteration 2 – Production Hardening (Weeks 7-12)

_Epics 4-10 follow same table format; abridged for brevity._

### Epic 4 Security & Anti-Cheat (52 h)
* T4-1 SSH key auth, T4-2 fail2ban, T4-3 MAC whitelisting, …

### Epic 5 Multiplayer Netcode (58 h)
* T5-1 authoritative state, T5-2 delta sync, T5-3 multi-table, …

### Epic 6 Database & Persistence (48 h)
* T6-1 schema, T6-2 tokio-pg pool, T6-3 hand-history ETL, …

### Epic 7 AI Bots (60 h)
* T7-1 strategy API, T7-2 rule-based bot, T7-3 difficulty scaling, …

### Epic 8 Observability (62 h)
* Tracing, metrics, Grafana dashboards, health-checks.

### Epic 9 QA & Load Test (72 h)
* Unit ≥80%, `artillery` 200 CC, fuzzing via `cargo-fuzz`.

### Epic 10 Deployment & Docs (48 h)
* Docker-Swarm blue-green, infra-as-code, operator run-book.

---

## Milestone Checklist
- [ ] MVP deployed to staging at `ssh poker.test` (password: `changeme`)
- [ ] Pair-programming walkthrough video recorded
- [ ] Production blueprint security-audited
- [ ] v1.0 tag cut and release notes published

---

## Pair-Programming Guide
1. **Driver/Navigator** rotation every 30 min.
2. Use **GitHub Codespaces** or shared tmux for live session.
3. Commit with `Co-Authored-By` trailer.
4. At EOD open a *learning journal* issue summarising insights & blockers.

Enjoy building – see you at the tables!