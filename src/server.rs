use crate::api::ApiRoutes;
use crate::app::{default_html_view, AppInner, ROOT_ELEMENT_ID};
use crate::pages::StaticPages;

use super::pages::DynPages;
use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::{DynComponent, DynRenderResult};
use next_rs_traits::pointers::*;

pub struct Server {
    inner: AppInner,
    api: ApiRoutes,
}

pub enum Response {
    Html(String),
    Api(String),
}

impl Server {
    pub(crate) fn new(inner: AppInner, api: ApiRoutes) -> Self {
        Server { inner, api }
    }

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

    pub async fn try_find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynComponent, PropsUntypedPtr)> {
        if let Some(page) = self.static_pages().find_static_page(url_infos) {
            return Some((page.as_dyn_component(), PropsUntypedPtr::new_unit()));
        }
        if let Some((page, props)) = self.dyn_pages().find_dyn_page_and_props(url_infos).await {
            return Some((page.as_dyn_component(), props));
        }
        None
    }

    pub async fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        self.try_find_page_and_props(url_infos)
            .await
            .unwrap_or_else(|| (self.not_found_page(), NotFoundPageProps::new_untyped()))
    }

    pub async fn try_render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<String> {
        let (page, props) = self.try_find_page_and_props(url_infos).await?;
        let serialized_props = unsafe { page.serialize_props(&props).unwrap() };
        let html = sycamore::render_to_string(|cx| {
            let DynRenderResult { body, head } = unsafe { page.render_server(cx, props) };
            let body = self.layout().render_server(cx, body);
            default_html_view(cx, body, head, &serialized_props)
        });
        Some(format!(
            "<!DOCTYPE html><html id=\"{}\">{}</html>",
            ROOT_ELEMENT_ID, html
        ))
    }

    pub fn render_not_found(&self) -> String {
        let (page, props) = (self.not_found_page(), NotFoundPageProps::new_untyped());
        let serialized_props = unsafe { page.serialize_props(&props).unwrap() };
        let html = sycamore::render_to_string(|cx| {
            let DynRenderResult { body, head } = unsafe { page.render_server(cx, props) };
            let body = self.layout().render_server(cx, body);
            default_html_view(cx, body, head, &serialized_props)
        });
        format!(
            "<!DOCTYPE html><html id=\"{}\">{}</html>",
            ROOT_ELEMENT_ID, html
        )
    }

    pub async fn respond<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<Response> {
        if url_infos.segments().first() == Some(&"api") {
            self.api
                .find_and_respond(url_infos)
                .await
                .map(Response::Api)
        } else {
            self.try_render_to_string(url_infos)
                .await
                .map(Response::Html)
        }
    }
}
