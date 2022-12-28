use sycamore::prelude::*;


pub trait Props: Send + 'static + IntoProps { }


pub trait ReactiveProps<'a> {
    type Props: Props;
}

pub trait IntoProps {
    type ReactiveProps<'a>: ReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a>;
}