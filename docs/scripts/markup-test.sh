#!/bin/bash
# Shows the full supported markup subset: b, i, u, a, br.
# Note: the <a> link renders styled (accent + underline) but is NOT clickable.
gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "markup-demo" 0 "dialog-information" \
  "Markup test" \
  "<b>bold</b> <i>italic</i> <u>underline</u><br><a href=\"https://github.com/pop-os/cosmic-notifications\">a styled link</a>" \
  "[]" "{}" 5000
