use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;

use crate::api::ApiRoutes;
use crate::client::Client;
use crate::pages::StaticPages;
use crate::utils::StaticPageAndRoute;

use super::default::{AppLayout, NotFound};
use super::pages::DynPages;
use super::prelude::*;
use async_fs as fs;

use futures::stream::FuturesUnordered;
use futures::{AsyncReadExt, AsyncWriteExt, TryStreamExt};
use std::hash::{Hash, Hasher};
use stonkks_core::api::DynApi;
use stonkks_core::layout::DynLayout;
use stonkks_core::pages::{DynComponent, DynPageDyn, DynRenderResult, DynStaticPage, StaticPage};
use stonkks_core::states::StatesMap;
use sycamore::prelude::*;

pub const SERIALIZED_PROPS_KEY: &str = "__STONKKS_SERIALIZED_PROPS__";
pub const STONKKS_WINDOW_OBJECT_KEY: &str = "__STONKKS_OBJECT__";
pub const CLIENT_WASM_FILE_PATH: &str = "/public/stonkks_wasm_app.wasm";
pub const CLIENT_JS_FILE_PATH: &str = "/public/stonkks_js_app.js";
pub const ROOT_ELEMENT_ID: &str = "__STONKKS_ROOT__";
pub const STONKKS_FOLDER_NAME: &str = ".stonkks";

#[derive(Default)]
pub struct App {
    dyn_pages: DynPages,
    static_pages: StaticPages,
    api: ApiRoutes,
    states: StatesMap,
    layout: AppLayout,
    not_found_page: NotFound,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dyn_page<T: DynPage>(mut self, page: T) -> Self {
        self.dyn_pages.add_page(page);
        self
    }

