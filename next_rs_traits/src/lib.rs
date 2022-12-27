pub mod pages;
pub mod routes;

pub mod predule {
    pub use super::pages::{BasePage, DynBasePage, DynPage, DynPageDyn};
    pub use super::routes::{Route, UrlInfos};
}
