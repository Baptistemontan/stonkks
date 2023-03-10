use serde::{de::DeserializeOwned, Serialize};
use sycamore::prelude::*;

/// Need Send for use in async functions,
/// need DeserializeOwned cause it will be converted to ReactiveProps anyway,
/// which can't have value borrowed from the serialized string or the url.
/// And need 'static for the same reason, there is nothing to borrow from so It makes
/// my life easier.
pub trait Props: Send + IntoProps + Serialize + DeserializeOwned + 'static {}

pub trait ReactiveProps<'a> {
    type Props: Props;
}

pub trait IntoProps {
    type ReactiveProps<'a>: ReactiveProps<'a>;

    fn into_reactive_props<'a>(self, cx: Scope<'a>) -> Self::ReactiveProps<'a>;
}

impl Props for () {}

impl IntoProps for () {
    type ReactiveProps<'a> = ();

    fn into_reactive_props<'a>(self, _cx: Scope<'a>) -> Self::ReactiveProps<'a> {
        ()
    }
}

impl<'a> ReactiveProps<'a> for () {
    type Props = ();
}
