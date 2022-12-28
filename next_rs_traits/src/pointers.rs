use crate::pages::NotFoundPageProps;

use super::pages::{Component, Page};

// Those pointer wrappers garanties that they have exclusive acces to the underlying pointer,
// because they can only be created by either consuming one another
// or by consumming the value, boxing it and then leaking the box, taking exclusive ownership of the pointer.
// They are finally consummed when converting to the inner type.
// They are made for moving data in a untyped, unlifetimed(?) maner for props and routes.

// TODO: implement drop for them and deallocating the box in case it is dropped without being consummed
// (early return, panic, future cancelling, ect...)
// and use mem::forget when consuming.
// ! After further research, you need recreate the box to the correct type for it to deallocate.
// So Drop can be implemented for CastedPtr, but not for UntypedPtr.
// At least not in the current implementation, need further research but maybe UntypedPtr can
// hold the Layout of the type and use that to free it.
// ! Forget that, we need the concrete type for dropping the inner type.
// Other possibility would be to have dyn Any pointer, and recreating a Box<dyn Any> for deallocating
// The Vtable would have the drop function.

// Route ptr wrapper:

pub struct RouteCastedPtr<'a, T: Page>(*mut T::Route<'a>);
pub struct RouteUntypedPtr(*mut ());

impl<'a, T: Page> From<RouteUntypedPtr> for RouteCastedPtr<'a, T> {
    fn from(RouteUntypedPtr(route_ptr): RouteUntypedPtr) -> Self {
        RouteCastedPtr(route_ptr as *mut _)
    }
}

impl<'a, T: Page> RouteCastedPtr<'a, T> {
    pub unsafe fn into_inner(self) -> T::Route<'a> {
        let route = Box::from_raw(self.0);
        *route
    }
}

unsafe impl<'a, T: Page> Send for RouteCastedPtr<'a, T> where T::Route<'a>: Send {}

impl RouteUntypedPtr {
    pub fn new<'a, T: Page>(route: T::Route<'a>) -> Self
    where
        T::Route<'a>: Send,
    {
        let boxed_route = Box::new(route);
        let ptr = Box::leak(boxed_route) as *mut _ as *mut ();
        RouteUntypedPtr(ptr)
    }

    pub unsafe fn cast<'a, T: Page>(self) -> RouteCastedPtr<'a, T> {
        self.into()
    }
}

// RouteUntypePtr can only be constructed if the concrete type implement Send
unsafe impl Send for RouteUntypedPtr {}

// Props ptr wrapper:

pub struct PropsCastedPtr<T: Component>(pub *mut T::Props);
pub struct PropsUntypedPtr(pub *mut ());

impl<T: Component> From<PropsUntypedPtr> for PropsCastedPtr<T> {
    fn from(PropsUntypedPtr(props_ptr): PropsUntypedPtr) -> Self {
        PropsCastedPtr(props_ptr as *mut _)
    }
}

impl<T: Component> PropsCastedPtr<T> {
    pub unsafe fn into_inner(self) -> T::Props {
        let props = Box::from_raw(self.0);
        *props
    }
}

unsafe impl<T: Component> Send for PropsCastedPtr<T> where T::Props: Send {}

impl PropsUntypedPtr {
    pub fn new<T: Page>(props: T::Props) -> Self
    where
        T::Props: Send,
    {
        let boxed_props = Box::new(props);
        let ptr = Box::leak(boxed_props) as *mut _ as *mut ();
        PropsUntypedPtr(ptr)
    }

    pub fn new_not_found_props(props: NotFoundPageProps) -> Self {
        let boxed_unit = Box::new(props);
        let ptr = Box::leak(boxed_unit) as *mut _ as *mut ();
        PropsUntypedPtr(ptr)
    }

    pub unsafe fn cast<T: Page>(self) -> PropsCastedPtr<T> {
        self.into()
    }
}

// same as RouteUntypePtr, can only be created if T is send
unsafe impl Send for PropsUntypedPtr {}
