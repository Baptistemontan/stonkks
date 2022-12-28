use sycamore::prelude::*;
use serde::{Serialize, Deserialize};


pub trait Props: Send + 'static + IntoProps + Serialize + Deserialize<'static> { }


pub trait ReactiveProps<'a> {
    type Props: Props;
}

pub trait IntoProps {
    type ReactiveProps<'a>: ReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a>;
}