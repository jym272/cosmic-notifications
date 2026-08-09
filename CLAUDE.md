# CLAUDE.md — Agent Harness Guide

Fork of [pop-os/cosmic-notifications](https://github.com/pop-os/cosmic-notifications): the COSMIC
notification daemon, customized for this machine (Pop!_OS 24.04 / COSMIC). Owner: Jorge (@jym272).
This file is the operating manual for agents working on this fork. Keep it concise; update it only
when something here becomes wrong or incomplete.

## Architecture (high level)

Rust workspace, three crates:

- **root crate** (`src/`) — the daemon. An iced/libcosmic layer-shell app (`app.rs`) that owns a
  zbus D-Bus server implementing `org.freedesktop.Notifications` (spec 1.2) in
  `src/subscriptions/notifications.rs`. Notifications arrive over D-Bus → rendered as layer-shell
  popup cards → forwarded to `cosmic-panel`'s applet over a Unix socket FD pair
  (`PANEL_NOTIFICATIONS_FD` / `DAEMON_NOTIFICATIONS_FD`).
- **cosmic-notifications-util** — shared types (`Notification`, `Hint`, `Image`, `ActionId`) and
  the body-markup HTML parser (`markup.rs`, uses `tl`).
- **cosmic-notifications-config** — `NotificationsConfig` via cosmic-config
  (ID `com.system76.CosmicNotifications`, v1): anchor, max_notifications (3), max_per_app (2),
  per-urgency max timeouts.

The daemon is spawned and respawned by `cosmic-session`. Full detail: `docs/architecture.md`.
The external D-Bus usage contract (for tooling that *sends* notifications): `docs/contract.md`,
with runnable examples in `docs/scripts/`.

## Verified gotchas

- `expire_timeout` is clamped in `app.rs` (`fn push_notification`): max 5000 ms normal,
  3000 ms low urgency; **uncapped for urgency=2**. `0` = persistent, `-1` = 3000 ms. Replacing a
  notification (`replaces_id`) does **not** restart its timer.
- `NotificationClosed` is only emitted with reason 2 (dismissed) and 3 (`CloseNotification`, which
  emits *both* 3 and 2); expiry emits nothing. `fn close` drops the `Input::Closed` send future
  without awaiting it — upstream bug, see `docs/contract.md`.
- Body markup supports only `<b> <i> <u> <a> <br>`, plus HTML entity decoding (five XML named
  entities + numeric refs; unknown ones stay literal). `<a href>` is clickable — only
  http/https/mailto, other schemes render as inert unstyled text. A link click is captured by the
  rich-text widget, so it never reaches the card: no `ActionInvoked`, no dismissal. Clicking
  elsewhere on the card still fires the `default` action (XDG activation token +
  `ActionInvoked`).
- No per-action buttons on popup cards: one click target per notification.
- `image-path` hint must be a `file://` URL (bare paths are treated as theme-icon names); images
  render at 16 px next to the app name. `<img>` tag is an upstream TODO.
- Test loop: `pkill -x -f cosmic-notifications; RUST_LOG=info ./target/release/cosmic-notifications`
  in a terminal. `RUST_LOG` is required — the tracing default directive is `WARN`, so without it
  the daemon prints essentially nothing (use `debug`/`trace` for the notification flow). Use `;`
  not `&&`: `pkill` exits 1 when nothing matched, which would skip the run. Kill it twice quickly
  if cosmic-session keeps respawning it. Log out/in restores stock behavior.
- `pkill cosmic-notifications` silently matches nothing: the kernel truncates process names to
  15 chars, and the daemon's cmdline is the bare name (no `/usr/bin/` prefix). Always use
  `pkill -x -f cosmic-notifications`. Verify which binary a running daemon executes with
  `ls -l /proc/$(pgrep -f '^cosmic-notifications$')/exe` — ` (deleted)` means it predates the
  last install and needs a restart.
- `sudo just ...` fails with `command not found`: just is installed via linuxbrew, which isn't in
  root's PATH. Use `sudo $(which just) ...`.

## Privileges — agents never run sudo

Agents CANNOT run sudo commands (no terminal for the password prompt — do not attempt it, ever).
Jorge runs everything privileged. Agents may freely run unprivileged commands: builds, `pgrep`/
`pkill` of Jorge's own processes, `apt-mark showhold`, `gh`, the docs/scripts, etc. When a step
needs root, stop and hand Jorge the exact command to run.

## Deployment flow (current state: custom build deployed)

This machine runs the fork's own binary at `/usr/bin/cosmic-notifications`, and the apt package
is frozen (`apt-mark showhold` → `cosmic-notifications`) so OS updates can't overwrite it. Apt is
no longer the update channel — this repo is. The cycle:

1. PR merged into `master` (only ever with Jorge's authorization).
2. Agent: `git checkout master && git pull origin master`, then `just build-release`.
3. Agent verifies the build and **reports to Jorge that it's ready to install** — nothing more.
4. Jorge deploys: `sudo $(which just) deploy` (installs the binary and restarts the daemon;
   cosmic-session respawns it).
5. Agent verifies the switch: `/proc/<pid>/exe` points at a non-deleted binary, and a
   `docs/scripts/hello.sh` notification renders.

## Workflow (mandatory)

1. **Discuss first.** Every feature/fix/refactor idea is discussed with Jorge before work starts.
2. **Create an issue** for it — ALWAYS on this fork
   (`jym272/cosmic-notifications`), NEVER on the upstream community repo
   (`pop-os/cosmic-notifications`); pass `--repo jym272/cosmic-notifications` explicitly to `gh`,
   since the `upstream` git remote makes bare `gh` commands ambiguous. Nothing is ever filed,
   commented, or PR'd upstream without Jorge explicitly asking for it. Before creating, scan open
   issues — open-issue awareness matters: comment on related issues when genuinely relevant
   (including "closing as delivered by #N" comments), never as noise.
3. **Branch off `master`**, work on the PR. Reference the issue.
4. **Before pushing**: update any docs affected by the change (`README.md`, `docs/`), then run
   local CI (below). Never open a PR with failing CI.
5. **Wait for code review.** Be critical of it — push back on points you disagree with; commit
   only the changes you agree with. Before that commit, refresh docs made stale by the review and
   update/refactor this CLAUDE.md concisely, only if necessary.
6. **PR descriptions are TIMELESS.** Whenever a new commit lands on a PR (review fixes, revisions,
   anything), rewrite the description to reflect the PR's current state. Never stack onto it or
   treat it as a historic log — no "due to review, now we…" phrasing. History lives in commits
   and review threads, not the description.
7. **Merge only with explicit authorization from Jorge.** No exceptions.

## Local CI (run before every PR push)

```sh
cargo fmt --check
just check            # clippy --all-features -W clippy::pedantic
just build-release
find . -name "*.desktop" -exec desktop-file-validate {} +   # mirrors upstream's only GH workflow
```

## Upstream sync

- Upstream cuts coordinated COSMIC-wide **epoch tags** (`epoch-1.5.0` is latest); there are no
  GitHub releases. Pop!_OS ships rolling CI builds of **master** (deb versions look like
  `0.1.0~<timestamp>~24.04~<commit>`). We track **master**, matching what the OS ships.
- `master` on this fork is the integration branch: upstream commits merged in + our merged PRs.
- Sync: `git fetch upstream && git merge upstream/master` on a branch, run local CI, open a PR
  like any other change. Expect occasional conflicts in `README.md` (we extend it) — ours keeps
  the fork section, take upstream's changes for the rest.

## Candidate work (agreed direction, each needs an issue + discussion first)

- Implement `<img>` tag rendering (upstream TODO in markup.rs).
- Render buttons for non-default actions on popup cards.
