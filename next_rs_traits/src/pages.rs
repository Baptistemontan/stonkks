use async_trait::async_trait;
// use sycamore::{reactive::Scope, view::View};
use sycamore::prelude::*;

use super::pointers::*;
use super::predule::*;

pub trait Component {
    type Props: Send + 'static;

    fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G>;
}

pub struct NotFoundPageProps;

pub trait NotFoundPage: Component<Props = NotFoundPageProps> + 'static {}

impl<T: Component<Props = NotFoundPageProps> + 'static> NotFoundPage for T {}

pub trait Page: Component {
    type Route<'a>: Route<'a>;

    fn try_match_route<'url>(url_infos: &UrlInfos<'url>) -> Option<Self::Route<'url>> {
        Self::Route::try_from_url(url_infos)
    }
}

pub trait DynComponent {
    unsafe fn render_client(&self, cx: Scope, props: PropsUntypedPtr) -> View<DomNode>;
    unsafe fn render_server(&self, cx: Scope, props: PropsUntypedPtr) -> View<SsrNode>;
    unsafe fn hydrate(&self, cx: Scope, props: PropsUntypedPtr) -> View<HydrateNode>;
}

impl<T: Component> DynComponent for T {
    unsafe fn render_client(&self, cx: Scope, props_ptr: PropsUntypedPtr) -> View<DomNode> {
        let props_casted_ptr: PropsCastedPtr<T> = props_ptr.into();
        let props = props_casted_ptr.into_inner();
        <T as Component>::render(cx, props)
    }

    unsafe fn render_server(&self, cx: Scope, props_ptr: PropsUntypedPtr) -> View<SsrNode> {
        let props_casted_ptr: PropsCastedPtr<T> = props_ptr.into();
        let props = props_casted_ptr.into_inner();
        <T as Component>::render(cx, props)
    }

    unsafe fn hydrate(&self, cx: Scope, props_ptr: PropsUntypedPtr) -> View<HydrateNode> {
        let props_casted_ptr: PropsCastedPtr<T> = props_ptr.into();
        let props = props_casted_ptr.into_inner();
        <T as Component>::render(cx, props)
    }
}

pub trait DynBasePage: DynComponent {
    unsafe fn try_match_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<RouteUntypedPtr<'url>>;

    fn as_dyn_component(&self) -> &dyn DynComponent;
}

impl<T: Page> DynBasePage for T {
    unsafe fn try_match_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<RouteUntypedPtr<'url>> {
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
    ) -> PropsUntypedPtr; // IndexRoute -> IndexPageProps
    fn as_dyn_base_page(&self) -> &dyn DynBasePage;
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T {
    async unsafe fn get_server_props<'url>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
    ) -> PropsUntypedPtr {
        let route_casted_ptr: RouteCastedPtr<T> = route_ptr.into();
        let route = route_casted_ptr.into_inner();
        let props = <T as DynPage>::get_server_props(route).await;
        PropsUntypedPtr::new::<T>(props)
    }

    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }
}
