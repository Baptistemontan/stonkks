use crate::app::{default_html_view, AppInner, ROOT_ELEMENT_ID};
use crate::pages::StaticPages;

use super::pages::DynPages;
use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::{DynComponent, DynRenderResult};
use next_rs_traits::pointers::*;

pub struct Server {
    inner: AppInner,
}

impl From<AppInner> for Server {
    fn from(inner: AppInner) -> Self {
        Server { inner }
    }
}

impl Server {
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

    pub async fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        if let Some(page) = self.static_pages().find_static_page(url_infos) {
            return (page.as_dyn_component(), PropsUntypedPtr::new_unit());
        }
        if let Some((page, props)) = self.dyn_pages().find_dyn_page_and_props(url_infos).await {
            return (page.as_dyn_component(), props);
        }
        (self.not_found_page(), NotFoundPageProps::new_untyped())
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> String {
        let (page, props) = self.find_page_and_props(url_infos).await;
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
}
