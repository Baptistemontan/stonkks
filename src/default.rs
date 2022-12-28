use std::ops::Deref;

use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::DynComponent;
use sycamore::prelude::*;

struct DefaultLayout;

impl Layout for DefaultLayout {
    fn render<'a, G: Html>(_: Scope<'a>, props: View<G>) -> View<G> {
        props
    }
}

pub struct AppLayout(Box<dyn DynLayout>);

impl Default for AppLayout {
    fn default() -> Self {
        Self::new(DefaultLayout)
    }
}

impl AppLayout {
    fn new<T: Layout>(layout: T) -> Self {
        let boxed_layout = Box::new(layout);
        Self(boxed_layout)
    }
}

impl<T: Layout> From<T> for AppLayout {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Deref for AppLayout {
    type Target = dyn DynLayout;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

struct DefaultNotFound;

impl Component for DefaultNotFound {
    type Props = NotFoundPageProps;

    fn render<'a, G: Html>(cx: Scope<'a>, _props: ComponentProps<'a, Self>) -> View<G> {
        view! { cx,
            h1 {
                "Page not found."
            }
        }
    }
}

pub struct NotFound(Box<dyn DynComponent>);

impl Default for NotFound {
    fn default() -> Self {
        Self::new(DefaultNotFound)
    }
}

impl NotFound {
    fn new<T: NotFoundPage>(not_found_page: T) -> Self {
        let boxed_not_found = Box::new(not_found_page);
        Self(boxed_not_found)
    }
}

impl<T: NotFoundPage> From<T> for NotFound {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Deref for NotFound {
    type Target = dyn DynComponent;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
