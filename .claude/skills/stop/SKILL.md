---
name: stop
description: "Stop the running dev server"
disable-model-invocation: true
allowed-tools: Bash(just *)
---

# Stop dev server

## Instructions

Stop the dev server. Prefer killing the Claude-tracked shell that `/run` started so you also tear down the cargo/server child process under it:

1. If you have a tracked task ID from a `/run` invocation in this session, call `TaskStop` on that task ID.

2. Otherwise (no tracked shell, e.g. across sessions or after a compact), fall back to:

   ```bash
   just dev-agent-stop
   ```

   which uses `pgrep` + `kill` to find and stop the binary by name.

3. Verify the port is no longer responding and report the outcome to the user.
