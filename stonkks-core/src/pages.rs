use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;

use async_trait::async_trait;
use sycamore::prelude::*;

use crate::routes::DynRoutable;
use crate::states::ExtractState;
use crate::states::StatesMap;

use super::pointers::*;
use super::predule::*;

use serde::{Deserialize, Serialize};
use serde_json::Error;

pub type ComponentReactiveProps<'a, T> = <<T as Component>::Props as IntoProps>::ReactiveProps<'a>;

pub trait Component: Send + Sync + 'static {
    type Props: Props;

    fn render<'a, G: Html>(cx: Scope<'a>, props: ComponentReactiveProps<'a, Self>) -> View<G>;
    fn serialize_props(props: &Self::Props) -> Result<String, Error> {
        serde_json::to_string(props)
    }

    fn deserialize_props(serialized_props: &str) -> Result<Self::Props, Error> {
        serde_json::from_str(serialized_props)
    }
    fn render_head<'a, G: Html>(cx: Scope<'a>, props: &Self::Props) -> View<G> {
        let _props = props;
        view! { cx, }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NotFoundPageProps;

impl NotFoundPageProps {
    pub fn to_untyped(self) -> PropsUntypedPtr {
        PropsUntypedPtr::new_not_found_props(self)
    }

    pub fn serialize(&self) -> Result<String, Error> {
        serde_json::to_string(self)
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

pub trait NotFoundPage: Component<Props = NotFoundPageProps> {}

impl<T: Component<Props = NotFoundPageProps>> NotFoundPage for T {}

pub trait Page: Component + Routable {}

impl<T: Component + Routable> Page for T {}

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
        let props = props_ptr.downcast::<T>();
        let head = <T as Component>::render_head(cx, &props);
        let reactive_props = props.into_reactive_props(cx);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn render_server(
        &self,
        cx: Scope,
        props_ptr: PropsUntypedPtr,
    ) -> DynRenderResult<SsrNode> {
        let props = props_ptr.downcast::<T>();
        let head = <T as Component>::render_head(cx, &props);
        let reactive_props = props.into_reactive_props(cx);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn hydrate(
        &self,
        cx: Scope,
        props_ptr: PropsUntypedPtr,
    ) -> DynRenderResult<HydrateNode> {
        let props = props_ptr.downcast::<T>();
        let head = <T as Component>::render_head(cx, &props);
        let reactive_props = props.into_reactive_props(cx);
        let body = <T as Component>::render(cx, reactive_props);
        DynRenderResult { body, head }
    }

    unsafe fn serialize_props(&self, props: &PropsUntypedPtr) -> Result<String, Error> {
        let shared_props = props.downcast_ref::<T>();
        T::serialize_props(shared_props)
    }

    fn deserialize_props(&self, serialized_props: &str) -> Result<PropsUntypedPtr, Error> {
        let props = T::deserialize_props(serialized_props)?;
        let props_ptr = PropsUntypedPtr::new::<T>(props);
        Ok(props_ptr)
    }
}

pub trait DynBasePage: DynComponent + DynRoutable {
    fn as_dyn_component(&self) -> &dyn DynComponent;
}

impl<T: Page> DynBasePage for T {
    fn as_dyn_component(&self) -> &dyn DynComponent {
        self
    }
}

#[async_trait]
pub trait DynPage: Page + Sync {
    type Err<'url>: Debug;
    type State<'r>: ExtractState<'r>;
    async fn get_server_props<'url, 'r>(
        route: Self::Route<'url>,
        states: Self::State<'r>,
    ) -> Result<Self::Props, Self::Err<'url>>;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async unsafe fn get_server_props<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<PropsUntypedPtr, String>;
    fn as_dyn_base_page(&self) -> &dyn DynBasePage;
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T {
    async unsafe fn get_server_props<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<PropsUntypedPtr, String> {
        let route = route_ptr.downcast::<T>();
        let state = states
            .extract::<T::State<'r>>()
            .map_err(|err| format!("Missing state {}.", err))?;
        let props_result = <T as DynPage>::get_server_props(*route, state).await;
        match props_result {
            Ok(props) => Ok(PropsUntypedPtr::new::<T>(props)),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }
}

#[async_trait]
pub trait StaticPage: Page {
    type RouteError: Debug;
    type PropsError<'url>: Debug;

    type RouteState<'r>: ExtractState<'r>;
    type PropsState<'r>: ExtractState<'r>;

    async fn get_props<'url, 'r>(
        route: Self::Route<'url>,
        states: Self::PropsState<'r>,
    ) -> Result<Self::Props, Self::PropsError<'url>>;
    async fn get_build_routes<'r>(
        states: Self::RouteState<'r>,
    ) -> Result<Vec<String>, Self::RouteError>;
}

#[async_trait]
pub trait DynStaticPage: DynBasePage {
    async unsafe fn get_props<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<PropsUntypedPtr, String>;

    async fn get_build_routes(&self, states: &StatesMap) -> Result<Vec<String>, String>;

    fn as_dyn_base_page(&self) -> &dyn DynBasePage;

    unsafe fn hash_route<'url>(&self, route: &RouteUntypedPtr<'url>) -> u64;

    fn get_name(&self) -> &'static str;
}

#[async_trait]
impl<T: StaticPage> DynStaticPage for T {
    async unsafe fn get_props<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<PropsUntypedPtr, String> {
        let route = route_ptr.downcast::<T>();
        let state = states
            .extract::<T::PropsState<'r>>()
            .map_err(|err| format!("Missing state {}.", err))?;
        let props_result = <T as StaticPage>::get_props(*route, state).await;
        match props_result {
            Ok(props) => Ok(PropsUntypedPtr::new::<T>(props)),
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    async fn get_build_routes(&self, states: &'_ StatesMap) -> Result<Vec<String>, String> {
        let states = states
            .extract::<T::RouteState<'_>>()
            .map_err(|err| format!("Missing state {}.", err))?;
        let routes = <T as StaticPage>::get_build_routes(states).await;
        routes.map_err(|err| format!("{:?}", err))
    }

    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }

    unsafe fn hash_route<'url>(&self, route: &RouteUntypedPtr<'url>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let route = route.downcast_ref::<T>();
        route.hash(&mut hasher);
        hasher.finish()
    }

    fn get_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
