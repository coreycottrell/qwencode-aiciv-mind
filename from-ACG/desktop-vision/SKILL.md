---
name: desktop-vision
description: See and control the desktop via vision + automation. Use for GUI testing, autonomous app interaction, screenshot analysis, and any task requiring visual understanding of the screen.
---

# Desktop Vision & Automation Skill

**Version**: 1.0
**Date**: 2025-12-16
**Status**: Production-ready

Complete reference for vision-guided desktop automation in WSL2/Windows environment.

---

## When to Use This Skill

**Trigger scenarios:**
- Taking screenshots to see current screen state
- Clicking on GUI elements (buttons, menus, taskbar)
- Typing text into applications
- Pressing keyboard shortcuts
- Navigating desktop applications
- GUI testing and verification
- Any task requiring visual understanding of screen content

---

## Two Available Systems

### 1. MCP Desktop Automation (Preferred for simple actions)

Direct tools available via MCP:

| Tool | Purpose | Example |
|------|---------|---------|
| `mcp__desktop-automation__screen_capture` | Capture current screen | Returns image for vision |
| `mcp__desktop-automation__get_screen_size` | Get dimensions | Returns {width, height} |
| `mcp__desktop-automation__mouse_move(x, y)` | Move mouse | x=500, y=300 |
| `mcp__desktop-automation__mouse_click(button, double)` | Click mouse | button="left", double=false |
| `mcp__desktop-automation__keyboard_type(text)` | Type text | text="Hello world" |
| `mcp__desktop-automation__keyboard_press(key, modifiers)` | Press key | key="enter", modifiers=["control"] |

**Limitation**: `screen_capture` may not return visible images in some contexts.

### 2. PowerShell Workaround (Reliable screenshots)

Use when MCP screenshots don't work:

```bash
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
```

Returns: `/mnt/c/temp/claude_screenshots/screenshot_YYYYMMDD_HHMMSS_mmm.png`

Then read the image:
```bash
Read tool: /mnt/c/temp/claude_screenshots/screenshot_20251216_135715_495.png
```

---

## Recommended Workflow

### Step 1: Take Screenshot

```bash
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
```

### Step 2: View Screenshot

Use Read tool on the returned path to see the screen.

### Step 3: Identify Coordinates

From the screenshot, identify pixel coordinates of target element.

### Step 4: Perform Action

Use MCP tools for interaction:

```
mcp__desktop-automation__mouse_move(x=500, y=300)
mcp__desktop-automation__mouse_click()
```

Or for keyboard:

```
mcp__desktop-automation__keyboard_press(key="t")
mcp__desktop-automation__keyboard_type(text="/teleport 0 20 0")
mcp__desktop-automation__keyboard_press(key="enter")
```

### Step 5: Verify Result

Take another screenshot to confirm action worked.

---

## The autonomous_control.py Tool

**Location**: `${ACG_ROOT}/tools/autonomous_control.py`

### Available Commands

```bash
# Screenshot (most reliable)
python3 tools/autonomous_control.py screenshot

# Click at coordinates
python3 tools/autonomous_control.py click <x> <y>

# Type text
python3 tools/autonomous_control.py type "text here"

# Press key
python3 tools/autonomous_control.py keypress <key>
```

### Python API

```python
import sys
sys.path.insert(0, '${ACG_ROOT}/tools')
from autonomous_control import DesktopController

controller = DesktopController()

# Get screen size
width, height = controller.get_screen_size()

# Take screenshot
screenshot_path = controller.take_screenshot()

# Move mouse
controller.mouse_move(500, 300)

# Click
controller.mouse_click('left', double=False)

# Type text
controller.keyboard_type("Hello world")

# Press key
controller.keyboard_press('ENTER')
controller.keyboard_press('F11')
controller.keyboard_press('TAB')
```

---

## Window Management

### Find and Focus Window (PowerShell)

```bash
powershell.exe -Command "
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport(\"user32.dll\")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport(\"user32.dll\")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport(\"user32.dll\")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport(\"user32.dll\")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
'@

[Win32]::EnumWindows({
    param(\$hwnd, \$lparam)
    \$title = New-Object System.Text.StringBuilder 256
    [Win32]::GetWindowText(\$hwnd, \$title, 256)
    if (\$title.ToString() -like '*WindowTitle*') {
        Write-Host \"Found: \$(\$title.ToString())\"
        [Win32]::ShowWindow(\$hwnd, 3)  # SW_MAXIMIZE = 3
        [Win32]::SetForegroundWindow(\$hwnd)
    }
    return \$true
}, [IntPtr]::Zero)
"
```

Replace `*WindowTitle*` with the target window name pattern (e.g., `*Minetest*`).

---

## Key Codes Reference

| Key | Code |
|-----|------|
| Enter | `enter` |
| Escape | `escape` |
| Tab | `tab` |
| Space | `space` |
| Arrow keys | `up`, `down`, `left`, `right` |
| Function keys | `f1` through `f12` |
| Letters | `a` through `z` |
| Numbers | `0` through `9` |

### Modifiers

```
mcp__desktop-automation__keyboard_press(key="c", modifiers=["control"])  # Ctrl+C
mcp__desktop-automation__keyboard_press(key="v", modifiers=["control"])  # Ctrl+V
mcp__desktop-automation__keyboard_press(key="tab", modifiers=["alt"])    # Alt+Tab
```

---

## Screen Coordinate System

- Origin (0, 0) is top-left corner
- X increases to the right
- Y increases downward
- Typical screen: 1920x1080 or similar

To find coordinates:
1. Take screenshot
2. View in image viewer
3. Note pixel position of target

---

## Troubleshooting

### MCP screen_capture returns no image?

Use PowerShell workaround:
```bash
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
```

### Clicks not registering?

1. Verify window is in foreground
2. Use PowerShell window focus command first
3. Add small delays between actions

### Keyboard not working?

1. Ensure target window has focus
2. Try clicking on window first
3. Use MCP tools instead of Python controller

### Window not found?

Check exact window title:
```bash
powershell.exe -Command "Get-Process | Where-Object {\$_.MainWindowTitle} | Select-Object ProcessName, MainWindowTitle"
```

---

## Best Practices

1. **Always screenshot first** - Verify screen state before acting
2. **Screenshot after actions** - Confirm actions worked
3. **Focus window before typing** - Click window or use PowerShell focus
4. **Small delays** - Add `sleep 0.5` between rapid actions
5. **Use MCP tools for simple clicks/keys** - More reliable
6. **Use Python controller for screenshots** - More reliable image capture

---

## Integration with Other Skills

This skill pairs well with:
- **luanti-ipc** - Control Minetest bots + see game via vision
- **browser-vision** - Web automation via Playwright MCP

When combining vision with game control:
1. Use vision to see game state
2. Use IPC to control bots
3. Use keyboard for player commands
