# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`${APP}` is a Rust web server built on Axum, generated from
[rust-axum-template](https://github.com/EnigmaCurry/rust-axum-template). The
template supplies the whole infrastructure layer: CLI and configuration, auth,
sessions, TLS, the SQLite layer, the SvelteKit SPA, and the build tooling.

## Keep generic changes upstream

**Before changing anything that is not specific to this app, ask whether a
different project generated from the same template would want the same change.
If the answer is yes, make it upstream in rust-axum-template and inherit it —
do not fix it here.**

That applies to auth, configuration and env plumbing, the `Justfile`, the
`Dockerfile` and docker recipes, the vendored `src/lib/components/ui`
components, build warnings, and the generator scripts in `_scripts/`.

The reason is practical: this project is a *snapshot* of the template, and the
supported way to pick up later template improvements is to re-instantiate and
move your application code across (`just new-template-branch`). Every generic
change made locally is a divergence you have to reconcile by hand each time, or
silently re-fix. Changes made upstream come back for free.

Keep edits to template-owned files (`src/main.rs`, `src/server.rs`,
`src/routes/mod.rs`, `src/config/mod.rs`) to the minimum wiring your app needs —
module declarations, config fields, `AppState` fields, router nesting — because
those are the files that conflict when you re-inherit. Put the app's own logic
in its own modules.

## Common Commands

```bash
just deps                    # Install dev tools (cargo-nextest, git-cliff, cargo-llvm-cov, sqlx-cli)
just run [ARGS]              # Build and run (e.g. just run help, just run serve)
just build                   # Release build (includes frontend)
just test                    # Run all tests (uses a temporary SQLite database)
just test test_name          # Run a single test
just test-verbose test_name  # Single test with log output
just test-watch              # Continuous testing on file change
just clippy                  # Lint (treats warnings as errors)
just migrate                 # Run SQLx migrations on the local database
just build-frontend          # Build the SvelteKit SPA (pnpm)
just set-password            # Set a local login password (see below)
```

`just test` runs against a temporary database, never your local dev one.

### First run

The root user is created without a password, so set one before logging in with
the default `username_password` auth method:

```bash
just run set-password            # hidden prompt
echo 'hunter2 hunter2' | just run set-password --stdin
```

## Configuration

Four levels, highest priority first: CLI args > env vars >
`~/.local/share/${APP}/defaults.toml` > compiled defaults. Config structs live
in `src/config/`.

Environment variables are prefixed with the app name — `${APP_PREFIX}DATABASE_URL`,
`${APP_PREFIX}NET_LISTEN_PORT`, and so on. `ROOT_DIR` is deliberately unprefixed,
because the CLI layer reads it before the config layer exists. `.env` is loaded
by `just`; create it with `just _env_init`.

## Architecture

- `AppState` (in `src/server.rs`) holds the `SqlitePool`, `AuthConfig`, and a
  broadcast channel used to notify SSE clients of shutdown
- Routes use `aide::axum::ApiRouter` so the OpenAPI spec is generated from them
- Middleware stack: `TraceLayer` → `trusted_forwarded_for` → `user_session` →
  optional OIDC → `csrf_protection` → route-specific layers
- Three auth methods: `UsernamePassword`, `ForwardAuth` (reverse proxy), `Oidc`
- Role-based access via `require_roles_middleware` (admin routes require
  `SystemRole::Admin`)
- Five TLS modes: `Http`, `RustlsFiles`, `SelfSigned`, `AcmeTlsAlpn01`, `AcmeDns01`
- SQLite with WAL, migrations in `migrations/`, compile-time checked queries
- Sessions are SQLite-backed via `tower-sessions`, with background cleanup
- API responses use the `ApiResponse<T>` envelope (`src/response.rs`)
- SSE at `/api/events`, with a 30s keep-alive and a shutdown broadcast
- The SvelteKit SPA is served as the `/*` fallback route

Route groups (`src/routes/mod.rs`): `/api/*` (CSRF protected), `/api/events`
(SSE, GET only), `/login/*`, `/admin/*` (Admin role), `/docs/*` (OpenAPI UIs),
`/static/*`, and `/*` for the SPA.

## Release Process

```bash
just bump-version   # Creates a release-vX.X.X branch with version updates
git push            # Push the branch, open a PR
# after merge:
git checkout master && git pull
just release        # Tags and pushes, triggering the release workflow
```

## Inheriting template updates

```bash
just new-template-branch      # Orphan branch with a fresh template instantiation
just merge-template-upstream  # Push local template changes back upstream
```

`just new-template-branch` runs `git clean -fdx`, which deletes ignored files —
`.env`, `target/`, `node_modules/`. Save anything you need from those first.
