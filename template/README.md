# ${APP}

[![Crates.io](https://img.shields.io/crates/v/${APP}?color=blue
)](https://crates.io/crates/${APP})
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://${GIT_USERNAME}.github.io/${APP}/coverage/master/)


## Run Docker container

### To run the container standalone with plain HTTP (no TLS)

```
docker run -d \
    --name ${APP} \
    -v my-app-data:/data \
    -p 3000:3000 \
    ghcr.io/${GIT_USERNAME}/${APP} \
    serve \
    --listen-ip 0.0.0.0 \
    --listen-port 3000
```

Modify the name of the data volume (`my-app-data`) to your liking (but
keep the same mount point `:/data`). `my-app-data` is a "named" Docker
volume, as such, it will be created automatically by Docker. This
volume is where the service's database will be permanently stored.

Without a proxy in front, some features cannot be turned on:

 - Trusted Header Auth will be disabled unless you use a proxy.
 - Trusted IP address forwarding will be disabled unless you use a proxy.

### To run the container with Traefik proxy (with TLS)

Run the container with Traefik Proxy and gain several advanced
features via [Forward
Authentication](https://doc.traefik.io/traefik/reference/routing-configuration/http/middlewares/forwardauth/).
This config is a bit specific to how the header authorization is done
in [d.rymcg.tech](https://github.com/EnigmaCurry/d.rymcg.tech). You
may need to adapt this for other environments:

```
## Double check these settings according to your environment:
TRAEFIK_HOST=${APP}.example.com
TRAEFIK_PROXY=10.13.16.1
TRUSTED_HEADER_NAME=X-Forwarded-User
TRUSTED_FORWARDED_FOR_NAME=X-Forwarded-For
TRAEFIK_ENTRYPOINT=websecure

## These middleware allow only the 'admin' OAuth group access to the service:
TRUSTED_HEADER_AUTH_MIDDLEWARE=traefik-forward-auth@docker,header-authorization-group-admin@file

docker run -d \
    --name ${APP} \
    -v my-app-data:/data \
    -e TRUSTED_PROXY=${TRAEFIK_PROXY} \
    -e TRUSTED_HEADER_NAME=${TRUSTED_HEADER_NAME} \
    -e TRUSTED_FORWARDED_FOR_NAME=${TRUSTED_FORWARDED_FOR_NAME} \
    -e TRUSTED_HEADER_AUTH=true \
    -e TRUSTED_FORWARDED_FOR=true \
    -l traefik.enable=true \
    -l traefik.http.routers.${APP}.rule=Host\(\`${TRAEFIK_HOST}\`\) \
    -l traefik.http.routers.${APP}.entrypoints=${TRAEFIK_ENTRYPOINT} \
    -l traefik.http.routers.${APP}.tls=true \
    -l traefik.http.services.${APP}.loadbalancer.server.port=3000 \
    -l traefik.http.routers.${APP}.middlewares=${TRUSTED_HEADER_AUTH_MIDDLEWARE} \
    ghcr.io/${GIT_USERNAME}/${APP} \
    serve
```

### Check the service

The service should now be running.

On standalone installs, it should be on port 3000. Open your browser
to the domain name or IP address associated with your server:

https://${APP}.example.com:3000

For Traefik installs, it should be on port 443, and you must use the
same domain name that was configured for the Traefik route
(`TRAEFIK_HOST`).

To view the container status and and its logs:

```
docker ps -f name=${APP}

docker logs ${APP}
```

## Install

If you don't want to run the Docker container, you can install the
binary directly:

[Download the latest release for your platform.](https://github.com/${GIT_USERNAME}/${APP}/releases)

Or install via cargo ([crates.io/crates/${APP}](https://crates.io/crates/${APP})):

```
cargo install ${APP}
```

### Tab completion

To install tab completion support, put this in your `~/.bashrc` (assuming you use Bash):

```
### Bash completion for ${APP} (Put this in ~/.bashrc)
source <(${APP} completions bash)
```

If you don't like to type out the full name `${APP}`, you can make
a shorter alias (`h`), as well as enable tab completion for the alias
(`h`):

```
### Alias ${APP} as h (Put this in ~/.bashrc):
alias h=${APP}
complete -F _${APP} -o bashdefault -o default h
```

Completion for Zsh and/or Fish has also been implemented, but the
author has not tested this:

```
### Zsh completion for ${APP} (Put this in ~/.zshrc):
autoload -U compinit; compinit; source <(${APP} completions zsh)

### Fish completion for ${APP} (Put this in ~/.config/fish/config.fish):
${APP} completions fish | source
```

## Usage

```
$ ${APP}

Usage: ${APP} [OPTIONS] [COMMAND]

Commands:
  serve        Run the HTTP API server
  hello        Greeting
  completions  Generates shell completions script (tab completion)
  help         Print this message or the help of the given subcommand(s)

Options:
      --log <LEVEL>  Sets the log level, overriding the RUST_LOG environment variable. [possible values: trace, debug, info, warn, error]
  -v                 Sets the log level to debug.
  -h, --help         Print help
  -V, --version      Print version
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md)
