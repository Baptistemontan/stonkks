use sycamore::prelude::*;

pub trait Layout: 'static {
    fn render<'a, G: Html>(cx: Scope<'a>, props: View<G>) -> View<G>;
}

pub trait DynLayout {
    fn render_client(&self, cx: Scope, props: View<DomNode>) -> View<DomNode>;
    fn render_server(&self, cx: Scope, props: View<SsrNode>) -> View<SsrNode>;
    fn hydrate(&self, cx: Scope, props: View<HydrateNode>) -> View<HydrateNode>;
}

impl<T: Layout> DynLayout for T {
    fn render_client(&self, cx: Scope, props: View<DomNode>) -> View<DomNode> {
        T::render(cx, props)
    }

    fn render_server(&self, cx: Scope, props: View<SsrNode>) -> View<SsrNode> {
        T::render(cx, props)
    }

    fn hydrate(&self, cx: Scope, props: View<HydrateNode>) -> View<HydrateNode> {
        T::render(cx, props)
    }
}
