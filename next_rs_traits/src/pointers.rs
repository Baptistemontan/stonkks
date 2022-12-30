use super::pages::{Component, NotFoundPageProps};
use super::routes::Routable;

use std::any::Any;

/// Send requirement is for use in async functions.
pub trait LifetimedAny<'a>: Send + 'a {}

impl<'a, T: Send + 'a> LifetimedAny<'a> for T {}

pub struct RouteUntypedPtr<'a>(Box<dyn LifetimedAny<'a>>);

impl<'a> RouteUntypedPtr<'a> {
    pub fn new<T: Routable>(route: T::Route<'a>) -> Self {
        let boxed_route = Box::new(route);
        RouteUntypedPtr(boxed_route)
    }

    pub unsafe fn cast<T: Routable>(self) -> Box<T::Route<'a>> {
        let ptr = self.into_raw() as *mut T::Route<'a>;
        Box::from_raw(ptr)
    }

    pub fn into_raw(self) -> *mut dyn LifetimedAny<'a> {
        Box::into_raw(self.0)
    }
}
pub struct PropsUntypedPtr(Box<dyn Any>);

impl PropsUntypedPtr {
    pub fn new<T: Component>(props: T::Props) -> Self {
        let boxed_props = Box::new(props);
        PropsUntypedPtr(boxed_props)
    }

    pub fn new_unit() -> Self {
        let boxed_props = Box::new(());
        PropsUntypedPtr(boxed_props)
    }

    pub fn new_not_found_props(props: NotFoundPageProps) -> Self {
        let boxed_props = Box::new(props);
        Self(boxed_props)
    }

    pub unsafe fn cast<T: Component>(self) -> Box<T::Props> {
        let ptr = self.into_raw() as *mut T::Props;
        Box::from_raw(ptr)
    }

    pub fn into_raw(self) -> *mut dyn Any {
        Box::into_raw(self.0)
    }

    pub unsafe fn shared_cast<T: Component>(&self) -> &T::Props {
        let ptr = self.0.as_ref() as *const _ as *const T::Props;
        unsafe { &*ptr }
    }
}
