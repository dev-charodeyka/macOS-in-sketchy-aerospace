#!/usr/bin/env bash
#ram.sh

sketchybar -m --set "$NAME" label="$(memory_pressure | grep "System-wide memory free percentage:" | awk '{ printf("%.0f\n", 100-$5"%") }')%"