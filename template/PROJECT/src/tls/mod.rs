// Certificate helpers the generated app may or may not reach for, depending
// on which TLS mode it uses. Dead code here is expected rather than a smell.
#![allow(dead_code)]

pub mod cert_details;
pub mod dns;
pub mod generate;
pub mod http_redirect;
pub mod self_signed_cache;
