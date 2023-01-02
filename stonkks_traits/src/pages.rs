use std::fmt::Debug;

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
        let props = props_ptr.cast::<T>();
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
        let props = props_ptr.cast::<T>();
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
        let props = props_ptr.cast::<T>();
        let head = <T as Component>::render_head(cx, &props);
        let reactive_props = props.into_reactive_props(cx);
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
        let route = route_ptr.cast::<T>();
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

pub trait StaticPage: Page<Props = ()> {}

pub trait DynStaticPage: DynBasePage {
    fn as_dyn_base_page(&self) -> &dyn DynBasePage;
}

impl<T: Page<Props = ()>> StaticPage for T {}

impl<T: StaticPage> DynStaticPage for T {
    fn as_dyn_base_page(&self) -> &dyn DynBasePage {
        self
    }
}
