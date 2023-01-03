use crate::api::ApiRoutes;
use crate::app::{default_html_view, AppInner, ROOT_ELEMENT_ID};
use crate::pages::StaticPages;
use crate::utils::PageAndProps;

use super::pages::DynPages;
use super::prelude::*;
use stonkks_core::layout::DynLayout;
use stonkks_core::pages::{DynComponent, DynRenderResult};
use stonkks_core::response::Response;
use stonkks_core::routes::UrlInfos;
use stonkks_core::states::StatesMap;

const API_ROUTE_SEGMENT: &str = "api";
const STATIC_FILES_ROUTE_SEGMENT: &str = "public";
const PROPS_ROUTE_SEGMENT: &str = "props";

pub struct Server {
    inner: AppInner,
    states: StatesMap,
    api: ApiRoutes,
}

pub enum ServerResponse {
    Props(String),
    Html(String),
    Api(Response),
}

impl Server {
    pub fn new(inner: AppInner, api: ApiRoutes, states: StatesMap) -> Self {
        Server { inner, api, states }
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

    async fn try_find_page_and_props<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<Result<PageAndProps<'_>, String>> {
        if let Some(page) = self.static_pages().find_static_page(url_infos) {
            return Some(Ok(page.get_props()));
        }
        if let Some(result) = self
            .dyn_pages()
            .find_dyn_page_and_props(url_infos, &self.states)
            .await
        {
            return Some(result);
        }
        None
    }

    pub async fn try_render_to_string<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<Result<String, String>> {
        let result = self.try_find_page_and_props(url_infos).await?;
        let page_and_props = match result {
            Ok(page_and_props) => page_and_props,
            Err(err) => return Some(Err(err)),
        };
        let serialize_result = page_and_props.serialize_props();
        let serialized_props = match serialize_result {
            Ok(s) => s,
            Err(err) => return Some(Err(format!("{:?}", err))),
        };
        let html = sycamore::render_to_string(|cx| {
            let DynRenderResult { body, head } = page_and_props.render_server(cx);
            let body = self.layout().render_server(cx, body);
            default_html_view(cx, body, head, &serialized_props, true)
        });
        Some(Ok(format!(
            "<!DOCTYPE html><html id=\"{}\">{}</html>",
            ROOT_ELEMENT_ID, html
        )))
    }

    pub fn render_not_found(&self) -> Result<String, String> {
        let not_found_page_props = NotFoundPageProps::new();
        let not_found_page = self.not_found_page();
        let serialized_props = not_found_page_props
            .serialize()
            .map_err(|err| format!("{:?}", err))?;
        let html = sycamore::render_to_string(|cx| {
            let DynRenderResult { body, head } =
                unsafe { not_found_page.render_server(cx, not_found_page_props.to_untyped()) };
            let body = self.layout().render_server(cx, body);
            default_html_view(cx, body, head, &serialized_props, true)
        });
        Ok(format!(
            "<!DOCTYPE html><html id=\"{}\">{}</html>",
            ROOT_ELEMENT_ID, html
        ))
    }

    async fn try_find_props<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<Result<String, String>> {
        let result = self.try_find_page_and_props(url_infos).await?;
        let page_and_props = match result {
            Ok(page_and_props) => page_and_props,
            Err(err) => return Some(Err(err)),
        };
        let serialize_result = page_and_props.serialize_props();
        Some(serialize_result.map_err(|err| format!("{:?}", err)))
    }

    pub async fn respond<'url>(
        &self,
        url_infos: &OwnedUrlInfos<'url>,
    ) -> Option<Result<ServerResponse, String>> {
        match url_infos.to_shared_shifted() {
            Some((PROPS_ROUTE_SEGMENT, url_infos)) => {
                // props API
                self.try_find_props(url_infos)
                    .await
                    .transpose()
                    .map(|props| props.map(ServerResponse::Props))
                    .transpose()
            }
            Some((STATIC_FILES_ROUTE_SEGMENT, _)) => None, // static file
            Some((API_ROUTE_SEGMENT, url_infos)) => {
                // api route
                self.api
                    .find_and_respond(url_infos, &self.states)
                    .await
                    .transpose()
                    .map(|response| response.map(ServerResponse::Api))
                    .transpose()
            }
            _ => {
                // possible page
                self.try_render_to_string(url_infos.to_shared())
                    .await
                    .transpose()
                    .map(|html| html.map(ServerResponse::Html))
                    .transpose()
            }
        }
    }
}
