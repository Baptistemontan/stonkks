use async_trait::async_trait;
// use sycamore::{reactive::Scope, view::View};
use sycamore::prelude::*;

use super::pointers::*;
use super::predule::*;

pub trait Component {
    type Props;

    fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G>;
}

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
    unsafe fn try_match_route(&self, url_infos: &UrlInfos) -> Option<RouteUntypedPtr>;
}

impl<T: Page> DynBasePage for T {
    unsafe fn try_match_route(&self, url_infos: &UrlInfos) -> Option<RouteUntypedPtr> {
        let route = <T as Page>::try_match_route(url_infos)?;
        let route_ptr = RouteUntypedPtr::new::<T>(route);
        Some(route_ptr)
    }
}

#[async_trait]
pub trait DynPage: Page + Sync
where
    Self::Props: Send,
{
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async unsafe fn get_server_props(&self, route_ptr: RouteUntypedPtr) -> PropsUntypedPtr; // IndexRoute -> IndexPageProps
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T
where
    T::Props: Send,
{
    async unsafe fn get_server_props(&self, route_ptr: RouteUntypedPtr) -> PropsUntypedPtr {
        let route_casted_ptr: RouteCastedPtr<T> = route_ptr.into();
        let route = route_casted_ptr.into_inner();
        let props = <T as DynPage>::get_server_props(route).await;
        PropsUntypedPtr::new::<T>(props)
    }
}
