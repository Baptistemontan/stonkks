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
    pub use next_rs_traits::predule::*;
    pub use server::{Response, Server};
}

// TODO:
// route macro
// props macro.
