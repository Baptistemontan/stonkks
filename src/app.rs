use crate::client::Client;
use crate::pages::StaticPages;

use super::default::{AppLayout, NotFound};
use super::pages::DynPages;
use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::{DynComponent, DynPageDyn, StaticPage, DynStaticPage};
use sycamore::prelude::*;

pub const SERIALIZED_PROPS_KEY: &str = "NEXT_RS_SERIALIZED_PROPS";
pub const NEXT_RS_WINDOW_OBJECT_KEY: &str = "__NEXT_RS__";
pub const CLIENT_WASM_FILE_PATH: &str = "/public/next_rs_wasm_app.wasm";
pub const CLIENT_JS_FILE_PATH: &str = "/public/next_rs_js_app.js";
pub const ROOT_ELEMENT_ID: &str = "__NEXT_RS_ROOT__";

#[derive(Default)]
pub struct App {
    dyn_pages: DynPages,
    static_pages: StaticPages,
    layout: AppLayout,
    not_found_page: NotFound,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dyn_page<T: DynPage + 'static>(mut self, page: T) -> Self {
        self.dyn_pages.add_page(page);
        self
    }

    pub fn static_page<T: StaticPage + 'static>(mut self, page: T) -> Self {
        self.static_pages.add_page(page);
        self
    }

    pub fn dyn_pages<I>(mut self, pages: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn DynPageDyn>>,
    {
        self.dyn_pages.add_boxed_pages(pages);
        self
    }

    pub fn static_pages<I>(mut self, pages: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn DynStaticPage>>,
    {
        self.static_pages.add_boxed_pages(pages);
        self
    }

    pub fn with_layout<T: Layout>(mut self, layout: T) -> Self {
        self.layout = layout.into();
        self
    }

    pub fn not_found<T: NotFoundPage>(mut self, not_found: T) -> Self {
        self.not_found_page = not_found.into();
        self
    }

    fn into_inner(self) -> AppInner {
        AppInner {
            dyn_pages: self.dyn_pages,
            static_pages: self.static_pages,
            layout: self.layout,
            not_found_page: self.not_found_page,
        }
    }

    pub fn into_client(self) -> Client {
        self.into_inner().into()
    }

    pub fn into_server(self) -> Server {
        self.into_inner().into()
    }
}

pub struct AppInner {
    dyn_pages: DynPages,
    static_pages: StaticPages,
    layout: AppLayout,
    not_found_page: NotFound,
}

impl AppInner {
    pub fn dyn_pages(&self) -> &DynPages {
        &self.dyn_pages
    }

    pub fn static_pages(&self) -> &StaticPages {
        &self.static_pages
    }

    pub fn layout(&self) -> &dyn DynLayout {
        &*self.layout
    }

    pub fn not_found_page(&self) -> &dyn DynComponent {
        &*self.not_found_page
    }
}

fn window_object_script(props: &str) -> String {
    format!(
        "window.{0}=window.{0}||{{}};window.{0}.{1}=\'{2}\';",
        NEXT_RS_WINDOW_OBJECT_KEY, SERIALIZED_PROPS_KEY, props
    )
}

fn default_head<G: Html>(cx: Scope, head: View<G>, props: &str) -> View<G> {
    let script = window_object_script(props);
    view! { cx,
        head {
            meta(charset = "UTF-8")
            meta(http-equiv="X-UA-Compatible", content="IE=edge")
            meta(name="viewport", content="width=device-width, initial-scale=1.0")
            link(rel="preload", href=CLIENT_WASM_FILE_PATH, as="fetch", type="application/wasm", crossorigin="")
            link(rel="modulepreload", href=CLIENT_JS_FILE_PATH)
            script {
                (script)
            }
            (head)
        }
    }
}

pub fn default_html_view<G: Html>(cx: Scope, body: View<G>, head: View<G>, props: &str) -> View<G> {
    let head = default_head(cx, head, props);
    view! { cx,
        (head)
        body {
            script(type="module") {
                "import init from '"(CLIENT_JS_FILE_PATH)"';init('"(CLIENT_WASM_FILE_PATH)"');"
            }
            (body)
        }
    }
}
