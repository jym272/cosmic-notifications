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
| `src/app.rs` | `CosmicNotifications` app: card state (`cards` / `hidden`), popup layout, timeout scheduling, activation flow |
| `src/subscriptions/notifications.rs` | zbus interface: `Notify`, `CloseNotification`, `GetCapabilities`, `GetServerInformation`; emits `ActionInvoked`, `NotificationClosed`, `ActivationToken` |
| `src/subscriptions/` (others) | panel socket, config watcher, freedesktop sound |
| `cosmic-notifications-util/src/lib.rs` | `Notification`, `Hint` parsing (urgency, image-path/data, sounds…), `Image`, `ActionId`, `CloseReason` |
| `cosmic-notifications-util/src/markup.rs` | `html_to_spans()` — body markup → iced rich-text spans (`tl` parser; tags b/i/u/a/br) |
| `cosmic-notifications-config/src/lib.rs` | `NotificationsConfig` (cosmic-config, ID `com.system76.CosmicNotifications`, version 1) |

## Notification lifecycle

1. Client calls `Notify` over D-Bus → `Notification::new()` parses hints into typed `Hint`s.
2. `app.rs` computes the display timeout: requested `expire_timeout` clamped by per-urgency config
   caps (`max_timeout_low/normal/urgent`; `None` cap = uncapped). `timeout == 0` → no expiry task
   (persistent). A tokio sleep task later fires `Message::Timeout(id)`.
3. Card renders: 16 px icon (image hint or app_icon) + app name + close button, summary (first
   line only), body via `rich_text(html_to_spans(body))`. Max 3 cards on screen, 2 per app
   (urgent notifications bypass `max_per_app`); the rest go to `hidden`.
4. Click on card → `Message::ActivateNotification` → request XDG activation token →
   `ActivationToken` signal + `ActionInvoked(id, action)` signal → card dismissed. Action chosen:
   the clicked action if valid, else `default` if present, else the first action, else the click
   just dismisses.
5. Expiry/dismissal/`CloseNotification` → `NotificationClosed(id, reason)` (1 expired,
   2 dismissed, 3 closed by call, 4 undefined).

## Config

Stored per cosmic-config conventions under
`~/.config/cosmic/com.system76.CosmicNotifications/v1/<field>` (RON values, one file per field).
Defaults: `do_not_disturb: false`, `anchor: Top`, `max_notifications: 3`, `max_per_app: 2`,
`max_timeout_urgent: None`, `max_timeout_normal: Some(5000)`, `max_timeout_low: Some(3000)`.
Example override: write `Some(10000)` into `.../v1/max_timeout_normal`.
