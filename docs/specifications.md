# SSH-Accessible Multiplayer Poker Game - Technical Specification

> **Version:** 1.0  **Status:** Draft (Engineering Sign-off Required)
> **Target Release:** v0.1-alpha (12-week schedule)  
> **Related Docs:** [requirements.md](requirements.md), [tasks.md](tasks.md)

---

## 1  Purpose & Scope
This specification translates the high-level business and functional requirements into a deterministic engineering blueprint for building a production-ready SSH-accessible Texas Hold’em poker game.  It defines system architecture, component interfaces, data models, operational budgets, and acceptance test matrices that must be met for the project to ship.

## 2  System Architecture

### 2.1  High-Level Diagram (textual)
```
┌────────────┐   SSH   ┌────────────────────┐  internal IPC   ┌───────────────┐
│ SSH Client │────────▶│ SSH Server (russh) │────────────────▶│ Game Engine   │
└────────────┘         │  + PTY mux        │      gRPC       │ (Authoritative)│
                       └────────┬──────────┘                 └─────┬─────────┘
                                │                                   │
                                │async mpsc                         │SQL (tokio-pg)
                                ▼                                   ▼
                       ┌────────────────────┐              ┌─────────────────┐
                       │ Terminal UI (TUI)  │              │ PostgreSQL      │
                       └────────────────────┘              └─────────────────┘
                                ▲                                  ▲
                                │WebSockets (metrics)              │OTel exporter
                                ▼                                  │
                       ┌────────────────────┐              └─────────────────┘
                       │ Observability      │
                       └────────────────────┘
```

*All gameplay rules, randomisation, and anti-cheat logic live inside the Game Engine crate; all clients are dumb terminals.*

### 2.2  Runtime Processes
| Process            | Binary/Crate          | Responsibility                                                      |
|--------------------|-----------------------|---------------------------------------------------------------------|
| `pokerd`           | `ssh-server`          | Accept SSH TCP sockets, perform auth, multiplex PTYs, forward IPC   |
| `tableworker`      | `poker-engine`        | Own authoritative state for **one** table, runs FSM tick loop       |
| `stats-worker`     | `analytics`           | ETL hand histories ⇒ aggregated stats, updates leaderboards         |
| `otel-agent`       | side-car              | Ship traces + metrics to Grafana stack                              |

### 2.3  Threading & Concurrency
* Tokio multi-thread scheduler (num_cpus*1.5).
* Each inbound SSH channel handled by an **actor** (`SessionActor`) pinned to a worker thread.
* GameEngine uses **lock-free RwLock** for hot path; mutation performed by owning task only.

## 3  Component Specifications

### 3.1  SSH Server (`ssh-server` crate)
| Aspect            | Spec                                                                              |
|-------------------|------------------------------------------------------------------------------------|
| Protocol          | SSH-2 RFC 4253, curves: `curve25519-sha256`, ciphers: `aes256-gcm`                  |
| Auth              | *mode 1* password (Argon2id, OPS=3, MEM=64 MB, PAR=1)  *mode 2* ed25519 public key |
| PTY               | 80×24 min; winch handled; UTF-8 only                                               |
| Rate-limits       | 5 auth attempts / 1 min; 200 msgs / 10 sec per channel                             |
| Hardening         | no port-forward, no SFTP, max sessions=2 per user                                  |

### 3.2  Game Engine (`poker-engine` crate)
* **FSM States:** `Waiting` → `PreFlop` → `Flop` → `Turn` → `River` → `Showdown` → `Payout`.
* **Tick Rate:** 60 Hz; server evaluates timers & pushes deltas.
* **Randomness:** ChaCha20 RNG seeded from `/dev/urandom`, reseeded every 10 min.
* **Hand Eval:** 32-bit perfect-hash lookup table (≈130 KB) giving ranking in 2–3 ns.
* **Pot Model:** fixed-precision i128 cents; side-pot tree for multi-all-in.

### 3.3  Terminal UI (`terminal-ui` crate)
| View            | Widgets                                                                    |
|-----------------|-----------------------------------------------------------------------------|
| Table Screen    | Community cards row, circular player boxes, pot HUD, action prompt         |
| Lobby Screen    | List<active tables> + create/join shortcuts                                 |
| Help Overlay    | Key-map cheat-sheet (press ?)                                               |

### 3.4  Data Persistence Layer (`database` crate)
* Connection pool: 50.
* Schema versioning via `sqlx migrate`.
* Critical tables: `users`, `games`, `hands`, `actions`, `sessions`, `chat`.
* Index budget: <30% total DB size.

### 3.5  AI Engine (`ai-bot` crate)
* Strategy interface `trait PokerStrategy` (pure functions for determinism in tests).
* Difficulty tuning via injected `DifficultySettings` struct.
* ML path (v2): integrate Candle for on-device inference (<30 ms/batch).

## 4  Data Model Details

```sql
-- simplified excerpt
CREATE TABLE users (
  id          UUID  PRIMARY KEY,
  username    VARCHAR(50) UNIQUE NOT NULL,
  email       TEXT    UNIQUE NOT NULL,
  pw_hash     TEXT    NOT NULL,
  created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE games (
  id          UUID PRIMARY KEY,
  table_name  TEXT NOT NULL,
  stakes      JSONB NOT NULL,
  status      TEXT CHECK (status IN ('waiting','running','finished')),
  created_at  TIMESTAMPTZ DEFAULT now()
);
```

## 5  Message/IPC Spec

| Channel      | Type         | Payload (serde JSON)                              |
|--------------|--------------|---------------------------------------------------|
| `GameTx`     | broadcast    | `StateDelta {version,u8,data}`                    |
| `PlayerTx`   | unicast      | `Prompt {allowed_actions, min_raise}`             |
| `ChatTx`     | room         | `ChatMsg {sender, body, ts}`                      |

Delivery: at-least-once with monotonically-increasing `version` for idempotency.

## 6  Performance Budgets
| Metric                     | Target                 |
|----------------------------|------------------------|
| Auth round-trip            | < 150 ms p95           |
| Action → broadcast delta   | < 50 ms p95            |
| Hand eval throughput       | ≥ 1 M hands/s (single) |
| Memory / connection        | ≤ 8 MB avg             |

## 7  Security Design
* Defense-in-depth: network (Firewalld) → SSH layer → App validation.
* All inputs validated with `validator` crate; zero `unsafe` in codebase.
* Anti-cheat: audit trail, behaviour anomaly ML (planned v1.1).
* Secrets managed via environment + sops-encrypted manifests.

## 8  Deployment Architecture
* Docker images: `ghcr.io/org/pokerd:sha` (Alpine + musl).
* Compose stacks: **dev** (single-node), **prod** (HA Postgres + 3 pokerd replicas behind HAProxy).
* Blue-green upgrades via `docker-swarm`.

## 9  Test Plan Overview
| Layer             | Tooling                  | Coverage Goal |
|-------------------|--------------------------|---------------|
| Unit              | `cargo test` + `proptest`| ≥ 80%         |
| Integration       | `cucumber-rs`            | all epics     |
| Load              | `artillery` (SSH mode)   | 2× target CC  |
| Security          | `cargo-audit` + `zap`    | 0 critical    |

## 10  Open Questions / Future Work
1. Support Rake/House commissions? 
2. GDPR data-export endpoint specs. 
3. Mobile SSH client UX testing. 

---
*End of Specification*
