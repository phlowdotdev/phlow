---
sidebar_position: 2
title: Debug
---

# Phlow Debugging

Debug mode pauses execution before each step and lets you inspect the context (main/payload), the current step, and the execution history through a TCP server and the TUI inspector.

## Enabling debug

Debug is enabled via an environment variable. Use `PHLOW_DEBUG=true`:

```bash
PHLOW_DEBUG=true phlow ./examples/any.phlow
```

By default the debug server listens on `0.0.0.0:31400`. To change the port, use `PHLOW_DEBUG_PORT`:

```bash
PHLOW_DEBUG=true PHLOW_DEBUG_PORT=31400 phlow ./examples/any.phlow
```

## Inspecting with phlow-tui-inspect

In another terminal, connect the inspector to the same port:

```bash
PHLOW_DEBUG_PORT=31400 phlow-cli-inspect
```

The inspector connects to `127.0.0.1`, so use a tunnel/port-forward if you need remote debugging.

## Main commands

You can type commands directly in the inspector bar:

- `STEP` - shows the step waiting for execution
- `SHOW` - shows the compiled script
- `NEXT` - releases one step
- `RELEASE` - releases the current pipeline
- `ALL` - shows step history
- `PAUSE` - pauses any ongoing release

Inspector shortcuts (equivalent to the commands above):

- `/n` (Ctrl+n) - NEXT + STEP
- `/a` (Ctrl+a) - NEXT + ALL
- `/r` (Ctrl+r) - RELEASE + ALL
- `/w` (Ctrl+w) - SHOW
- `/g` (Ctrl+g) - STEP

Use `/m` to open the command summary and `ESC` to close it.

## Notes and safety

- When debug is active, execution pauses at each step until it receives `NEXT` or `RELEASE`.
- The debug server is a simple TCP port. Use it only in trusted environments and avoid exposing it to the internet.
