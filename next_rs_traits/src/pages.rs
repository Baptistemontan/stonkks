use async_trait::async_trait;
// use sycamore::{reactive::Scope, view::View};
use std::any::Any;
use sycamore::prelude::*;

use super::predule::*;

pub trait BasePage: Any + Sized {
    type Route: Route;
    type Props: Send;

    fn try_match_route(url_infos: &UrlInfos) -> Option<Self::Route> {
        Self::Route::try_from_url(url_infos)
    }

    fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G>;
}

pub trait DynBasePage {
    fn try_match_route(&self, url_infos: &UrlInfos) -> Option<Box<dyn Any + Send>>;
    fn render_client(&self, cx: Scope, props: Box<dyn Any>) -> View<DomNode>;
    fn render_server(&self, cx: Scope, props: Box<dyn Any>) -> View<SsrNode>;
    fn hydrate(&self, cx: Scope, props: Box<dyn Any>) -> View<HydrateNode>;
}

impl<T: BasePage> DynBasePage for T {
    fn try_match_route(&self, url_infos: &UrlInfos) -> Option<Box<dyn Any + Send>> {
        let route = <T as BasePage>::try_match_route(url_infos)?;
        Some(Box::new(route))
    }

    fn render_client(&self, cx: Scope, props: Box<dyn Any>) -> View<DomNode> {
        let props = props
            .downcast::<T::Props>()
            .expect("An error occured when downcasting a dyn Any props");
        <T as BasePage>::render(cx, *props)
    }

    fn render_server(&self, cx: Scope, props: Box<dyn Any>) -> View<SsrNode> {
        let props = props
            .downcast::<T::Props>()
            .expect("An error occured when downcasting a dyn Any props");
        <T as BasePage>::render(cx, *props)
    }

    fn hydrate(&self, cx: Scope, props: Box<dyn Any>) -> View<HydrateNode> {
        let props = props
            .downcast::<T::Props>()
            .expect("An error occured when downcasting a dyn Any props");
        <T as BasePage>::render(cx, *props)
    }
}

#[async_trait]
pub trait DynPage: BasePage + Sync {
    async fn get_server_props(route: Self::Route) -> Self::Props;
}

#[async_trait]
pub trait DynPageDyn: DynBasePage {
    async fn get_server_props(&self, route: Box<dyn Any + Send>) -> Box<dyn Any + Send>; // IndexRoute -> IndexPageProps
}

#[async_trait]
impl<T: DynPage> DynPageDyn for T {
    async fn get_server_props(&self, route: Box<dyn Any + Send>) -> Box<dyn Any + Send> {
        let route = route
            .downcast::<T::Route>()
            .expect("An error occured when downcasting a dyn Any route");
        let props = <T as DynPage>::get_server_props(*route).await;
        Box::new(props)
    }
}

#[cfg(test)]
mod test {
    use sycamore::render_to_string;

    use super::*;

    struct MyPage;

    struct MyRoute;

    impl Route for MyRoute {
        fn try_from_url(url: &UrlInfos) -> Option<Self> {
            let mut iter = url.segments().iter();

            match (iter.next(), iter.next()) {
                (Some(value), None) if value == &"index" => Some(MyRoute),
                _ => None,
            }
        }
    }

    impl BasePage for MyPage {
        type Route = MyRoute;

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
        assert!(dyn_page.try_match_route(&url_infos).is_none());
        let url_infos = UrlInfos::parse_from_url("/index/other");
        assert!(dyn_page.try_match_route(&url_infos).is_none());
        let url_infos = UrlInfos::parse_from_url("/index");
        assert!(dyn_page.try_match_route(&url_infos).is_some());

        let dyn_ssr_view = render_to_string(|cx| dyn_page.render_server(cx, Box::new(())));
        let ssr_view = render_to_string(|cx| MyPage::render(cx, ()));

        assert_eq!(dyn_ssr_view, ssr_view);
        assert!(ssr_view.contains("Greetings!"));
    }
}
