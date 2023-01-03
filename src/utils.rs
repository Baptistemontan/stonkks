use stonkks_core::{
    pages::{DynComponent, DynPageDyn, DynRenderResult, DynStaticPage},
    pointers::{PropsUntypedPtr, RouteUntypedPtr},
    routes::UrlInfos,
    states::StatesMap,
};
use sycamore::prelude::*;

pub(crate) struct DynPageAndRoute<'a, 'url> {
    page: &'a dyn DynPageDyn,
    route: RouteUntypedPtr<'url>,
}

impl<'a, 'url> DynPageAndRoute<'a, 'url> {
    pub fn try_match_route(
        page: &'a dyn DynPageDyn,
        url_infos: UrlInfos<'_, 'url>,
    ) -> Option<Self> {
        let route = page.try_match_route(url_infos)?;
        Some(DynPageAndRoute { page, route })
    }

    pub async fn get_props(self, states: &StatesMap) -> Result<PageAndProps<'a>, String> {
        let props = unsafe { self.page.get_server_props(self.route, states).await? };
        Ok(PageAndProps {
            page: self.page.as_dyn_component(),
            props,
        })
    }
}

pub(crate) struct StaticPageAndRoute<'a, 'url> {
    page: &'a dyn DynStaticPage,
    #[allow(unused)]
    route: RouteUntypedPtr<'url>,
}

impl<'a, 'url> StaticPageAndRoute<'a, 'url> {
    pub fn try_match_route(
        page: &'a dyn DynStaticPage,
        url_infos: UrlInfos<'_, 'url>,
    ) -> Option<Self> {
        let route = page.try_match_route(url_infos)?;
        Some(StaticPageAndRoute { page, route })
    }

    pub async fn get_props(self, states: &StatesMap) -> Result<PageAndProps<'a>, String> {
        let props = unsafe { self.page.get_props(self.route, states).await? };
        Ok(PageAndProps {
            page: self.page.as_dyn_component(),
            props,
        })
    }

    pub fn hash_route(&self) -> u64 {
        unsafe { self.page.hash_route(&self.route) }
    }

    pub fn page_name(&self) -> &'static str {
        self.page.get_name()
    }
}

pub(crate) struct PageAndProps<'a> {
    page: &'a dyn DynComponent,
    props: PropsUntypedPtr,
}

impl<'a> PageAndProps<'a> {
    pub fn deserialize(
        page: &'a dyn DynComponent,
        serialized_props: &str,
    ) -> Result<Self, serde_json::Error> {
        let props = page.deserialize_props(serialized_props)?;
        Ok(PageAndProps { page, props })
    }

    pub fn serialize_props(&self) -> Result<String, String> {
        let result = unsafe { self.page.serialize_props(&self.props) };
        result.map_err(|err| format!("{:?}", err))
    }

    pub fn render_client(self, cx: Scope) -> DynRenderResult<DomNode> {
        unsafe { self.page.render_client(cx, self.props) }
    }

    pub fn render_server(self, cx: Scope) -> DynRenderResult<SsrNode> {
        unsafe { self.page.render_server(cx, self.props) }
    }

    pub fn hydrate(self, cx: Scope) -> DynRenderResult<HydrateNode> {
        unsafe { self.page.hydrate(cx, self.props) }
    }
}
