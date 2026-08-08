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

- `expire_timeout` is clamped in `app.rs` (`fn update`, Notify arm): max 5000 ms normal,
  3000 ms low urgency; **uncapped for urgency=2**. `0` = persistent, `-1` = 3000 ms.
- Body markup supports only `<b> <i> <u> <a> <br>`. `<a>` renders styled but is **not clickable**
  — the span never gets the href and rich-text events map to `Message::Ignore`. Card click fires
  the `default` action instead (via XDG activation token + `ActionInvoked` signal).
- No per-action buttons on popup cards: one click target per notification.
- `image-path` hint must be a `file://` URL (bare paths are treated as theme-icon names); images
  render at 16 px next to the app name. `<img>` tag is an upstream TODO.
- Test loop: `pkill cosmic-notifications && ./target/release/cosmic-notifications` in a terminal
  (live tracing logs). Kill it twice quickly if cosmic-session keeps respawning it. Log out/in
  restores stock behavior.
- Deploying (`sudo just install`) overwrites the apt-managed binary; `apt upgrade` will clobber it
  back. Use `sudo apt-mark hold cosmic-notifications` while running a custom build.

## Workflow (mandatory)

1. **Discuss first.** Every feature/fix/refactor idea is discussed with Jorge before work starts.
2. **Create an issue** for it — but first scan open issues. Open-issue awareness matters: comment
   on related issues when genuinely relevant (including "closing as delivered by #N" comments),
   never as noise.
3. **Branch off `master`**, work on the PR. Reference the issue.
4. **Before pushing**: update any docs affected by the change (`README.md`, `docs/`), then run
   local CI (below). Never open a PR with failing CI.
5. **Wait for code review.** Be critical of it — push back on points you disagree with; commit
   only the changes you agree with. Before that commit, refresh docs made stale by the review and
   update/refactor this CLAUDE.md concisely, only if necessary.
6. **Merge only with explicit authorization from Jorge.** No exceptions.

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

- Make `<a href>` hyperlinks actually clickable (markup.rs spans + app.rs message wiring).
- Implement `<img>` tag rendering (upstream TODO in markup.rs).
- Render buttons for non-default actions on popup cards.
