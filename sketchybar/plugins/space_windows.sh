#!/usr/bin/env bash
source "$(dirname "$0")/aura_dark_colors.sh"
echo "NAME: $NAME"
echo "SENDER: $SENDER"
echo "FOCUSED WORKSPACE: $FOCUSED_WORKSPACE"

# this part handles updating icons when you move an app window to another AEROSPACE workspace
# it also sets the FOCUSED_WORKSPACE variable if SENDER is space_windows_change, since that is NOT
# an AEROSPACE EVENT but a default sketchybar event triggered when a new app window opens
# without setting FOCUSED_WORKSPACE here, nothing below this if…else block will be applied, 
# meaning that icon of new app will not appear. 
# NB! suplicating icons will not be added, i.e if you open 2,3,4 ecc terminal emulators, eg. kitty,
# only one kitty icon will be displayed

if [ "$SENDER" = "aerospace_workspace_change" ]; then
  prevapps=$(aerospace list-windows --format %{app-name} --workspace "$PREV_WORKSPACE" | sort -u)
  if [ "${prevapps}" != "" ]; then
    icon_strip=""
    while read -r app
    do
      icon_strip+=" $($CONFIG_DIR/plugins/icon_map.sh "$app")"
    done <<< "${prevapps}"
    sketchybar --animate tahn 10 --set space."$PREV_WORKSPACE" label="$icon_strip" drawing=on
  else
    sketchybar --animate tahn 10 --set space."$PREV_WORKSPACE" drawing=off
  fi
else
  FOCUSED_WORKSPACE="$(aerospace list-workspaces --focused)"
fi

# The code below applies sketchybar changes when SENDER is space_windows_change AND aerospace_workspace_change
# space_windows_change event is triggered when you open a new app window in the focused workspace
# in case of aerospace_workspace_change event, the previous block updated the PREV_WORKSPACE
# (the workspace that was focused before you moved focus to a new workspace) and this block updates 
# newly focused aerospace workspace

apps=$(aerospace list-windows --format %{app-name} --workspace "$FOCUSED_WORKSPACE" | sort -u)
icon_strip=""
if [ "${apps}" != "" ]; then
  while read -r app
  do
    icon_strip+=" $("$CONFIG_DIR"/plugins/icon_map.sh "$app")"
    icon_color="$AURA_ORANGE"
  done <<< "${apps}"
else
  icon_strip="—"
  icon_color="$AURA_PURPLE"
fi
sketchybar --animate tahn 10 --set space."$FOCUSED_WORKSPACE" label="$icon_strip" label.color="$icon_color" drawing=on