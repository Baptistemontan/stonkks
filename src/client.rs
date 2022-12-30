use crate::app::{
    default_html_view, AppInner, NEXT_RS_WINDOW_OBJECT_KEY, ROOT_ELEMENT_ID, SERIALIZED_PROPS_KEY,
};

use super::pages::DynPages;
use super::prelude::*;
use js_sys::{Object, JsString};
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::{DynBasePage, DynComponent, DynRenderResult};
use next_rs_traits::pointers::*;
use serde_json::Error;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

fn log(msg: &str) {
    let s = JsString::from(msg);
    web_sys::console::log_1(&s);
}

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

    fn not_found_page(&self) -> &dyn DynComponent {
        self.inner.not_found_page()
    }

    fn layout(&self) -> &dyn DynLayout {
        self.inner.layout()
    }

    fn find_any_page<'url, 'a, I: IntoIterator<Item = &'a dyn DynBasePage>>(
        pages: I,
        url_infos: &UrlInfos<'url>,
    ) -> Option<&'a dyn DynComponent> {
        pages.into_iter().find_map(|page| {
            page.try_match_route(url_infos)
                .map(|_| page.as_dyn_component())
        })
    }

    fn find_page<'url>(&self, url_infos: &UrlInfos<'url>) -> &'_ dyn DynComponent {
        let dyn_pages = self.dyn_pages().iter_as_base_page();
        Self::find_any_page(dyn_pages, url_infos).unwrap_or(self.not_found_page())
    }

    fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
        serialized_props: &str,
    ) -> Result<(&'_ dyn DynComponent, PropsUntypedPtr), Error> {
        let page = self.find_page(url_infos);
        let props = page.deserialize_props(serialized_props)?;
        Ok((page, props))
    }

    pub fn hydrate<'url>(&self, url: &'url str, serialized_props: &str) {
        let url_infos = UrlInfos::parse_from_url(url);
        let (page, props) = self
            .find_page_and_props(&url_infos, serialized_props)
            .expect("Error appened deserializing the props");

        let root = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector(&format!("#{}", ROOT_ELEMENT_ID))
            .unwrap()
            .unwrap();

        sycamore::hydrate_to(
            |cx| {
                let DynRenderResult { body, head } = unsafe { page.hydrate(cx, props) };
                let body = self.layout().hydrate(cx, body);
                default_html_view(cx, body, head, &serialized_props)
            },
            &root,
        )
    }

    fn get_current_url() -> Option<String> {
        web_sys::window()?.location().pathname().ok()
    }

    fn get_window_object() -> Option<Object> {
        web_sys::window()?.get(NEXT_RS_WINDOW_OBJECT_KEY)
    }

    fn get_serialized_props() -> Option<String> {
        let window_object: JsValue = Self::get_window_object()?.into();
        let props_key = js_sys::JsString::from(SERIALIZED_PROPS_KEY);
        let props_string = js_sys::Reflect::get(&window_object, &props_key).ok()?;
        props_string.as_string()
    }

    fn get_url_and_props() -> Option<(String, String)> {
        let url = Self::get_current_url()?;
        let props = Self::get_serialized_props()?;
        Some((url, props))
    }

    fn try_run(&self) -> Option<()> {
        let (url, serialized_props) = Self::get_url_and_props()?;
        log("path: ");
        log(&url);
        log("props: ");
        log(&serialized_props);
        log("start hydrate.");
        self.hydrate(&url, &serialized_props);
        log("hydrate finished.");
        Some(())
    }

    pub fn run(&self) {
        self.try_run().unwrap_throw();
    }
}
