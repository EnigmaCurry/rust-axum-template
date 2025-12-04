use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AuthenticationMethod {
    /// Use a forward-auth proxy (Traefik, etc.) via trusted header.
    #[clap(name = "forward_auth")]
    ForwardAuth,

    /// Use traditional username/password login.
    #[clap(name = "username_password")]
    UsernamePassword,
}
