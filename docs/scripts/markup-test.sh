#!/bin/bash
# Shows the full supported markup subset: b, i, u, a, br — plus HTML entity decoding.
#
# The <a> link is clickable: it opens in the default browser (xdg-open) and the
# click does NOT trigger the card's action or dismiss the card. Only http/https/
# mailto links are clickable; a file:// link renders as plain, inert text.
#
# Escaped text (&lt; &amp; …) is decoded after tag parsing, so "&lt;b&gt;" shows
# as literal "<b>" instead of turning bold.
#
# urgency byte 2 (critical) is required for the 15s timeout to survive: normal
# urgency is capped at 5s by max_timeout_normal, which is not enough time to
# read the body and click the link.
gdbus call --session --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "markup-demo" 0 "dialog-information" \
  "Markup test" \
  "<b>bold</b> <i>italic</i> <u>underline</u><br><a href=\"https://github.com/pop-os/cosmic-notifications\">clickable link</a> · <a href=\"file:///etc/passwd\">inert file link</a><br>entities: &lt;b&gt;not bold&lt;/b&gt; &amp; Q&amp;A &quot;quoted&quot; &#60;numeric&#62;" \
  "[]" "{'urgency': <byte 2>}" 15000
