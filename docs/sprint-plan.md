# Sprint plan (S1-S21)

This is the [roadmap](./roadmap.md) translated into a realistic personal-project
schedule. The roadmap says *what* each milestone delivers and *why*; this page
says *when* - sized to the time actually available.

## The cadence

- **One sprint per week.** Each sprint is a small, self-contained step.
- **Three to five focused hours per sprint.** This is a project worked in
  evenings and weekends, not full-time, so the plan is honest about that budget
  rather than pretending otherwise.
- **Roughly one issue per sprint.** The board's "one issue = one branch = one PR"
  unit maps cleanly onto a week; a few larger issues span two sprints, and a few
  small ones share a week.
- **Sequenced to reach a playable game first, then harden.** The path to playable
  (M3-M7) comes before the hardening work (M8-M12), so there is something real to
  play with as early as possible - while testing, CI, and security are
  *interleaved* along the way rather than bolted on at the end.

## How to use this

1. **Pick the current sprint** from the tables below.
2. **Work its issue(s)** on a branch named `m<phase>/<slug>` (for example,
   `m3/apply-action-rpc`), following the [development guide](./development.md).
3. **Open a PR that `Closes` the issue**, and let CI run before merging.

The estimates are deliberately modest; treat a sprint as "done when" its
acceptance line is true, not when the hours run out. If a sprint finishes early,
pull the next one forward; if it runs long, let it spill into the following week
rather than cutting the "done when" bar.

## Path to playable (M3-M7)

| Sprint | Focus | Issues | Est. | Done when |
| --- | --- | --- | --- | --- |
| S1 | Engine service scaffold + state blob round-trip | #6 | 3-4h | tonic server runs; GameState serializes through the blob |
| S2 | RPCs: NewHand + ApplyAction | #7 | 4-5h | a hand can start and accept an action over gRPC |
| S3 | RPCs: GetValidActions + CompleteHand | #7 | 3-4h | full action set + payout exposed |
| S4 | Domain<->proto mapping + per-seat TableView | #8 | 4-5h | each seat gets a correct view (own hole cards only) |
| S5 | engine-service tests | #9 | 3-4h | a full hand is driven through the service in tests |
| S6 | Wish SSH server + Bubble Tea middleware | #10 | 4-5h | ssh into the gateway shows a TUI |
| S7 | gRPC client + smoke test, then Go CI | #11, #22 | 4-5h | a keypress hits the engine; Go CI green |
| S8 | Name-prompt view | #12 | 3h | guest name captured on connect |
| S9 | Lobby view | #13 | 4-5h | list / create / join tables |
| S10 | Game/table view | #14 | 5h | table, board, pot, seats render |
| S11 | Game view polish + gateway unit tests | #14, #25 | 4h | view stable; mapping/table-manager unit-tested |
| S12 | Table manager (authoritative state per table) | #15 | 4-5h | one blob per table; seating works |
| S13 | Turn enforcement + fan-out via program.Send | #16 | 5h | an action updates all seated players |
| S14 | Disconnect handling + 2-session integration test | #17, #33 | 4-5h | leaves free seats; two sessions sync |
| S15 | Playable: two humans, full hand + gRPC integration | #18, #31 | 3-4h | a complete hand played over SSH |

**Milestone reached: a playable game.** That is roughly **15 weeks (~3.5 months)
and ~55 hours** of focused work to the point where two humans can play a full
hand over SSH.

## Harden and ship (M8-M12)

| Sprint | Focus | Issues | Est. |
| --- | --- | --- | --- |
| S16 | Host key + config; Docker + compose + Makefile | #19, #20 | 5h |
| S17 | Run docs; compose integration env; SSH e2e harness | #21, #34, #32 | 5h |
| S18 | CI/CD: codegen-drift check, unified pipeline, release images | #27, #28, #29 | 5h |
| S19 | Security: SSH hardening + rate limiting | #35, #36 | 4-5h |
| S20 | Security: dependency scanning, gRPC boundary/secrets, fuzzing | #37, #40, #38 | 5h |
| S21 | Coverage + edge tests, CD, auth design, threat model | #23, #24, #26, #30, #39, #41 | 5h (buffer) |

## Closing note

End to end, the plan runs about **21 weeks (~5 months) at 3-5 hours per week**,
with the game **playable at roughly S15** and the remaining sprints turning it
into something deployable and defensible.

A deliberate point about ordering: **testing, CI, and security are interleaved,
not deferred to the end.** Engine tests land with the engine (S5), Go unit tests
land with the views (S11), integration tests land with multiplayer (S14-S15),
and CI/security fill the back half - but the *practices* are present from the
first sprint. This mirrors the [testing strategy](./adr/0006-testing-strategy.md)
and the [Phase 0 lesson](./adr/0001-stabilize-the-rust-baseline.md): a green,
trustworthy baseline is maintained continuously, not reconstructed at the finish
line.

## Related reading

- [Roadmap](./roadmap.md) - the milestone-by-milestone narrative these sprints
  implement.
- [Development guide](./development.md) - the branch/PR workflow each sprint uses.
- [Documentation hub](./README.md) - everything else.
