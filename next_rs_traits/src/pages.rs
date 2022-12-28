use async_trait::async_trait;
// use sycamore::{reactive::Scope, view::View};
use sycamore::prelude::*;

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

pub mod pages_ptr {
    use super::{Page, Component};

    // Those pointer wrappers garanties that they have exclusive acces to the underlying pointer,
    // because they can only be created by either consuming one another
    // or by consumming the value, boxing it and then leaking the box, taking exclusive ownership of the pointer.
    // They are finally consummed when converting to the inner type.
    // They are made for moving data in a untyped, unlifetimed(?) maner for props and routes.

    // TODO: implement drop for them and deallocating the box in case it is dropped without being consummed
    // (early return, panic, future cancelling, ect...)
    // and use mem::forget when consuming.
    // ! After further research, you need recreate the box to the correct type for it to deallocate.
    // So Drop can be implemented for CastedPtr, but not for UntypedPtr.
    // At least not in the current implementation, need further research but maybe UntypedPtr can
    // hold the Layout of the type and use that to free it.
    // ! Forget that, we need the concrete type for dropping the inner type.
    // Other possibility would be to have dyn Any pointer, and recreating a Box<dyn Any> for deallocating
    // The Vtable would have the drop function.

    // Route ptr wrapper:

    pub struct RouteCastedPtr<'a, T: Page>(*mut T::Route<'a>);
    pub struct RouteUntypedPtr(*mut ());

    impl<'a, T: Page> From<RouteUntypedPtr> for RouteCastedPtr<'a, T> {
        fn from(RouteUntypedPtr(route_ptr): RouteUntypedPtr) -> Self {
            RouteCastedPtr(route_ptr as *mut _)
        }
    }

    impl<'a, T: Page> RouteCastedPtr<'a, T> {
        pub unsafe fn into_inner(self) -> T::Route<'a> {
            let route = Box::from_raw(self.0);
            *route
        }
    }

    unsafe impl<'a, T: Page> Send for RouteCastedPtr<'a, T> where T::Route<'a>: Send {}

    impl RouteUntypedPtr {
        pub fn new<'a, T: Page>(route: T::Route<'a>) -> Self
            where T::Route<'a>: Send
        {
            let boxed_route = Box::new(route);
            let ptr = Box::leak(boxed_route) as *mut _ as *mut ();
            RouteUntypedPtr(ptr)
        }

        pub unsafe fn cast<'a, T: Page>(self) -> RouteCastedPtr<'a, T> {
            self.into()
        }
    }

    // RouteUntypePtr can only be constructed if the concrete type implement Send
    unsafe impl Send for RouteUntypedPtr {}

    // Props ptr wrapper:

    pub struct PropsCastedPtr<T: Component>(pub *mut T::Props);
    pub struct PropsUntypedPtr(pub *mut ());

    impl<T: Component> From<PropsUntypedPtr> for PropsCastedPtr<T> {
        fn from(PropsUntypedPtr(props_ptr): PropsUntypedPtr) -> Self {
            PropsCastedPtr(props_ptr as *mut _)
        }
    }

    impl<T: Component> PropsCastedPtr<T> {
        pub unsafe fn into_inner(self) -> T::Props {
            let props = Box::from_raw(self.0);
            *props
        }
    }

    unsafe impl<T: Component> Send for PropsCastedPtr<T> where T::Props: Send {}

    impl PropsUntypedPtr {
        pub fn new<T: Page>(props: T::Props) -> Self
            where T::Props: Send
        {
            let boxed_props = Box::new(props);
            let ptr = Box::leak(boxed_props) as *mut _ as *mut ();
            PropsUntypedPtr(ptr)
        }

        pub unsafe fn cast<T: Page>(self) -> PropsCastedPtr<T> {
            self.into()
        }
    }

    // same as RouteUntypePtr, can only be created if T is send
    unsafe impl Send for PropsUntypedPtr {}
}

use pages_ptr::*;

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
    where Self::Props: Send,
{
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async unsafe fn get_server_props(&self, route_ptr: RouteUntypedPtr) -> PropsUntypedPtr; // IndexRoute -> IndexPageProps
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T
    where T::Props: Send
{
    async unsafe fn get_server_props(&self, route_ptr: RouteUntypedPtr) -> PropsUntypedPtr {
        let route_casted_ptr: RouteCastedPtr<T> = route_ptr.into();
        let route = route_casted_ptr.into_inner();
        let props = <T as DynPage>::get_server_props(route).await;
        PropsUntypedPtr::new::<T>(props)
    }
}

#[cfg(test)]
mod test {
    use sycamore::render_to_string;

    use super::*;

    struct MyPage;

    struct MyRoute;

    impl<'a> Route<'a> for MyRoute {
        fn try_from_url(url: &UrlInfos<'a>) -> Option<Self> {
            let mut iter = url.segments().iter();

            match (iter.next(), iter.next()) {
                (Some(value), None) if value == &"index" => Some(MyRoute),
                _ => None,
            }
        }
    }

    impl Component for MyPage {
        type Props = ();

        fn render<G: Html>(cx: Scope, _props: Self::Props) -> View<G> {
            view! { cx,
                p {
                    "Greetings!"
                }
            }
        }
    }

    impl Page for MyPage {
        type Route<'a> = MyRoute;
    }

    #[test]
    fn test() {
        let page = MyPage;
        let dyn_page: Box<dyn DynBasePage> = Box::new(page);
        let url_infos = UrlInfos::parse_from_url("/about");
        unsafe {
            assert!(dyn_page.try_match_route(&url_infos).is_none());
        }
        let url_infos = UrlInfos::parse_from_url("/index/other");
        unsafe {
            assert!(dyn_page.try_match_route(&url_infos).is_none());
        }
        let url_infos = UrlInfos::parse_from_url("/index");
        unsafe {
            assert!(dyn_page.try_match_route(&url_infos).is_some());
        }

        let dyn_ssr_view = render_to_string(|cx| unsafe {
            dyn_page.render_server(cx, PropsUntypedPtr::new::<MyPage>(()))
        });
        let ssr_view = render_to_string(|cx| MyPage::render(cx, ()));

        assert_eq!(dyn_ssr_view, ssr_view);
        assert!(ssr_view.contains("Greetings!"));
    }
}
