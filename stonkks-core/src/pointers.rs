use super::pages::{Component, NotFoundPageProps};
use super::routes::Routable;

use std::any::Any;

/// Send requirement is for use in async functions.
trait LifetimedAny<'a>: Send + 'a {}

/// Auto impl of LifetimedAny
impl<'a, T: Send + 'a> LifetimedAny<'a> for T {}

/// Struct holding data in an Untyped way but still tracking it's lifetime.
/// Can be seen as a `Box<dyn Any>` but with a lifetime annotation.
/// The downside of this method is that we lose the safe `downcast` method provided by
/// `Box<dyn Any>`, so the only downcast method exposed is unsafe, because it trust the caller
/// that the given type is correct.
struct UntypedPtr<'a>(Box<dyn LifetimedAny<'a>>);

impl<'a> UntypedPtr<'a> {
    /// Create a new `UntypedPtr` from a value implementing `LifetimedAny`
    pub fn new<T: LifetimedAny<'a>>(value: T) -> Self {
        let boxed_value = Box::new(value);
        UntypedPtr(boxed_value)
    }

    /// Downcast the inner value into a concrete type.
    /// This method is marked as unsafe because it can't check if T is correct.
    /// The caller must make sure T is the correct concrete type of the backed data.
    pub unsafe fn downcast<T: LifetimedAny<'a>>(self) -> Box<T> {
        let ptr = self.into_raw() as *mut T;
        Box::from_raw(ptr)
    }

    /// Consumme the inner box and leak it as a raw pointer.
    pub fn into_raw(self) -> *mut dyn LifetimedAny<'a> {
        Box::into_raw(self.0)
    }

    pub unsafe fn downcast_ref<T: LifetimedAny<'a>>(&self) -> &T {
        let ptr = self.0.as_ref() as *const _ as *const T;
        unsafe { &*ptr }
    }
}

/// Wrapper for moving the `Route` of a `Routable` type in an untyped way but sill allowing it
/// to borrow for the Url.
pub struct RouteUntypedPtr<'a>(UntypedPtr<'a>);

impl<'a> RouteUntypedPtr<'a> {
    /// Create a new `RouteUntypedPtr` from the `Route` of a `Routable` type.
    pub fn new<T: Routable>(route: T::Route<'a>) -> Self {
        Self(UntypedPtr::new(route))
    }

    /// Downcast the backed data into the concrete `Route` of a `Routable` type.
    /// Marked as unsafe because the function does not check if the given type match
    /// the type of the backed data.
    /// The caller must make sure the type is correct.
    pub unsafe fn downcast<T: Routable>(self) -> Box<T::Route<'a>> {
        self.0.downcast()
    }

    pub unsafe fn downcast_ref<T: Routable>(&self) -> &T::Route<'a> {
        self.0.downcast_ref()
    }
}

///
pub struct PropsUntypedPtr(Box<dyn Any + Send>);

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

    pub unsafe fn downcast<T: Component>(self) -> Box<T::Props> {
        // The best way would be to use `Box::downcast_unchecked` but unstable at the moment.
        let ptr = self.into_raw() as *mut T::Props;
        Box::from_raw(ptr)
    }

    fn into_raw(self) -> *mut dyn Any {
        Box::into_raw(self.0)
    }

    pub unsafe fn downcast_ref<T: Component>(&self) -> &T::Props {
        // `Box::downcast_ref_unchecked` would be more suited but unstable at the moment.
        let ptr = self.0.as_ref() as *const _ as *const T::Props;
        unsafe { &*ptr }
    }
}
