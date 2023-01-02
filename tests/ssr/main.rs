use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stonkks::prelude::*;
use stonkks_traits::pages::DynBasePage;
use stonkks_traits::pointers::*;
use stonkks_traits::states::StatesMap;
use sycamore::prelude::*;

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
    fn try_from_url(url: UrlInfos<'_, 'a>) -> Option<Self> {
        let mut iter = url.segments().iter().cloned();

        match (iter.next(), iter.next(), iter.next()) {
            (Some(value), Some(greeting), None) if value == "index" => Some(MyRoute(greeting)),
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

impl Routable for MyDynPage {
    type Route<'a> = MyRoute<'a>;
}

#[async_trait]
impl DynPage for MyDynPage {
    type Err<'url> = ();
    type State<'r> = ();
    async fn get_server_props<'url, 'r>(
        route: Self::Route<'url>,
        _state: (),
    ) -> Result<Self::Props, ()> {
        Ok(MyProps(route.0.to_string()))
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

    let url_infos = OwnedUrlInfos::parse_from_url(&url);

    let server = app.into_server();

    let states = StatesMap::default();

    let rendered_html = server
        .try_render_to_string(url_infos.to_shared(), &states)
        .await
        .unwrap()
        .unwrap();

    assert!(rendered_html.contains(greeting));
}

#[test]
fn test_routing() {
    let page = MyDynPage;
    let dyn_page: Box<dyn DynBasePage> = Box::new(page);
    let url_infos = OwnedUrlInfos::parse_from_url("/about");
    assert!(dyn_page.try_match_route(url_infos.to_shared()).is_none());
    let url_infos = OwnedUrlInfos::parse_from_url("/index/other");
    assert!(dyn_page.try_match_route(url_infos.to_shared()).is_some());
    let url_infos = OwnedUrlInfos::parse_from_url("/index");
    assert!(dyn_page.try_match_route(url_infos.to_shared()).is_none());

    let props: &str = "Greetings!";

    let dyn_ssr_view = sycamore::render_to_string(|cx| unsafe {
        let result =
            dyn_page.render_server(cx, PropsUntypedPtr::new::<MyDynPage>(MyProps(props.into())));
        result.body
    });
    let ssr_view = sycamore::render_to_string(|cx| {
        MyDynPage::render(cx, MyProps(props.into()).into_reactive_props(cx))
    });

    assert_eq!(dyn_ssr_view, ssr_view);
    assert!(ssr_view.contains(props));
}

#[tokio::test]
async fn test_layout() {
    let greeting = "test_greeting";
    let url = format!("index/{}", greeting);

    let app = App::new().dyn_page(MyDynPage).with_layout(MyLayout);

    let server = app.into_server();

    let url_infos = OwnedUrlInfos::parse_from_url(&url);

    let states = StatesMap::default();

    let rendered_html = server
        .try_render_to_string(url_infos.to_shared(), &states)
        .await
        .unwrap()
        .unwrap();

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

    let url_infos = OwnedUrlInfos::parse_from_url("absolutely_not_index");

    let states = StatesMap::default();

    assert!(server
        .try_render_to_string(url_infos.to_shared(), &states)
        .await
        .is_none());

    let rendered_html = server.render_not_found().unwrap();

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

    let url_infos = OwnedUrlInfos::parse_from_url("absolutely_not_index");

    let states = StatesMap::default();

    assert!(server
        .try_render_to_string(url_infos.to_shared(), &states)
        .await
        .is_none());

    let rendered_html = server.render_not_found().unwrap();

    println!("{}", rendered_html);

    assert!(rendered_html.contains("Custom not found"));
}

#[tokio::test]
async fn test_dyn_page_total_render() {
    let greeting = "test_greeting";
    let url = format!("index/{}", greeting);

    let app = App::new().dyn_page(MyDynPage);

    let url_infos = OwnedUrlInfos::parse_from_url(&url);

    let server = app.into_server();

    let states = StatesMap::default();

    let rendered_html = server
        .try_render_to_string(url_infos.to_shared(), &states)
        .await
        .unwrap()
        .unwrap();

    println!("{}", rendered_html);

    assert!(rendered_html.contains(greeting));
}
