use super::prelude::*;
use std::any::Any;

#[derive(Default)]
pub struct Pages {
    dyn_pages: Vec<Box<dyn DynPageDyn>>,
}

impl Pages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_dyn_page_and_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, Box<dyn Any + Send>)> {
        for page in &self.dyn_pages {
            if let Some(route) = page.try_match_route(url_infos) {
                return Some((&**page, route));
            }
        }
        None
    }

    pub async fn find_dyn_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, Box<dyn Any>)> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props = page.get_server_props(route).await;
        Some((page, props))
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<String> {
        let (page, props) = self.find_dyn_page_and_props(url_infos).await?;
        let html = sycamore::render_to_string(|cx| page.render_server(cx, props));
        Some(html)
    }

    pub fn dyn_route<T: DynPage>(mut self, page: T) -> Self {
        self.dyn_pages.push(Box::new(page));
        self
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use sycamore::prelude::*;

    use super::*;

    struct MyPage;

    struct MyRoute(String);

    impl Route for MyRoute {
        fn try_from_url(url: &UrlInfos) -> Option<Self> {
            let mut iter = url.segments().iter();

            match (iter.next(), iter.next(), iter.next()) {
                (Some(value), Some(greeting), None) if value == &"index" => {
                    Some(MyRoute(greeting.to_string()))
                }
                _ => None,
            }
        }
    }

    impl BasePage for MyPage {
        type Route = MyRoute;

        type Props = String;

        fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G> {
            view! { cx,
                p {
                    (props)
                }
            }
        }
    }

    #[async_trait]
    impl DynPage for MyPage {
        async fn get_server_props(route: Self::Route) -> Self::Props {
            route.0
        }
    }

    #[tokio::test]
    async fn test() {
        let greeting = "test_greeting";
        let url = format!("index/{}", greeting);

        let pages = Pages::new().dyn_route(MyPage);

        let url_infos = UrlInfos::parse_from_url(&url);

        let rendered_html = pages.render_to_string(&url_infos).await.unwrap();
        assert!(rendered_html.contains(greeting));
    }
}
