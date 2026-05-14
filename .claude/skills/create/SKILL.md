---
name: create
description: "Instantiate the rust-axum-template into a new project"
allowed-tools: Bash, AskUserQuestion, Read
---

# Create a new project from the rust-axum-template

## Arguments

- Optional: project name, git forge, git username

## Instructions

1. Ask the user for the following (use arguments if already provided):
   - **APP**: The project/application name (no spaces, alphanumeric and hyphens only)
   - **GIT_FORGE**: Git forge domain (default: `github.com`)
   - **GIT_USERNAME**: Git forge username or org name
   - **TEMPLATE_REPO**: Template repository URL (default: `https://github.com/EnigmaCurry/rust-axum-template.git`)
   - **TEMPLATE_BRANCH**: Branch to clone from (default: `master`)

2. Ask whether to:
   - **a)** Transform the current repository in-place (run `setup.sh` here), or
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
   APP="${APP}" GIT_FORGE="${GIT_FORGE}" GIT_USERNAME="${GIT_USERNAME}" bash setup.sh
   ```

5. Report success and remind the user to:
   - Review `LICENSE.txt`
   - Update the git remote if needed: `git remote set-url origin <new-url>`
   - Run `just run help` to verify the build
