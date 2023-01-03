use stonkks::prelude::*;
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

#[derive(Hash)]
pub struct IndexRoute;

impl<'a> Route<'a> for IndexRoute {
    fn try_from_url(url: UrlInfos<'_, 'a>) -> Option<Self> {
        url.segments().is_empty().then_some(IndexRoute)
    }
}

impl Routable for Index {
    type Route<'a> = IndexRoute;
}

#[async_trait::async_trait]
impl StaticPage for Index {
    type RouteError = ();
    type PropsError<'a> = ();
    type RouteState<'r> = ();
    type PropsState<'r> = ();

    async fn get_props<'url, 'r>(
        _route: Self::Route<'url>,
        _states: Self::PropsState<'r>,
    ) -> Result<Self::Props, Self::PropsError<'url>> {
        Ok(())
    }
    async fn get_build_routes<'r>(
        _states: Self::RouteState<'r>,
    ) -> Result<Vec<String>, Self::RouteError> {
        Ok(vec!["".into()])
    }
}
