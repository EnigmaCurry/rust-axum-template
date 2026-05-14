# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a Rust project template for building production web servers with Axum. The repo has two layers: the outer `setup.sh` instantiation system and the inner `template/` directory containing the actual application. Most development happens inside `template/`.

The template uses `${APP}` as a placeholder variable throughout — it gets replaced by `setup.sh` when instantiating a new project. When working in an instantiated project, `${APP}` will be the actual project name.

## Common Commands

All commands run from the `template/` directory (or the project root in an instantiated repo):

```bash
just deps                    # Install dev tools (cargo-nextest, git-cliff, cargo-llvm-cov, sqlx-cli)
just run [ARGS]              # Build and run (e.g., just run help, just run serve)
just build                   # Release build (includes frontend)
just test                    # Run all tests (uses temp SQLite DB)
just test test_name          # Run a single test
just test-verbose test_name  # Single test with log output
just test-watch              # Continuous testing on file change
just clippy                  # Lint (treats warnings as errors)
just clippy --fix            # Auto-fix lints
just migrate                 # Run SQLx migrations on local DB
just test-coverage           # LLVM coverage report
just build-frontend          # Build SvelteKit SPA (pnpm)
```

Tests always use a temporary SQLite database (via `_with-temp-db` helper in Justfile) — never the local dev database.

## Architecture

**Workspace crates** (in `template/`):
- `PROJECT/` (renamed to `${APP}`) — main binary crate
- `api-doc-macros/` — proc macros for OpenAPI documentation
- `app-macros/` — proc macros for app utilities
- `frontend/` — SvelteKit SPA served as static files by the backend

**Application flow**: `main.rs` → `run_cli()` parses CLI/env config → subcommands (primarily `serve` which calls `server::run()`).

**Configuration** (12-factor, 4-level priority): CLI args > env vars > `~/.local/share/${APP}/defaults.toml` > compiled defaults. Config structs live in `src/config/` with modules for network, database, session, auth, and TLS.

**Key patterns**:
- `AppState` (in `server.rs`) holds `SqlitePool` and `AuthConfig`, shared across handlers
- Routes use `aide::axum::ApiRouter` for automatic OpenAPI spec generation
- Middleware stack (Tower-based): `TraceLayer` → `trusted_forwarded_for` → `user_session` → optional OIDC → `csrf_protection` → route-specific layers
- Three auth methods selectable via config: `UsernamePassword`, `ForwardAuth` (reverse proxy), `Oidc`
- Role-based access control via `require_roles_middleware` (e.g., admin routes require `SystemRole::Admin`)
- Five TLS modes: `Http`, `RustlsFiles`, `SelfSigned` (with renewal loop), `AcmeTlsAlpn01`, `AcmeDns01`
- Database: SQLite with WAL mode, migrations in `PROJECT/migrations/`, compile-time checked queries via `sqlx`
- Sessions: SQLite-backed via `tower-sessions` with background expired-session cleanup
- Templates: Askama (Jinja-like) in `PROJECT/templates/`
- API responses wrapped in `ApiResponse<T>` envelope (see `response.rs`)
- Frontend SPA: SvelteKit static build served as fallback route (`/*`)

**Route structure** (in `src/routes/mod.rs`):
- `/api/*` — REST API (CSRF protected)
- `/login/*` — Auth endpoints (CSRF protected)
- `/admin/*` — Admin interface (requires Admin role + CSRF)
- `/docs/*` — OpenAPI docs (Scalar/Redoc/Swagger)
- `/static/*` — Static assets
- `/*` — SvelteKit SPA fallback

## Release Process

```bash
just bump-version   # Creates release-vX.X.X branch with version updates
git push            # Push branch, create PR
# After PR merge:
git checkout master && git pull
just release        # Tags and pushes, triggers GitHub Actions (builds binaries + Docker images)
```

## Template System

`setup.sh` instantiates the template by replacing `${APP}`, `${GIT_FORGE}`, `${GIT_USERNAME}` via `envsubst`. To merge changes back upstream to the template repo:

```bash
just merge-template-upstream  # Reverses variable substitution and stages changes in ../rust-axum-template
just new-template-branch      # Test template changes in an orphan branch
```
