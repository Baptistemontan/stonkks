use async_trait::async_trait;
use next_rs::prelude::*;
use sycamore::{prelude::*, render_to_string};
use next_rs_traits::pages::DynBasePage;
use next_rs_traits::pages::pages_ptr::*;

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
async fn test_1() {
    let greeting = "test_greeting";
    let url = format!("index/{}", greeting);

    let pages = Pages::new().dyn_page(MyPage);

    let url_infos = UrlInfos::parse_from_url(&url);

    let rendered_html = pages.render_to_string(&url_infos).await.unwrap();
    println!("{}", rendered_html);
    assert!(rendered_html.contains(greeting));
}

#[test]
fn test_routing() {
    let page = MyPage;
    let dyn_page: Box<dyn DynBasePage> = Box::new(page);
    let url_infos = UrlInfos::parse_from_url("/about");
    unsafe {
        assert!(dyn_page.try_match_route(&url_infos).is_none());
    }
    let url_infos = UrlInfos::parse_from_url("/index/other");
    unsafe {
        assert!(dyn_page.try_match_route(&url_infos).is_some());
    }
    let url_infos = UrlInfos::parse_from_url("/index");
    unsafe {
        assert!(dyn_page.try_match_route(&url_infos).is_none());
    }

    let props: &str = "Greetings!";

    let dyn_ssr_view = render_to_string(|cx| unsafe {
        dyn_page.render_server(cx, PropsUntypedPtr::new::<MyPage>(props.into()))
    });
    let ssr_view = render_to_string(|cx| MyPage::render(cx, props.into()));

    assert_eq!(dyn_ssr_view, ssr_view);
    assert!(ssr_view.contains(props));
}