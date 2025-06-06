#!/usr/bin/env bash

# this command’s output is the closest approximation to the CPU usage (%) shown in Activity Monitor
# it would probably be more efficient to use an existing Rust app to fetch CPU usage directly from IOKit,
# but for now, it is what it is

sketchybar -m --set "$NAME" label="$(top -l  2 | grep -E "^CPU" | tail -1 | awk '{ printf("%.0f\n", ($3 + $5)) }')%"
