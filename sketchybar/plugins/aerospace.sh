#!/usr/bin/env bash

# make sure it's executable with:
# chmod +x ~/.config/sketchybar/plugins/aerospace.sh
source "$(dirname "$0")/aura_dark_colors.sh"

if [[ "$1" = "$FOCUSED_WORKSPACE" ]]; then
  sketchybar --animate tahn 10 --set "$NAME" background.border_color="$AURA_ORANGE" icon.color="$AURA_ORANGE" background.shadow.color="$AURA_ORANGE" background.shadow.distance=1
  #echo "FOCUSED"
  #echo "$NAME"
else
  #echo "NOT FOCUSED"
  #echo "$NAME"
  sketchybar --animate tahn 10 --set "$NAME" background.border_color="$AURA_GREEN" background.shadow.color="$AURA_GREEN" background.shadow.distance=3 icon.color="$AURA_PURPLE"
fi
