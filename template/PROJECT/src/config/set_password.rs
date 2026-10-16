use app_macros::app_conf_root;
use conf::Conf;

use super::DatabaseConfig;

// app_macros::app_conf_root hard-codes the app prefix for env vars:
app_conf_root! {
    pub struct SetPasswordConfig {
        /// Username of the account to set a local password for.
        #[arg(long, short = 'u', env = "SET_PASSWORD_USERNAME")]
        #[conf(default("root".to_string()))]
        pub username: String,

        /// Read the password from stdin instead of prompting for it.
        ///
        /// Implied when stdin is not a terminal, so piping works unattended.
        #[arg(long = "stdin")]
        pub stdin: bool,

        /// Where the database lives.
        #[conf(flatten, serde(flatten))]
        pub database: DatabaseConfig,
    }
}
