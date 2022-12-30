use next_rs::prelude::*;
use sycamore::prelude::*;
use serde::{Serialize, Deserialize};


struct CounterPage;

#[derive(Serialize, Deserialize)]
struct CounterProps {
    count: i32
}

struct CounterReactiveProps<'a> {
    count: &'a Signal<i32>
}

impl Props for CounterProps {}

impl<'a> ReactiveProps<'a> for CounterReactiveProps<'a> {
    type Props = CounterProps;
}

impl IntoProps for CounterProps {
    type ReactiveProps<'a> = CounterReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a> {
        let count = create_signal(cx, self.count);
        CounterReactiveProps {
            count
        }
    }
}

impl Component for CounterPage {
    type Props = CounterProps;

    fn render<'a, G: Html>(cx: Scope<'a>, props: ComponentReactiveProps<'a, Self>) -> View<G> {
        let state = props.count;
        let increment = |_| state.set(*state.get() + 1);
        let decrement = |_| state.set(*state.get() - 1);
        let reset = |_| state.set(0);
        view! { cx,
            div {
                p { "Value: " (state.get()) }
                button(on:click=increment) { "+" }
                button(on:click=decrement) { "-" }
                button(on:click=reset) { "Reset" }
            }
        }
    }

    fn render_head<'a, G:Html>(cx: Scope<'a>, props: &ComponentReactiveProps<'a, Self>) -> View<G> {
        let count = props.count;
        view! { cx, 
            title {
                "counter: " (*count.get())
            }
        }
    }
}

struct CounterRoute(i32);

impl<'a> Route<'a> for CounterRoute {
    fn try_from_url(url: &UrlInfos<'a>) -> Option<Self> {
        let mut segments = url.segments().into_iter().copied();
        if segments.next()? != "counter" {
            return None;
        }
        match (segments.next(), segments.next()) {
            (Some(second), None) => Some(CounterRoute(second.parse().ok()?)),
            (None, None) => Some(CounterRoute(0)),
            _ => None
        }
    }
}

impl Page for CounterPage {
    type Route<'a> = CounterRoute;
}

#[async_trait::async_trait]
impl DynPage for CounterPage {
    async fn get_server_props<'url>(route: Self::Route<'url>) -> Self::Props {
        CounterProps { count: route.0 }
    }
}

pub fn get_app() -> App {
    App::new().dyn_page(CounterPage)
}