# Architecture

Detailed component map of the daemon. For the external D-Bus usage contract see
[contract.md](contract.md).

## Process model

`cosmic-session` spawns `cosmic-notifications` at login and respawns it if it dies. The daemon:

1. Claims `org.freedesktop.Notifications` on the session bus (zbus).
2. Renders popup cards as Wayland **layer-shell** surfaces via iced/libcosmic.
3. Streams every notification to the panel's notifications applet over a Unix socket pair, so the
   applet's history/tray stays in sync. The FDs are exchanged through the env vars
   `PANEL_NOTIFICATIONS_FD` / `DAEMON_NOTIFICATIONS_FD` (see `cosmic-notifications-config`).

## Crate / file map

| Path | Role |
|---|---|
| `src/main.rs` | Entry point, tracing setup, launches the iced app |
| `src/app.rs` | `CosmicNotifications` app: live cards (`cards`) + expired archive (`hidden`, max 200), popup layout, timeout scheduling, config watching, activation flow |
| `src/subscriptions/notifications.rs` | zbus interface: `Notify`, `CloseNotification`, `GetCapabilities`, `GetServerInformation`; emits `ActionInvoked`, `NotificationClosed`, `ActivationToken` |
| `src/subscriptions/applet.rs` | Unix-socket p2p connection to the panel applet (`com.system76.NotificationsSocket`) |
| `cosmic-notifications-util/src/lib.rs` | `Notification`, `Hint` parsing (urgency, image-path/data, sound hints…), `Image`, `ActionId`, `CloseReason` |
| `cosmic-notifications-util/src/markup.rs` | `html_to_spans()` — body markup → iced rich-text spans (`tl` parser; tags b/i/u/a/br), HTML entity decoding, `href` sanitizing (http/https/mailto only) |
| `cosmic-notifications-config/src/lib.rs` | `NotificationsConfig` (cosmic-config, ID `com.system76.CosmicNotifications`, version 1) |

## Notification lifecycle

1. Client calls `Notify` over D-Bus → `Notification::new()` parses hints into typed `Hint`s.
2. `app.rs` computes the display timeout: requested `expire_timeout` clamped by per-urgency config
   caps (`max_timeout_low/normal/urgent`; `None` cap = uncapped). `timeout == 0` → no expiry task
   (persistent). A tokio sleep task later fires `Message::Timeout(id)`.
3. Card renders: 16 px icon (image hint or app_icon) + app name + close button, summary (first
   line only), body via `rich_text(html_to_spans(body))`. At most `max_notifications` (3) popups
   exist at a time — extra `cards` simply get no popup. `max_per_app` (2) is only a *reordering*
   pass (`group_notifications`): per-app overflow is moved to the back of `cards`, not removed
   and not hidden, and despite the config doc comment urgency does not exempt a notification
   from it. `hidden` holds only already-expired notifications (for nothing but bookkeeping).
4. Click on card → `Message::ActivateNotification` → request XDG activation token →
   `ActivationToken` signal + `ActionInvoked(id, action)` signal → card dismissed. Action chosen:
   the clicked action if valid, else `default` if present, else the first action, else the click
   just dismisses.
   Click on a body hyperlink instead → `Message::OpenLink` → own activation token request
   (`Message::LinkActivationToken`) → `xdg-open <url>` with `XDG_ACTIVATION_TOKEN` in its env.
   The rich-text widget captures the press, so the card's own click handler never runs: no
   `ActionInvoked`, no dismissal, no D-Bus traffic at all.
5. `NotificationClosed(id, reason)` emission (verified live, spec reasons are 1 expired,
   2 dismissed, 3 closed by call, 4 undefined):
   - **expiry** (`fn expire`) emits **nothing** — the card just moves to `hidden`;
   - **dismissal / click** emits reason **2**;
   - **`CloseNotification`** emits **two** signals, reason 3 then reason 2.

   Reasons 1 and 4 are never sent: `fn close` builds the `Input::Closed(id, reason)` send future
   inside `tokio::spawn` but drops it without `.await` (`_ = sender.send(...)`), so only the
   separate `Input::Dismissed` path ever reaches the bus. Upstream bug, unfixed here.

## Config

Stored per cosmic-config conventions under
`~/.config/cosmic/com.system76.CosmicNotifications/v1/<field>` (RON values, one file per field).
Defaults: `do_not_disturb: false`, `anchor: Top`, `max_notifications: 3`, `max_per_app: 2`,
`max_timeout_urgent: None`, `max_timeout_normal: Some(5000)`, `max_timeout_low: Some(3000)`.
Example override: write `Some(10000)` into `.../v1/max_timeout_normal`.
