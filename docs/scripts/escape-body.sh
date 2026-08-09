#!/bin/bash
# Escaping free text for the notification body — copy escape_body() into your sender.
#
# The body is parsed as HTML (Freedesktop spec), and this daemon decodes entities,
# so any text you did not author as markup must be escaped before it goes in.
# Get it wrong in either direction and it shows: escape too little and stray tags
# vanish or turn text bold; escape too much and "&amp;" reaches the screen.
#
# Order matters: `&` MUST be replaced first, otherwise it re-escapes the
# ampersands introduced by the later replacements ("<" -> "&lt;" -> "&amp;lt;").
#
# The backslashes are load-bearing, do not "clean them up": since bash 5.2
# (patsub_replacement, on by default) an unquoted `&` in the replacement half of
# ${var//pat/rep} stands for the text the pattern matched, so `${s//</&lt;}`
# yields "<lt;", not "&lt;". `\&` is the documented escape for it, and the
# backslash is removed during quote removal, so the same lines are also correct
# on pre-5.2 bash where `&` had no special meaning. Verified on 5.2.21.
escape_body() {
  local s=$1
  s=${s//&/\&amp;}
  s=${s//</\&lt;}
  s=${s//>/\&gt;}
  s=${s//\"/\&quot;}
  s=${s//\'/\&apos;}
  printf '%s' "$s"
}

notify() {
  gdbus call --session --dest org.freedesktop.Notifications \
    --object-path /org/freedesktop/Notifications \
    --method org.freedesktop.Notifications.Notify \
    "escape-demo" 0 "dialog-information" "$1" "$2" \
    "[]" "{'urgency': <byte 2>}" 15000
}

# Text no sender controls: a diff hunk, a shell command, a log line.
RAW='patch <file.rs & git commit -m "fix <T> & <U>"'

# Wrong: raw text straight into the body. Everything from `<file.rs` through the
# first `>` is eaten as a tag, `<U>` is eaten as another, and what reaches the
# screen is:   patch  & "
notify "Unescaped (broken)" "$RAW"

sleep 1

# Right: every metacharacter escaped, so the command renders verbatim — and the
# markup you authored yourself still works, because the text is escaped BEFORE
# being concatenated with your own tags. Renders as:   ran: patch <file.rs & git
# commit -m "fix <T> & <U>"   with "ran:" in bold.
notify "Escaped (correct)" "<b>ran:</b> $(escape_body "$RAW")"
