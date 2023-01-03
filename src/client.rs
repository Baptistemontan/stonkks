use crate::app::{
    default_html_view, AppInner, ROOT_ELEMENT_ID, SERIALIZED_PROPS_KEY, STONKKS_WINDOW_OBJECT_KEY,
};
use crate::pages::StaticPages;
use crate::utils::PageAndProps;

use super::pages::DynPages;
use super::prelude::*;
use js_sys::{JsString, Object};
use serde_json::Error;
use stonkks_core::layout::DynLayout;
use stonkks_core::pages::{DynBasePage, DynComponent, DynRenderResult};
use stonkks_core::routes::UrlInfos;
use wasm_bindgen::{throw_str, JsValue};
use web_sys::{Element, Window};

fn log(msg: &str) {
    let s = JsString::from(msg);
    web_sys::console::log_1(&s);
}

enum StartupError {
    NoWindow,
    NoProps,
    NoPathname,
    NoStonkksObject,
    PropsNotUTF8,
}

impl StartupError {
    pub fn error_msg(self) -> &'static str {
        match self {
            StartupError::NoWindow => "Unable to aquire the window object.",
            StartupError::NoProps => "No props present in the Stonkks object.",
            StartupError::NoPathname => "Unable to get the pathname.",
            StartupError::NoStonkksObject => "No Stonkks object.",
            StartupError::PropsNotUTF8 => "Props are not UTF8 encoded.",
        }
    }
}

type StartupResult<T> = Result<T, StartupError>;

pub struct Client {
    inner: AppInner,
}

impl From<AppInner> for Client {
    fn from(inner: AppInner) -> Self {
        Client { inner }
    }
}

impl Client {
    fn dyn_pages(&self) -> &DynPages {
        self.inner.dyn_pages()
    }

    fn static_pages(&self) -> &StaticPages {
        self.inner.static_pages()
    }

    fn not_found_page(&self) -> &dyn DynComponent {
        self.inner.not_found_page()
    }

    fn layout(&self) -> &dyn DynLayout {
        self.inner.layout()
    }

    fn find_any_page<'inf, 'url, 'a, I: IntoIterator<Item = &'a dyn DynBasePage>>(
        pages: I,
        url_infos: UrlInfos<'inf, 'url>,
    ) -> Option<&'a dyn DynComponent> {
        pages.into_iter().find_map(|page| {
            page.try_match_route(url_infos)
                .map(|_| page.as_dyn_component())
        })
    }

    fn find_page<'a, 'url>(&self, url_infos: UrlInfos<'a, 'url>) -> &'_ dyn DynComponent {
        let static_pages = self.static_pages().iter_as_base_page();
        let dyn_pages = self.dyn_pages().iter_as_base_page();
        let iter_pages = static_pages.chain(dyn_pages);
        Self::find_any_page(iter_pages, url_infos).unwrap_or(self.not_found_page())
    }

    fn find_page_and_props<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
        serialized_props: &str,
    ) -> Result<PageAndProps<'_>, Error> {
        let page = self.find_page(url_infos);
        PageAndProps::deserialize(page, serialized_props)
    }

    fn prepare_render<'url>(
        &self,
        url: &'url str,
        serialized_props: &str,
    ) -> (PageAndProps<'_>, Element) {
        let url_infos = OwnedUrlInfos::parse_from_url(url);
        let url_infos = url_infos.to_shared();
        let page_and_props = self
            .find_page_and_props(url_infos, serialized_props)
            .expect("Error appened deserializing the props");

        let root = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector(&format!("#{}", ROOT_ELEMENT_ID))
            .unwrap()
            .unwrap();

        (page_and_props, root)
    }

    pub fn render<'url>(&self, url: &'url str, serialized_props: &str) {
        let (page_and_props, root) = self.prepare_render(url, serialized_props);

        root.set_inner_html("");

        sycamore::render_to(
            |cx| {
                let DynRenderResult { body, head } = page_and_props.render_client(cx);
                let body = self.layout().render_client(cx, body);
                default_html_view(cx, body, head, &serialized_props, false)
            },
            &root,
        )
    }

    pub fn hydrate<'url>(&self, url: &'url str, serialized_props: &str) {
        let (page_and_props, root) = self.prepare_render(url, serialized_props);

        sycamore::hydrate_to(
            |cx| {
                let DynRenderResult { body, head } = page_and_props.hydrate(cx);
                let body = self.layout().hydrate(cx, body);
                default_html_view(cx, body, head, &serialized_props, false)
            },
            &root,
        )
    }

    fn get_window() -> StartupResult<Window> {
        web_sys::window().ok_or(StartupError::NoWindow)
    }

    fn get_current_url() -> StartupResult<String> {
        Self::get_window()?
            .location()
            .pathname()
            .map_err(|_| StartupError::NoPathname)
    }

    fn get_stonkks_object() -> StartupResult<Object> {
        Self::get_window()?
            .get(STONKKS_WINDOW_OBJECT_KEY)
            .ok_or(StartupError::NoStonkksObject)
    }

    fn get_serialized_props() -> StartupResult<String> {
        let window_object: JsValue = Self::get_stonkks_object()?.into();
        let props_key = js_sys::JsString::from(SERIALIZED_PROPS_KEY);
        let props_string =
            js_sys::Reflect::get(&window_object, &props_key).map_err(|_| StartupError::NoProps)?;
        props_string.as_string().ok_or(StartupError::PropsNotUTF8)
    }

    fn get_url_and_props() -> StartupResult<(String, String)> {
        let url = Self::get_current_url()?;
        let props = Self::get_serialized_props()?;
        Ok((url, props))
    }

    fn try_run(&self) -> StartupResult<()> {
        let (url, serialized_props) = Self::get_url_and_props()?;
        log("path: ");
        log(&url);
        log("props: ");
        log(&serialized_props);
        log("start hydrate.");
        self.hydrate(&url, &serialized_props);
        // self.render(&url, &serialized_props);
        log("hydrate finished.");
        Ok(())
    }

    pub fn run(&self) {
        if let Err(err) = self.try_run() {
            throw_str(err.error_msg());
        }
    }
}
