---
name: run
description: "Start the dev server as a Claude-tracked foreground subshell, or restart it if already running"
disable-model-invocation: true
allowed-tools: Bash(just *), Bash(curl *), Read
---

# Run dev server

## Instructions

Start the dev server as a Claude-tracked background shell so you can read its output and stop it later.

1. Invoke `just dev-agent` via Bash with `run_in_background: true`. The recipe kills any pre-existing instance and then `exec`s `just run serve` in the foreground of that shell, so Claude tracks the cargo/server process.

2. Capture the returned task ID and output file path.

3. Wait for the server to be listening on port 3000 by polling the port. A cold cargo build can take a couple of minutes, so give it up to 180s:

   ```bash
   for i in $(seq 1 180); do
       if curl -sf http://127.0.0.1:3000/ > /dev/null 2>&1; then
           echo "Server is up"; exit 0
       fi
       sleep 1
   done
   echo "Server did not come up within 180s"; exit 1
   ```

4. Report the task ID and the output file path to the user so they know which shell holds the server. Mention that `/stop` will kill that shell.

If the server fails to come up — or if you want to check build progress while waiting — Read the tail of the tracked output file to surface cargo output or the error.
