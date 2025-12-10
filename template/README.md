# ${APP}

[![Crates.io](https://img.shields.io/crates/v/${APP}?color=blue
)](https://crates.io/crates/${APP})
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://${GIT_USERNAME}.github.io/${APP}/coverage/master/)

## Features

 * Single binary deployment.
 * Embedded SQLite database.
 * RESTFul JSON API built with
   [axum](https://github.com/tokio-rs/axum).
 * Builtin TLS with the following modes:
   * ACME supporting TLS-ALPN-01 and DNS-01 challenge types. (e.g.,
     when you need a production certificate from Let's Encrypt.)
   * Automatic TLS with self-signed certificate (e.g., when using
     certificate pinning).
   * TLS with a provided certificate and key file (e.g., `.pem` files
     that you rotate manually).
   * None (plain HTTP) (e.g., when deployed behind a reverse proxy
     that terminates TLS on its behalf).
 * OpenAPI specification built with
   [aide](https://github.com/tamasfe/aide/).
   * Interactive API docs with your choice of
     [Scalar](https://github.com/ScalaR/ScalaR),
     [Redoc](https://github.com/Redocly/redoc), or [Swagger
     UI](https://github.com/swagger-api/swagger-ui?tab=readme-ov-file).
 * Multiple user authentication backends:
   * Username / Password.
   * Forward Auth via trusted header (Traefik Proxy or compatible proxy layer).
   * Todo: OAuth (OIDC).
 * Admin web interface.
 * [Just](https://github.com/casey/just) enabled project build
   targets.
 * [Clap](https://docs.rs/clap/latest/clap/) CLI argument parser.
 * Bash / Fish / Zsh shell (tab)
   [completion](https://docs.rs/clap_complete/latest/clap_complete/).
 * GitHub actions for tests and releases:
   * Test coverage report published to GitHub pages.
   * Builds executables for multiple platforms.
   * Builds Docker images for X86_64 and AArch64.
   * Publishing crates to crates.io (disabled by default, uncomment in
   [release.yml](template/.github/workflows/release.yml)).

## Build

 * Install Rust with [rustup](https://rustup.rs/).
 * Install
   [Just](https://github.com/casey/just?tab=readme-ov-file#installation)
   (`cargo install just`)

```
just build --release
```

Find your built executable in `target/release/${APP}`.

## Install

```
sudo install \
  target/release/${APP} \
  /usr/local/bin/${APP}
```

## Configuration

The application uses a multi-source configuration system, consisting
of the following layers (from highest to lowest priority):

 1. **Command line arguments**. Every configuration setting has a long
    form CLI argument (e.g., `--some-setting foo`). Explicit args like
    this have the highest priority and override the setting from all
    other layers.

 2. **Environment variables**. Every configuration setting has an
    associated environment variable. This is the preferred
    configuration style for Docker containers.

 3. **User Defaults**. The application has an optional config file in
    it's data root (`defaults.toml`). This file dynamically overrides
    the application's *default* settings and help messages.

 4. **Application defaults**. Every configuration setting has a
    default value compiled into the binary.

### Application storage (stateful data)

The application needs to store its SQLite database files, ACME
accounts, and TLS certificates. By default, the application creates
files in `${XDG_DATA_HOME}/${APP}`, or `${HOME}/.local/share/${APP}`
(if no `XDG_DATA_HOME` is set) or `./${APP}-data` (if no `HOME`
variable is set).

If you want to use a different path, or if you want to support
multiple instances of the app, you need to override the path using any
of the following methods:

 * Use the command line arguments `-C PATH` or `--root-dir PATH`.
 * Set the `ROOT_DIR` environment variable.

## Run

Run `${APP} --help` to find all the options, but broadly speaking
there are a few ways you can run it:

### Plain HTTP

You should always use TLS, so only use plain HTTP if you are hosting
behind a reverse proxy that terminates TLS for you:

```
${APP} serve -v \
  --net-host           ${APP}.example.org \
  --net-listen-ip      0.0.0.0 \
  --net-listen-port    8000 \
  --auth-method        username_password \
  --session-secure     false
```

### Manual TLS

```
${APP} serve -v \
  --net-host           ${APP}.example.org \
  --net-listen-ip      0.0.0.0 \
  --net-listen-port    8000 \
  --auth-method        username_password \
  --session-secure     true \
  --tls-mode           manual \
  --tls-cert-path      cert.pem \
  --tls-key-path       key.pem
```

### ACME (TLS-ALPN-01)

```
${APP} serve -v \
  --net-host               ${APP}.example.org \
  --net-listen-ip          0.0.0.0 \
  --net-listen-port        443 \
  --auth-method            username_password \
  --session-secure         true \
  --tls-mode               acme \
  --tls-acme-challenge     tls-alpn-01 \
  --tls-acme-directory-url https://acme-v02.api.letsencrypt.org/directory \
  --tls-acme-email         ""
```

Note: TLS-ALPN-01 only work on port 443. So you need to run as `root`.

### ACME (DNS-01 via ACME-DNS)

```
## Register your ACME-DNS account. 
## Specify all of your domains (SANS) to get help with the CNAME records:
${APP} acme-dns-register \
  --net-host  ${APP}.example.org \
  --tls-san ""

## Loads ACME-DNS credentials and provisions cert on first run:
${APP} serve -v \
  --net-host               ${APP}.example.org \
  --net-listen-ip          0.0.0.0 \
  --net-listen-port        8443 \
  --auth-method            username_password \
  --session-secure         true \
  --tls-mode               acme \
  --tls-san                "" \
  --tls-acme-challenge     dns-01 \
  --tls-acme-directory-url https://acme-v02.api.letsencrypt.org/directory \
  --tls-acme-email         "" \
  --acme-dns-api-base      https://auth.acme-dns.io
```

### Self-Signed TLS

```
${APP} serve -v \
  --net-host               ${APP}.example.org \
  --net-listen-ip          0.0.0.0 \
  --net-listen-port        8443 \
  --auth-method            username_password \
  --session-secure         true \
  --tls-mode               self-signed
```

## Development

For development, you are advised to install
[just](https://github.com/casey/just) and use the targets defined in
the [Justfile](Justfile).

## Configure the .env file

```
just config
```

This will copy the provided [.env-dist](template/.env-dist) to `.env`.
You should edit the generated `.env` file by hand to configure your
application.

You can set an alternative `.env` file path by setting the `ENV_FILE`
environment variable.

## Run the program

```
just run [ARGS ...]
```

You can also run the binary directly by building manually (`just
build`) and running the static binary
`{{app_name}}/target/debug/{{app_name}}`.

Also see [DEVELOPMENT.md](DEVELOPMENT.md)
