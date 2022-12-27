pub mod pages;
pub mod routes;

pub mod predule {
    pub use super::pages::{BasePage, DynPage, DynBasePage, DynPageDyn};
    pub use super::routes::{Route, UrlInfos};
}
