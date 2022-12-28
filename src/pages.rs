use super::prelude::*;
use next_rs_traits::pages::pages_ptr::*;
use next_rs_traits::pages::{DynBasePage, DynPageDyn, DynComponent};

// struct DefaultLayout;

// impl Component for DefaultLayout {
//     type Props = ;

//     fn render<G: sycamore::web::Html>(cx: sycamore::reactive::Scope, props: Self::Props) -> sycamore::view::View<G> {
//         todo!()
//     }
// }

#[derive(Default)]
pub struct Pages {
    dyn_pages: Vec<Box<dyn DynPageDyn>>,
    // layout: Box<dyn DynComponent>
}

impl Pages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_dyn_page_and_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, RouteUntypedPtr)> {
        for page in &self.dyn_pages {
            unsafe {
                if let Some(route) = page.try_match_route(url_infos) {
                    return Some((&**page, route));
                }
            }
        }
        None
    }

    pub async fn find_dyn_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, PropsUntypedPtr)> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props = unsafe { page.get_server_props(route).await };
        Some((page, props))
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<String> {
        let (page, props) = self.find_dyn_page_and_props(url_infos).await?;
        let html = sycamore::render_to_string(|cx| unsafe { page.render_server(cx, props) });
        Some(html)
    }

    pub fn dyn_page<T: DynPage + 'static>(mut self, page: T) -> Self 
        where T::Props: Send
    {
        self.dyn_pages.push(Box::new(page));
        self
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use next_rs_traits::pages::Component;
    use sycamore::prelude::*;

    use super::*;

    struct MyPage;

    struct MyRoute<'a>(&'a str);

    impl<'a> Route<'a> for MyRoute<'a> {
        fn try_from_url(url: &UrlInfos<'a>) -> Option<Self> {
            let mut iter = url.segments().iter();

            match (iter.next(), iter.next(), iter.next()) {
                (Some(value), Some(greeting), None) if value == &"index" => Some(MyRoute(greeting)),
                _ => None,
            }
        }
    }

    impl Component for MyPage {
        type Props = String;

        fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G> {
            view! { cx,
                p {
                    (props)
                }
            }
        }
    }

    impl Page for MyPage {
        type Route<'a> = MyRoute<'a>;
    }

    #[async_trait]
    impl DynPage for MyPage {
        async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props {
            route.0.to_string()
        }
    }

    #[tokio::test]
    async fn test() {
        let greeting = "test_greeting";
        let url = format!("index/{}", greeting);

        let pages = Pages::new().dyn_page(MyPage);

        let url_infos = UrlInfos::parse_from_url(&url);

        let rendered_html = pages.render_to_string(&url_infos).await.unwrap();
        assert!(rendered_html.contains(greeting));
    }
}
