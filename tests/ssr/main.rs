use async_trait::async_trait;
use next_rs::prelude::*;
use next_rs_traits::pages::DynBasePage;
use next_rs_traits::pointers::*;
use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, render_to_string};

struct MyLayout;

impl Layout for MyLayout {
    fn render<'a, G: Html>(cx: Scope<'a>, props: View<G>) -> View<G> {
        view! { cx,
            h1 {
                "This is a Title"
            }
            div {
                (props)
            }
            p {
                "test paragraphe"
            }
        }
    }
}

struct MyDynPage;

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

#[derive(Serialize, Deserialize)]
struct MyProps(String);

struct MyReactiveProps<'a>(&'a Signal<String>);

impl Props for MyProps {}

impl IntoProps for MyProps {
    type ReactiveProps<'a> = MyReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a> {
        let signal = create_signal(cx, self.0);
        MyReactiveProps(signal)
    }
}

impl<'a> ReactiveProps<'a> for MyReactiveProps<'a> {
    type Props = MyProps;
}

impl Component for MyDynPage {
    type Props = MyProps;

    fn render<'a, G: Html>(cx: Scope<'a>, props: ComponentReactiveProps<'a, Self>) -> View<G> {
        view! { cx,
            p {
                (props.0.get())
            }
        }
    }
}

impl Page for MyDynPage {
    type Route<'a> = MyRoute<'a>;
}

#[async_trait]
impl DynPage for MyDynPage {
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props {
        MyProps(route.0.to_string())
    }
}

struct MyNotFound;

impl Component for MyNotFound {
    type Props = NotFoundPageProps;

    fn render<'a, G: Html>(cx: Scope<'a>, _props: ComponentReactiveProps<'a, Self>) -> View<G> {
        view! { cx,
            p {
                "Custom not found page"
            }
        }
    }
}

#[tokio::test]
async fn test_dyn_page() {
    let greeting = "test_greeting";
    let url = format!("index/{}", greeting);

    let app = App::new().dyn_page(MyDynPage);

    let url_infos = UrlInfos::parse_from_url(&url);

    let server = app.into_server();

    let (rendered_html, _props) = server.render_to_string(&url_infos).await;

    assert!(rendered_html.contains(greeting));
}

#[test]
fn test_routing() {
    let page = MyDynPage;
    let dyn_page: Box<dyn DynBasePage> = Box::new(page);
    let url_infos = UrlInfos::parse_from_url("/about");
    assert!(dyn_page.try_match_route(&url_infos).is_none());
    let url_infos = UrlInfos::parse_from_url("/index/other");
    assert!(dyn_page.try_match_route(&url_infos).is_some());
    let url_infos = UrlInfos::parse_from_url("/index");
    assert!(dyn_page.try_match_route(&url_infos).is_none());

    let props: &str = "Greetings!";

    let dyn_ssr_view = render_to_string(|cx| unsafe {
        dyn_page.render_server(cx, PropsUntypedPtr::new::<MyDynPage>(MyProps(props.into())))
    });
    let ssr_view =
        render_to_string(|cx| MyDynPage::render(cx, MyProps(props.into()).into_reactive_props(cx)));

    assert_eq!(dyn_ssr_view, ssr_view);
    assert!(ssr_view.contains(props));
}

#[tokio::test]
async fn test_layout() {
    let greeting = "test_greeting";
    let url = format!("index/{}", greeting);

    let app = App::new().dyn_page(MyDynPage).with_layout(MyLayout);

    let server = app.into_server();

    let url_infos = UrlInfos::parse_from_url(&url);

    let (rendered_html, _props) = server.render_to_string(&url_infos).await;

    println!("{}", rendered_html);

    assert!(rendered_html.contains(greeting));
    assert!(rendered_html.contains("</h1>")); // check for closing tag, opening tag could have some data on it
    assert!(rendered_html.contains("Title"));
    assert!(rendered_html.contains("test paragraphe"));
}

#[tokio::test]
async fn test_default_not_found() {
    let app = App::new().dyn_page(MyDynPage).with_layout(MyLayout);
    let server = app.into_server();

    let url_infos = UrlInfos::parse_from_url("absolutely_not_index");

    let (rendered_html, _props) = server.render_to_string(&url_infos).await;

    println!("{}", rendered_html);

    assert!(rendered_html.contains("Page not found."));
}

#[tokio::test]
async fn test_custom_not_found() {
    let app = App::new()
        .dyn_page(MyDynPage)
        .with_layout(MyLayout)
        .not_found(MyNotFound);

    let server = app.into_server();

    let url_infos = UrlInfos::parse_from_url("absolutely_not_index");

    let (rendered_html, _props) = server.render_to_string(&url_infos).await;

    println!("{}", rendered_html);

    assert!(rendered_html.contains("Custom not found"));
}
