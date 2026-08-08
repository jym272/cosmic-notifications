#!/bin/bash
# expire_timeout=0 => stays until dismissed/clicked, for any urgency.
# urgency byte 2 (critical) is also the way to exceed the 5s normal-urgency cap
# with a nonzero timeout.
gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "persistent-demo" 0 "dialog-warning" \
  "Persistent notification" "I stay until you dismiss me (expire_timeout=0)" \
  "[]" "{'urgency': <byte 2>}" 0