    pub fn static_page<T: StaticPage>(mut self, page: T) -> Self
    where
        for<'a> <T as Routable>::Route<'a>: Hash,
    {
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

    pub fn api<T: Api>(mut self, api: T) -> Self {
        self.api.add_route(api);
        self
    }

    pub fn apis<I>(mut self, apis: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn DynApi>>,
    {
        self.api.add_routes(apis);
        self
    }

    pub fn state<T: Any + Send + Sync>(mut self, state: T) -> (Self, Option<T>) {
        let old = self
            .states
            .add_state(state)
            .map(|old| *old.downcast::<T>().unwrap());
        (self, old)
    }

    pub fn state_unwrap<T: Any + Send + Sync>(mut self, state: T) -> Self {
        if let Some(_) = self.states.add_state(state) {
            let name = std::any::type_name::<T>();
            panic!("This state was already present: {}.", name);
        } else {
            self
        }
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
        let App {
            dyn_pages,
            static_pages,
            api,
            layout,
            not_found_page,
            states,
        } = self;
        let inner = AppInner::new(dyn_pages, static_pages, layout, not_found_page);
        Server::new(inner, api, states)
    }
}

pub struct AppInner {
    dyn_pages: DynPages,
    static_pages: StaticPages,
    layout: AppLayout,
    not_found_page: NotFound,
}

#[derive(Debug)]
pub enum StaticGenerationError {
    Io(std::io::Error),
    User(String),
    RouteMismatch(String),
}

impl From<std::io::Error> for StaticGenerationError {
    fn from(value: std::io::Error) -> Self {
        StaticGenerationError::Io(value)
    }
}

impl From<String> for StaticGenerationError {
    fn from(value: String) -> Self {
        StaticGenerationError::User(value)
    }
}

impl AppInner {
    pub fn new(
        dyn_pages: DynPages,
        static_pages: StaticPages,
        layout: AppLayout,
        not_found_page: NotFound,
    ) -> Self {
        AppInner {
            dyn_pages,
            static_pages,
            layout,
            not_found_page,
        }
    }

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

    async fn save_page(
        page: String,
        page_name: &str,
        route_hash: u64,
        serialized_props: &str,
    ) -> Result<(), std::io::Error> {
        let mut hasher = DefaultHasher::new();
        page_name.hash(&mut hasher);
        let hashed_page_name = hasher.finish();
        let dir_path = format!(
            "./{}/page_{:x}/route_{:x}",
            STONKKS_FOLDER_NAME, hashed_page_name, route_hash
        );
        fs::create_dir_all(&dir_path).await?;
        let page_html_path = format!("{}/html.html", dir_path);

        let mut page_html_file = fs::File::create(page_html_path).await?;
        page_html_file.write_all(page.as_bytes()).await?;
        page_html_file.flush().await?;

        let page_props_path = format!("{}/props.json", dir_path);

        let mut page_props_file = fs::File::create(page_props_path).await?;
        page_props_file
            .write_all(serialized_props.as_bytes())
            .await?;
        page_props_file.flush().await
    }

    async fn generate_page(
        &self,
        page: &dyn DynStaticPage,
        states: &StatesMap,
    ) -> Result<(), StaticGenerationError> {
        let build_routes = page.get_build_routes(states).await?;
        for url in build_routes {
            let url_infos = OwnedUrlInfos::parse_from_url(&url);
            let page_and_route = StaticPageAndRoute::try_match_route(page, url_infos.to_shared());
            let Some(page_and_route) = page_and_route else {
                return Err(StaticGenerationError::RouteMismatch(format!("Route {} was provided as build route, but did not match.", url)));
            };
            let route_hash = page_and_route.hash_route();
            let page_and_props = page_and_route.get_props(states).await?;
            let serialized_props = page_and_props.serialize_props()?;
            let html = sycamore::render_to_string(|cx| {
                let DynRenderResult { body, head } = page_and_props.render_server(cx);
                let body = self.layout().render_server(cx, body);
                default_html_view(cx, body, head, &serialized_props, true)
            });
            let full_page = format!(
                "<!DOCTYPE html><html id=\"{}\">{}</html>",
                ROOT_ELEMENT_ID, html
            );
            let page_name = page.get_name();

            Self::save_page(full_page, page_name, route_hash, &serialized_props).await?;
        }
        Ok(())
    }

    pub async fn generate_static_pages(
        &self,
        states: &StatesMap,
    ) -> Result<(), StaticGenerationError> {
        self.static_pages
            .iter()
            .map(|page| async { self.generate_page(page, states).await })
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .await
    }

    pub async fn get_static_page_html(
        page_name: &str,
        route_hash: u64,
    ) -> Result<String, std::io::Error> {
        let mut hasher = DefaultHasher::new();
        page_name.hash(&mut hasher);
        let hashed_page_name = hasher.finish();
        let page_html_path = format!(
            "./{}/page_{:x}/route_{:x}/html.html",
            STONKKS_FOLDER_NAME, hashed_page_name, route_hash
        );
        let mut page_html_file = fs::File::open(page_html_path).await?;
        let mut html = String::new();
        page_html_file.read_to_string(&mut html).await?;
        Ok(html)
    }

    pub async fn get_static_page_props(
        page_name: &str,
        route_hash: u64,
    ) -> Result<String, std::io::Error> {
        let mut hasher = DefaultHasher::new();
        page_name.hash(&mut hasher);
        let hashed_page_name = hasher.finish();
        let page_html_path = format!(
            "./{}/page_{:x}/route_{:x}/props.json",
            STONKKS_FOLDER_NAME, hashed_page_name, route_hash
        );
        let mut page_html_file = fs::File::open(page_html_path).await?;
        let mut props = String::new();
        page_html_file.read_to_string(&mut props).await?;
        Ok(props)
    }
}

fn window_object_script(props: &str) -> String {
    format!(
        "window.{0}=window.{0}||{{}};window.{0}.{1}=\'{2}\';",
        STONKKS_WINDOW_OBJECT_KEY, SERIALIZED_PROPS_KEY, props
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

/// the render imports argument is for client rendering, sycamore::render re-render everyhting, so it re-init the wasm file
/// and start an infinite loop.
pub fn default_html_view<G: Html>(
    cx: Scope,
    body: View<G>,
    head: View<G>,
    props: &str,
    render_imports: bool,
) -> View<G> {
    let head = default_head(cx, head, props);
    view! { cx,
        (head)
        body {
            (if render_imports {
                view! { cx,
                    script(type="module") {
                        "import init from '"(CLIENT_JS_FILE_PATH)"';init('"(CLIENT_WASM_FILE_PATH)"');"
                    }
                }
            } else {
                view! { cx, }
            })
            (body)
        }
    }
}
