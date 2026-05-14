---
name: create
description: "Instantiate the rust-axum-template into a new project"
allowed-tools: Bash, AskUserQuestion, Read
---

# Create a new project from the rust-axum-template

## Arguments

Parse `$ARGUMENTS` for the following values. Arguments may be passed positionally
(`/create my-app github.com acme`) or as `key=value` pairs
(`/create app=my-app git_username=acme`). Skip prompts for any value already provided.

- **APP** (1st positional or `app=`): Project name (alphanumeric and hyphens only)
- **GIT_FORGE** (2nd positional or `git_forge=`): Git forge domain (default: `github.com`)
- **GIT_USERNAME** (3rd positional or `git_username=`): Git forge username or org name
- **DEPS_TARGET** (`deps_target=`): Dependency install target (default: `bin-deps`)

Only ask about template repo and branch if the user mentioned them:
- **TEMPLATE_REPO**: `https://github.com/EnigmaCurry/rust-axum-template.git`
- **TEMPLATE_BRANCH**: `master`

## Instructions

1. For any required value not provided via arguments, ask the user with AskUserQuestion:
   - For **APP**: offer a default based on the current directory name as the first option, with a second descriptive option like "Enter a custom name" (select Other to type one)
   - For **GIT_FORGE**: offer `github.com` (Recommended) and `codeberg.org` as options
   - For **GIT_USERNAME**: offer the username parsed from `git remote get-url origin` as default, with "Enter a different username" as the second option

2. Ask whether to:
   - **a)** Transform the current repository in-place (run `setup.sh` here) (Recommended)
   - **b)** Create a new project directory elsewhere

3. If creating a new directory (option b):
   ```bash
   NEW_DIR="../${APP}"
   git clone --branch "${TEMPLATE_BRANCH}" "${TEMPLATE_REPO}" "${NEW_DIR}"
   ```
   Then run setup.sh from inside that directory.

4. Run setup.sh non-interactively:
   ```bash
   cd "${PROJECT_DIR}"
   DEPS_TARGET="${DEPS_TARGET}" APP="${APP}" GIT_FORGE="${GIT_FORGE}" GIT_USERNAME="${GIT_USERNAME}" bash setup.sh
   ```

   Note: `DEPS_TARGET=bin-deps` uses `cargo-binstall` for prebuilt binaries, but some
   packages (e.g., `sqlx-cli`) may fall back to source compilation. This requires
   `pkg-config` and `openssl` headers. If building in a Nix environment, prefer
   `nix-shell` (the repo includes a `shell.nix`) to ensure all native deps are available.

5. Verify the build succeeded:
   ```bash
   just test
   ```
   All tests should pass. If they fail, diagnose and fix before continuing.

6. Report success and remind the user to:
   - Review `LICENSE.txt`
   - Run `just run help` to verify the build
