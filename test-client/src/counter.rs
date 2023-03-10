use serde::{Deserialize, Serialize};
use stonkks::prelude::*;
use sycamore::prelude::*;

pub struct Counter;

#[derive(Serialize, Deserialize)]
pub struct CounterProps {
    count: i32,
}

pub struct CounterReactiveProps<'a> {
    count: &'a Signal<i32>,
}

impl Props for CounterProps {}

impl<'a> ReactiveProps<'a> for CounterReactiveProps<'a> {
    type Props = CounterProps;
}

impl IntoProps for CounterProps {
    type ReactiveProps<'a> = CounterReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a> {
        let count = create_signal(cx, self.count);
        CounterReactiveProps { count }
    }
}

impl Component for Counter {
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

    fn render_head<'a, G: Html>(cx: Scope<'a>, props: &Self::Props) -> View<G> {
        let count = props.count;
        view! { cx,
            title {
                "counter: " (count)
            }
        }
    }
}

#[derive(Hash)]
pub struct CounterRoute(i32);

impl<'url> Route<'url> for CounterRoute {
    fn try_from_url(url: UrlInfos<'_, 'url>) -> Option<Self> {
        match url.segments() {
            ["counter"] => Some(CounterRoute(0)),
            ["counter", count] => Some(CounterRoute(count.parse().ok()?)),
            _ => None,
        }
    }
}

impl Routable for Counter {
    type Route<'a> = CounterRoute;
}

#[async_trait::async_trait]
impl DynPage for Counter {
    type Err<'url> = ();
    type State<'r> = ();
    async fn get_server_props<'url, 'r>(
        route: Self::Route<'url>,
        _states: (),
    ) -> Result<Self::Props, ()> {
        Ok(CounterProps { count: route.0 })
    }
}
