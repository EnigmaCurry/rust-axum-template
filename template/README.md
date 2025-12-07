# ${APP}

[![Crates.io](https://img.shields.io/crates/v/${APP}?color=blue
)](https://crates.io/crates/${APP})
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://${GIT_USERNAME}.github.io/${APP}/coverage/master/)

## Build

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

## Run

Run `${APP} --help` to find all the options, but broadly speaking
there are a few ways you can run it:

### Plain HTTP

You should always use TLS, so only use plain HTTP if you are hosting
behind a reverse proxy that terminates TLS for you:

```
export NET_HOST=${APP}.example.org
export NET_LISTEN_IP=0.0.0.0
export NET_LISTEN_PORT=80
export DATABASE_URL=sqlite:data.db
export AUTH_METHOD=username_password
export SESSION_SECURE=false
export RUST_LOG=${APP_MODULE}=info

${APP} serve
```

### Manual TLS

```
export NET_HOST=${APP}.example.org
export NET_LISTEN_IP=0.0.0.0
export NET_LISTEN_PORT=443
export DATABASE_URL=sqlite:data.db
export AUTH_METHOD=username_password
export SESSION_SECURE=true
export TLS_MODE=manual
export TLS_CERT_PATH=cert.pem
export TLS_KEY_PATH=key.pem
export RUST_LOG=${APP_MODULE}=info

${APP} serve
```

### ACME (TLS-ALPN-01)

```
export NET_HOST=${APP}.example.org
export TLS_SANS=
export NET_LISTEN_IP=0.0.0.0
export NET_LISTEN_PORT=443
export DATABASE_URL=sqlite:data.db
export AUTH_METHOD=username_password
export SESSION_SECURE=true
export TLS_MODE=acme
export TLS_ACME_CHALLENGE=tls-alpn-01
export TLS_ACME_DIRECTORY_URL=https://acme-v02.api.letsencrypt.org/directory
export TLS_ACME_EMAIL=
export TLS_CACHE_DIR=./tls-cache
export RUST_LOG=${APP_MODULE}=info

${APP} serve
```

### ACME (DNS-01 via ACME-DNS)

```
export NET_HOST=${APP}.example.org
export TLS_SANS=
export NET_LISTEN_IP=0.0.0.0
export NET_LISTEN_PORT=443
export DATABASE_URL=sqlite:data.db
export AUTH_METHOD=username_password
export SESSION_SECURE=true
export TLS_MODE=acme
export TLS_ACME_CHALLENGE=dns-01
export TLS_ACME_DIRECTORY_URL=https://acme-v02.api.letsencrypt.org/directory
export TLS_ACME_EMAIL=
export ACME_DNS_API_BASE=https://auth.acme-dns.io
export TLS_CACHE_DIR=./tls-cache
export RUST_LOG=${APP_MODULE}=info

## Register ACME-DNS account 
## Follow the instructions it gies to create CNAME records:
${APP} acme-dns-register

## Loads ACME-DNS credentials and provisions cert on first run:
${APP} serve
```

### Self-Signed TLS

```
export NET_HOST=${APP}.example.org
export NET_LISTEN_IP=0.0.0.0
export NET_LISTEN_PORT=443
export DATABASE_URL=sqlite:data.db
export AUTH_METHOD=username_password
export SESSION_SECURE=true
export TLS_MODE=self-signed
## If TLS_CACHE_DIR is not set, self-signed certs are ephemeral:
export TLS_CACHE_DIR=./tls-cache
export RUST_LOG=${APP_MODULE}=info

${APP} serve
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md)
