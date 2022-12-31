use next_rs::prelude::*;
use sycamore::prelude::*;

pub struct Index;

impl Component for Index {
    type Props = ();

    fn render<'a, G: Html>(cx: Scope<'a>, _props: ComponentReactiveProps<'a, Self>) -> View<G> {
        view! { cx,
            h1 {
                "Index Page."
            }
            a(href="/counter/45") {
                "counter"
            }
        }
    }

    fn render_head<'a, G: Html>(
        cx: Scope<'a>,
        _props: &ComponentReactiveProps<'a, Self>,
    ) -> View<G> {
        view! { cx,
            title {
                "index"
            }
        }
    }
}

pub struct IndexRoute;

impl<'a> Route<'a> for IndexRoute {
    fn try_from_url(url: UrlInfos<'_, 'a>) -> Option<Self> {
        url.segments().is_empty().then_some(IndexRoute)
    }
}

impl Routable for Index {
    type Route<'a> = IndexRoute;
}
