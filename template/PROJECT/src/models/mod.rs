// This module is API surface the template ships for the generated app to
// call: user lookups, role checks, id newtypes. Until the app calls them they
// are dead code, and 17 such warnings on a fresh project drown out the first
// real one. Remove this allow once the app uses enough of it.
#![allow(dead_code)]

pub mod identity_provider;
pub mod ids;
pub mod role;
pub mod user;
pub mod user_role;
pub mod user_status;
pub mod user_with_roles;
