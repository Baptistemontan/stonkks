use async_trait::async_trait;
use sycamore::prelude::*;

use super::pointers::*;
use super::predule::*;

use serde::{Deserialize, Serialize};
use serde_json::Error;

pub type ComponentReactiveProps<'a, T> = <<T as Component>::Props as IntoProps>::ReactiveProps<'a>;

pub trait Component: Send + Sync {
    type Props: Props;

    fn render<'a, G: Html>(cx: Scope<'a>, props: ComponentReactiveProps<'a, Self>) -> View<G>;
    fn serialize_props(props: &Self::Props) -> Result<String, Error> {
        serde_json::to_string(props)
    }

    fn deserialize_props(serialized_props: &str) -> Result<Self::Props, Error> {
        serde_json::from_str(serialized_props)
    }

    fn render_head<'a, G: Html>(
        cx: Scope<'a>,
        props: &ComponentReactiveProps<'a, Self>,
    ) -> View<G> {
        let _props = props;
        view! { cx, }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NotFoundPageProps;

impl NotFoundPageProps {
    pub fn new_untyped() -> PropsUntypedPtr {
        let props = Self::new();
        PropsUntypedPtr::new_not_found_props(props)
    }

    pub fn new() -> Self {
        NotFoundPageProps
    }
}

pub struct NotFountPageReactiveProps;

impl ReactiveProps<'_> for NotFountPageReactiveProps {
    type Props = NotFoundPageProps;
}

impl IntoProps for NotFoundPageProps {
    type ReactiveProps<'a> = NotFountPageReactiveProps;

    fn into_reactive_props<'a>(self, _cx: Scope<'a>) -> Self::ReactiveProps<'a> {
        NotFountPageReactiveProps
    }
}

impl Props for NotFoundPageProps {}

pub trait NotFoundPage: Component<Props = NotFoundPageProps> + 'static {}

impl<T: Component<Props = NotFoundPageProps> + 'static> NotFoundPage for T {}

pub trait Page: Component {
    type Route<'a>: Route<'a>;

    fn try_match_route<'url>(url_infos: &UrlInfos<'url>) -> Option<Self::Route<'url>> {
        Self::Route::try_from_url(url_infos)
    }
}

pub struct DynRenderResult<G: Html> {
    pub body: View<G>,
    pub head: View<G>,
}

pub trait DynComponent: Send + Sync {
    unsafe fn render_client(&self, cx: Scope, props: PropsUntypedPtr) -> DynRenderResult<DomNode>;
    unsafe fn render_server(&self, cx: Scope, props: PropsUntypedPtr) -> DynRenderResult<SsrNode>;
    unsafe fn hydrate(&self, cx: Scope, props: PropsUntypedPtr) -> DynRenderResult<HydrateNode>;

    unsafe fn serialize_props(&self, props: &PropsUntypedPtr) -> Result<String, Error>;
    fn deserialize_props(&self, serialized_props: &str) -> Result<PropsUntypedPtr, Error>;
}

impl<T: Component> DynComponent for T {
    unsafe fn render_client(
        &self,
        cx: Scope,
        props_ptr: PropsUntypedPtr,
    ) -> DynRenderResult<DomNode> {
        let props = props_ptr.cast::<T>();
        let reactive_props = props.into_reactive_props(cx);
        let head = <T as Component>::render_head(cx, &reactive_props);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn render_server(
        &self,
        cx: Scope,
        props_ptr: PropsUntypedPtr,
    ) -> DynRenderResult<SsrNode> {
        let props = props_ptr.cast::<T>();
        let reactive_props = props.into_reactive_props(cx);
        let head = <T as Component>::render_head(cx, &reactive_props);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn hydrate(
        &self,
        cx: Scope,
        props_ptr: PropsUntypedPtr,
    ) -> DynRenderResult<HydrateNode> {
        let props = props_ptr.cast::<T>();
        let reactive_props = props.into_reactive_props(cx);
        let head = <T as Component>::render_head(cx, &reactive_props);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn serialize_props(&self, props: &PropsUntypedPtr) -> Result<String, Error> {
        let shared_props = props.shared_cast::<T>();
        T::serialize_props(shared_props)
    }

    fn deserialize_props(&self, serialized_props: &str) -> Result<PropsUntypedPtr, Error> {
        let props = T::deserialize_props(serialized_props)?;
        let props_ptr = PropsUntypedPtr::new::<T>(props);
        Ok(props_ptr)
    }
}

pub trait DynBasePage: DynComponent {
    fn try_match_route<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<RouteUntypedPtr<'url>>;

    fn as_dyn_component(&self) -> &dyn DynComponent;
}

impl<T: Page> DynBasePage for T {
    fn try_match_route<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<RouteUntypedPtr<'url>> {
        let route = <T as Page>::try_match_route(url_infos)?;
        let route_ptr = RouteUntypedPtr::new::<T>(route);
        Some(route_ptr)
    }

    fn as_dyn_component(&self) -> &dyn DynComponent {
        self
    }
}

#[async_trait]
pub trait DynPage: Page + Sync {
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async unsafe fn get_server_props<'url>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
    ) -> PropsUntypedPtr;
    fn as_dyn_base_page(&self) -> &dyn DynBasePage;
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T {
    async unsafe fn get_server_props<'url>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
    ) -> PropsUntypedPtr {
        let route = route_ptr.cast::<T>();
        let props = <T as DynPage>::get_server_props(*route).await;
        PropsUntypedPtr::new::<T>(props)
    }

    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }
}

pub trait StaticPage: Page<Props = ()> { }

pub trait DynStaticPage: DynBasePage {
    fn as_dyn_base_page(&self) -> &dyn DynBasePage;
}

impl<T: Page<Props = ()>> StaticPage for T { }

impl<T: StaticPage> DynStaticPage for T {
    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }
}
