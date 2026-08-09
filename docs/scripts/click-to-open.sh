#!/bin/bash
# Click-to-open: sends a notification with a "default" action and launches a
# command when the card is clicked.
#
# Usage: ./click-to-open.sh [command...]
#   default command: xdg-open https://github.com/pop-os/cosmic-notifications
#
# How it works (see docs/contract.md): for actions the daemon launches nothing
# — body hyperlinks are the one thing it opens itself, and this card has none.
# Clicking the card emits ActionInvoked(id, "default") on the session bus; a
# listener must be alive to react. This script starts the listener BEFORE
# sending, to avoid missing a fast click.

CMD=("${@:-}")
[ ${#CMD[@]} -eq 0 ] || [ -z "${CMD[0]}" ] && CMD=(xdg-open https://github.com/pop-os/cosmic-notifications)

LOG=$(mktemp)
trap 'rm -f "$LOG"' EXIT
timeout 35 gdbus monitor --session --dest org.freedesktop.Notifications > "$LOG" &
MONPID=$!
sleep 0.5

ID=$(gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "click-demo" 0 "web-browser" \
  "Click me" "Clicking this card runs: <b>${CMD[*]}</b>" \
  "['default', 'Open']" "{'urgency': <byte 2>}" 30000 | grep -oP '(?<=uint32 )\d+')

if [ -z "$ID" ]; then
  echo "Notify failed (no id returned) — is cosmic-notifications running?" >&2
  kill "$MONPID" 2>/dev/null
  exit 1
fi

# The daemon emits no NotificationClosed when a card expires on its own (see
# docs/contract.md), so the 35s monitor is the real upper bound here, not the
# 30s expire_timeout.
echo "notification id: $ID — waiting up to 35s for a click..."

RESULT=timeout
while kill -0 "$MONPID" 2>/dev/null; do
  if grep -q "ActionInvoked (uint32 $ID," "$LOG"; then
    RESULT=clicked; kill "$MONPID" 2>/dev/null; break
  fi
  if grep -q "NotificationClosed (uint32 $ID," "$LOG"; then
    RESULT=closed; kill "$MONPID" 2>/dev/null; break
  fi
  sleep 0.3
done

if [ "$RESULT" = clicked ]; then
  echo "clicked -> launching: ${CMD[*]}"
  setsid "${CMD[@]}" >/dev/null 2>&1 &
else
  echo "result: $RESULT (no action invoked)"
fi
