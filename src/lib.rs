mod api;
mod app;
mod client;
mod default;
mod pages;
mod server;

pub mod prelude {
    use super::*;
    pub use app::App;
    pub use client::Client;
    pub use server::{Response, Server};
    pub use stonkks_core::predule::*;
}

// TODO:
// route macro
// props macro.
