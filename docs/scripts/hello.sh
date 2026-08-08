#!/bin/bash
# Minimal notification. Prints the assigned notification id.
gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "${1:-hello-demo}" 0 "dialog-information" \
  "${2:-Hello World}" "${3:-A minimal cosmic-notifications test}" \
  "[]" "{}" 5000
