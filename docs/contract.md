# D-Bus Contract — how to send notifications to this daemon

Reference for any tool that wants to display notifications on this system. Everything below was
tested live against the daemon (spec 1.2, server name `cosmic-notifications`). Runnable examples:
[scripts/](scripts/).

Bus: **session** · Name: `org.freedesktop.Notifications` · Path: `/org/freedesktop/Notifications`
· Interface: `org.freedesktop.Notifications`

## Capabilities (as advertised by `GetCapabilities`)

`body`, `icon-static`, `persistence`, `actions`, `action-icons`, `body-markup`,
`body-hyperlinks`, `sound`

## Notify

```
Notify(app_name s, replaces_id u, app_icon s, summary s, body s,
       actions as, hints a{sv}, expire_timeout i) → (id u)
```

```sh
gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "my-app" 0 "dialog-information" "Title" "<b>bold</b> body" \
  "[]" "{}" 5000
```

- `replaces_id`: pass a previous id to update that notification in place; `0` for new.
- `app_icon`: theme icon name, or a `file://` URL.
- Summary renders **first line only**. Body renders at 12 px below it.

## expire_timeout semantics (verified)

| Value | Behavior |
|---|---|
| `> 0` | Honored, but clamped to a per-urgency cap (below) |
| `0` | **Persistent** — stays until clicked/dismissed |
| `-1` | Server default: 3000 ms |

| Urgency (hint) | Default max display time |
|---|---|
| `0` low | 3000 ms |
| `1` normal (default) | 5000 ms |
| `2` critical | **uncapped** — your value used as-is |

To exceed 5 s, send `urgency: <byte 2>`. Caps are user-configurable (see architecture.md §Config).

## Body markup

HTML subset only (Freedesktop spec — **not** Markdown): `<b>`, `<i>`, `<u>`, `<a href="…">`,
`<br>`. Anything else is stripped/ignored (`<img>` not implemented).

⚠️ `<a>` renders underlined in the accent color but is **not clickable**. To make "click opens a
URL" work, attach a `default` action and handle `ActionInvoked` (below).

## Hints

| Hint | Type | Notes |
|---|---|---|
| `urgency` | `y` | 0/1/2 — affects timeout cap, max_per_app bypass |
| `image-path` | `s` | **must be `file://` URL**; bare string = theme-icon name. Renders 16 px |
| `image-data` | `(iiibiiay)` | raw image struct; also 16 px |
| `sound-name` / `sound-file` | `s` | freedesktop sound name / file path |
| `suppress-sound`, `transient`, `resident`, `category`, `desktop-entry`, `action-icons` | | per spec |

## Actions and click handling

The popup card has **one click target** (the whole card) and renders **no action buttons**.
A click invokes: the `default` action if present → else the first action → else it just dismisses.

The daemon never launches anything. On click it emits:

1. `ActivationToken(id u, token s)` — XDG activation token so the app you launch gets focus.
2. `ActionInvoked(id u, action_key s)`.

**Your process must be alive and subscribed** to act on these. Fire-and-forget senders get
dismiss-on-click only. Minimal shell pattern (full version: `scripts/click-to-open.sh`):

```sh
gdbus monitor --session --dest org.freedesktop.Notifications   # watch for ActionInvoked (uint32 <id>, 'default')
```

Real apps should use libnotify / zbus / Gio, which handle the subscription for you.

## Other methods and signals

- `CloseNotification(id)` — programmatic close.
- `NotificationClosed(id, reason)` — reasons: 1 expired, 2 dismissed, 3 CloseNotification,
  4 undefined.
- `GetServerInformation()` → `("cosmic-notifications", "System76", "0.1.0", "1.2")`.

## Scripts

| Script | Demonstrates |
|---|---|
| [scripts/hello.sh](scripts/hello.sh) | Minimal Notify call |
| [scripts/markup-test.sh](scripts/markup-test.sh) | b/i/u/a markup rendering |
| [scripts/persistent.sh](scripts/persistent.sh) | `expire_timeout=0` + critical urgency |
| [scripts/click-to-open.sh](scripts/click-to-open.sh) | default action + ActionInvoked listener → launches an app |
