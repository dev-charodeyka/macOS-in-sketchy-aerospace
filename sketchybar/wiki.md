# How Sketchybar default config must be modified to make SketchyBar workspaces item have following functions:

- Shows only active (non-empty) AeroSpace workspaces
- Each workspace element displays icons of apps whose windows are open in that active workspace
- No duplicate app icons: if you open 3 Kitty terminals in the same workspace, only one Kitty icon is displayed
- Styles the focused workspace differently
- Allows changing the focused workspace by mouse click

### :one: About SketchyBar integration with AeroSpace

The default SketchyBar configuration contains code showing how to configure the "spaces item" to display dynamically virtual desktops in use and react to Mission Control and native macOS virtual desktops actions.

Styling elements with SketchyBar is pretty easy. The most difficult part to implement is visualizing icons for apps whose windows are open in an AeroSpace workspace and showing only active AeroSpace workspaces (non-empty).

SketchyBar has the concept of events, and you can subscribe SketchyBar items to those events. That means the events will trigger the scripts attached to those items.

Let’s take, for example, an easy item like visualizing CPU or RAM load in my setup. There is no subscription to any event, so the `update_freq` is set for these items, meaning their scripts run at the intervals scheduled in `update_freq`.

    sketchybar --add item cpu right \
               --set cpu update_freq=1 \
                 ... \
                 script="$PLUGIN_DIR/cpu.sh" 


Approach with `update_freq` could theoretically also update the "workspaces item" *partially*, though I didn’t test it. Anyway, it’s the least optimal approach since it’s resource inefficient. The solution is to run *scripts* only when a specific workspace-related event occurs. The scripts simply apply styles to the way AeroSpace workspace that has undergone some change is visualized on SketchyBar. For example, when you move between AeroSpace workspace, focus changes, and focues workspace "box" change border and text colors. 

What are events? For example, you click or use a shortcut to go to another workspace - that event changes the FOCUSED_WORKSPACE. You move a window to another workspace using a shortcut; that also changes the FOCUSED_WORKSPACE, but it additionally changes the list of windows in two workspaces: the one you moved from and the one you moved the app window to. All items can subscribe to arbitrary events; when the event happens, all items subscribed to the event will execute their script. 

As I mentioned, SketchyBar reacts well to events related to native MacOS virtual desktops (space_windows_change, space_change events), though by default it knows nothing about AeroSpace WM. However, in the `aerospace.toml` config, you can define a custom AeroSpace event - aerospace_workspace_change. Since AeroSpace workspaces are virtual workspaces all located on 1 native macOS virtual desktop, there is still a default event that’s triggered whenever any app window is opened or closed in that virtual desktop (space_windows_change).

TL;DR:

- You can customize the visualization of AeroSpace workspaces to your taste, but they all belong to a single SketchyBar item.
- To make the workspace item reactive to any AeroSpace workspace-related action you perform, the workspace item should be subscribed to relevant events.
- Subscription in SketchyBar items means each item "listens" for the events it’s subscribed to, and when an event occurs, it triggers the script attached to that item.
- SketchyBar supports default events for native macOS virtual desktops, and you can also add custom events.
- The functionality of my setup boils down to: hide empty workspaces, apply specific styling to new workspaces, update two workspaces when I move an app window from one to another, and update app icons when I open or close app windows in any AeroSpace workspace.

### 2️⃣ aerospace_workspace_change custom event.





