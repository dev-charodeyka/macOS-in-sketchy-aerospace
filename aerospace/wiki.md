# How AeroSpace default config must be modified to make SketchyBar workspaces item function properly

### 1️⃣ Recap of SketchyBar workspaces item functionality in my setup
- Shows only active (non-empty) AeroSpace workspaces
- Each workspace element displays icons of apps whose windows are open in that active workspace
- No duplicate app icons: if you open 3 Kitty terminals in the same workspace, only one Kitty icon is displayed
- Styles the focused workspace differently
- Allows changing the focused workspace by mouse click

### 2️⃣ Some terminology
"Focused workspace" = the AeroSpace workspace where you currently are (i.e., where the app you’re interacting with is located).  
AeroSpace workspaces are virtual workspaces—they do **not** correspond to macOS’s native virtual desktops. Technically, all your active AeroSpace workspaces (e.g., Workspaces 1–9) live on the **same** macOS virtual desktop. 

That means:
- You can’t use Mission Control features to switch between AeroSpace workspaces.
- If you try to use more than one native macOS virtual desktop, AeroSpace will treat them as a single, extended horizontal desktop, and window sizing/position will be messed up.

Why? Because MacOS isn’t very permissive with virtual desktop management outsorcing to third-party tools. AeroSpace would need elevated privileges to fully control native desktops, so it took a different approach: all AeroSpace workspaces live "inside" one macOS space.

### 3️⃣ About tiling vs. floating windows
Opting for a tiling window manager means you’re happy with tiled windows - you are not a fan of free resizing/moving windows and you prefer  automatic layouts. AeroSpace, however, gives you the flexibility to switch any window's state from "tiled" to "floating", making it behave like a normal macOS window whenever you want.

### 4️⃣ About native keyboard shortcuts
- I use macOS-native shortcuts: `Cmd + Q` (quit) and `Cmd + W` (close window).
- All other shortcuts are custom-defined in my AeroSpace config.

### 5️⃣ Starting AeroSpace on login
In AeroSpace’s config, you might see that I don’t start it on login. In reality, I **do** launch AeroSpace at startup, but I set that up through macOS’s native System Settings. This is because macOS will anyway ask to tick in system settings an app that wants to autostart.

### 6️⃣ About AeroSpace stability
AeroSpace is just an app; it doesn’t replace any core system components. If AeroSpace crashes, you’ll simply fall back to macOS’s native window manager. Interestingly, AeroSpace has a built-in mechanism: if it’s about to crash, it drops all windows together in one ("exited") workspace, so you don’t have to search for windows. From my experience, AeroSpace has never crashed unexpectedly.

### 7️⃣ Why SketchyBar needs custom config for AeroSpace
If you look at SketchyBar’s default config, it’s designed for macOS’s native Mission Control desktops. [AeroSpace’s "goodies" documentation](https://nikitabobko.github.io/AeroSpace/goodies#show-aerospace-workspaces-in-sketchybar) includes shell scripts to configure SketchyBar for AeroSpace workspaces. With this config, you will only see numbered workspaces (all available) with some styling for focused workspace and click to change focus. To get the advanced functionality I listed above in this wiki you need a more advanced config. 

This advanced config, from AeroSpace side, it boils down to force AeroSpace to pass some environment variables to SketchyBar that contain AeroSpace workspaces’ states: which workspace is focused, which was focused previously, which apps are open, and so on. By reading those variables, and reacting on change in AeroSpace worksapces, SketchyBar can dynamically render desired functionality. 

### 8️⃣ Aerospace config updates to make Sketchybar functionality from section :one: work:
    
    after-startup-command = [
    "exec-and-forget sketchybar"]

    exec-on-workspace-change = ['/bin/bash', '-c',
    'sketchybar --trigger aerospace_workspace_change FOCUSED_WORKSPACE=$AEROSPACE_FOCUSED_WORKSPACE PREV_WORKSPACE=$AEROSPACE_PREV_WORKSPACE'
    ]

It’s exactly the second change that’s very important for SketchyBar. SketchyBar should know the current focused AeroSpace workspace any time it queries it, and the previous workspace needs to be updated when you change focus—especially when you move an app from one workspace to another via shortcut. Speaking of that, it’s crucial to add the `--focus-follows-window` flag to every command that binds a keyboard shortcut for the move functionality.
    
    alt-shift-1 = 'move-node-to-workspace 1 --focus-follows-window'
    alt-shift-2 = 'move-node-to-workspace 2 --focus-follows-window'
    alt-shift-3 = 'move-node-to-workspace 3 --focus-follows-window'
    alt-shift-4 = 'move-node-to-workspace 4 --focus-follows-window'
    alt-shift-5 = 'move-node-to-workspace 5 --focus-follows-window'
    alt-shift-6 = 'move-node-to-workspace 6 --focus-follows-window'
    alt-shift-7 = 'move-node-to-workspace 7 --focus-follows-window'
    alt-shift-8 = 'move-node-to-workspace 8 --focus-follows-window'
    alt-shift-9 = 'move-node-to-workspace 9 --focus-follows-window'

Another useful custom shortcut is exactly for changing the state of any window to floating and back to tiling - toggling tiled state:

    alt-ctrl-f = 'layout floating tiling'

### 9️⃣ If you want to have the CPU&GPU temperature graph on your SketchyBar

First, it is important that you follow the instructions in the wiki file located in the iokit_rust directory. That directory contains source code in Rust which generates a stream of CPU&GPU temperature data. You need to build a binary from this code, store the compiled binary wherever you like and make it executable: with `chmod +x /path/to/your/binary`

Then, in AeroSpace config you can configure the execution of this binary before the launch of SketchyBar. It will run in the background as a normal process until it is killed - either on shutdown or manually.

    after-startup-command = [
     #add this:
     "exec-and-forget /Users/alisa/.config/sketchybar/helpers/temp-gpu-cpu/bin/temp-gpu-cpu", 
     
     "exec-and-forget sketchybar"
     ] 
