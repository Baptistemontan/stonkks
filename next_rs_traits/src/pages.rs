use async_trait::async_trait;
// use sycamore::{reactive::Scope, view::View};
use std::sync::atomic::AtomicPtr;
use sycamore::prelude::*;

use super::predule::*;

pub trait BasePage {
    type Route<'a>: Route<'a>;
    type Props: Send;

    fn try_match_route<'url>(url_infos: &UrlInfos<'url>) -> Option<Self::Route<'url>> {
        Self::Route::try_from_url(url_infos)
    }

    fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G>;
}

pub mod pages_ptr {
    use std::sync::atomic::AtomicPtr;
    use super::BasePage;

    pub type RouteCastedPtr<'a, T> = * mut <T as BasePage>::Route<'a>;
    pub type PropsCastedPtr<T> = * mut <T as BasePage>::Props;
    
    pub type UntypedPtr = * mut ();
    pub type RouteDynPtr = UntypedPtr;
    pub type PropsDynPtr = UntypedPtr;
    
    pub type UntypedSendPtr = AtomicPtr<()>;
    pub type RouteDynSendPtr = UntypedSendPtr;
    pub type PropsDynSendPtr = UntypedSendPtr;
}

use pages_ptr::*;



pub trait DynBasePage {
    unsafe fn try_match_route(&self, url_infos: &UrlInfos) -> Option<RouteDynPtr>;
    unsafe fn render_client(&self, cx: Scope, props: PropsDynPtr) -> View<DomNode>;
    unsafe fn render_server(&self, cx: Scope, props: PropsDynPtr) -> View<SsrNode>;
    unsafe fn hydrate(&self, cx: Scope, props: PropsDynPtr) -> View<HydrateNode>;
}

impl<T: BasePage> DynBasePage for T {
    unsafe fn try_match_route(&self, url_infos: &UrlInfos) -> Option<RouteDynPtr> {
        let route = <T as BasePage>::try_match_route(url_infos)?;
        let route = Box::new(route);
        
        let route = Box::leak(route) as RouteCastedPtr<T> as RouteDynPtr;
        Some(route)
    }

    unsafe fn render_client(&self, cx: Scope, props: PropsDynPtr) -> View<DomNode> {
        let props = props as PropsCastedPtr<T>;
        let props = Box::from_raw(props as * mut _); 
        <T as BasePage>::render(cx, *props)
    }

    unsafe fn render_server(&self, cx: Scope, props: PropsDynPtr) -> View<SsrNode> {
        let props = props as PropsCastedPtr<T>;
        let props = Box::from_raw(props as * mut _); 
        <T as BasePage>::render(cx, *props)
    }

    unsafe fn hydrate(&self, cx: Scope, props: PropsDynPtr) -> View<HydrateNode> {
        let props = props as PropsCastedPtr<T>;
        let props = Box::from_raw(props as * mut _); 
        <T as BasePage>::render(cx, *props)
    }
}

#[async_trait]
pub trait DynPage: BasePage + Sync {
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async unsafe fn get_server_props(&self, route: RouteDynSendPtr) -> PropsDynSendPtr; // IndexRoute -> IndexPageProps
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T {
    async unsafe fn get_server_props(&self, route: RouteDynSendPtr) -> PropsDynSendPtr {
        let route = {
            let route_ptr = route.into_inner() as RouteCastedPtr<T>;
            let route = Box::from_raw(route_ptr);
            *route
        };
        let props = <T as DynPage>::get_server_props(route).await;
        let props = Box::new(props);
        let props = Box::leak(props) as PropsCastedPtr<T> as UntypedPtr;
        AtomicPtr::new(props)
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

    impl BasePage for MyPage {
        type Route<'a> = MyRoute;

        type Props = ();

        fn render<G: Html>(cx: Scope, _props: Self::Props) -> View<G> {
            view! { cx,
                p {
                    "Greetings!"
                }
            }
        }
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
            dyn_page.render_server(cx, Box::leak(Box::new(())))
        });
        let ssr_view = render_to_string(|cx| MyPage::render(cx, ()));

        assert_eq!(dyn_ssr_view, ssr_view);
        assert!(ssr_view.contains("Greetings!"));
    }
}
