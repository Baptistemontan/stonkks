use std::any::Any;
use super::prelude::*;

pub struct Pages {
    dyn_pages: Vec<Box<dyn DynPageDyn>>,
}

impl Pages {
    pub fn find_dyn_page_and_route<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<(&'_ dyn DynPageDyn, Box<dyn Any + Send>)> {    
        for page in &self.dyn_pages {
            if let Some(route) = page.try_match_route(url_infos) {
                return Some((&**page, route));
            }
        }
        None
    }
    
    pub async fn find_dyn_page_and_props<'url>(&self, url_infos: &UrlInfos<'url>,) -> Option<(&'_ dyn DynPageDyn, Box<dyn Any>)> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props = page.get_server_props(route).await;
        Some((page, props))
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
                (Some(value), Some(greeting), None) if value == &"index" => Some(MyRoute(greeting.to_string())),
                _ => None
            }
        }
    }

    impl BasePage for MyPage {
        type Route = MyRoute;

        type Props = String;

        fn render<G: Html>(cx: Scope, props: Self::Props) -> View<G> {
            view!{ cx,
                p {
                    (props)
                }
            }
        }
    }

    #[async_trait]
    impl DynPage for MyPage {
        async fn get_server_props(route: Self::Route) -> Self::Props  {
            route.0
        }
    }

    #[tokio::test]
    async fn test() {

        let greeting = "test_greeting";

        let url = format!("index/{}", greeting);

        let pages = Pages {
            dyn_pages: vec![Box::new(MyPage)]
        };

        let url_infos = UrlInfos::parse_from_url(&url);

        let (page, props) = pages.find_dyn_page_and_props(&url_infos).await.unwrap();

        let rendered_html = sycamore::render_to_string(|cx| page.render_server(cx, props));

        assert!(rendered_html.contains(greeting));
    }
}



