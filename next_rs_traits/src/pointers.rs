use crate::pages::NotFoundPageProps;

use super::pages::{Component, Page};

use std::{any::Any, mem, ops::Deref};

// Those pointer wrappers garanties that they have exclusive acces to the underlying pointer,
// because they can only be created by either consuming one another
// or by consumming the value, boxing it and then leaking the box, taking exclusive ownership of the pointer.
// They are finally consummed when converting to the inner type.
// They are made for moving data in untyped manner for props and routes, the routes one are also made to retain lifetime informations.
// Those pointers act like a box, if they are drop without being consummed the data is still being dropped
// if you want access to the contained pointer use Ptr::leak(), this will return the pointer and mem::forget the pointer.

pub trait LifetimedAny<'a>: 'a {}

impl<'a, T: 'a> LifetimedAny<'a> for T {}

pub struct RouteCastedPtr<'a, T: Page>(*mut T::Route<'a>);
pub struct RouteUntypedPtr<'a>(*mut dyn LifetimedAny<'a>);

impl<'a, T: Page> From<RouteUntypedPtr<'a>> for RouteCastedPtr<'a, T> {
    fn from(route_ptr: RouteUntypedPtr) -> Self {
        let ptr = route_ptr.leak();
        RouteCastedPtr(ptr as *mut _)
    }
}

impl<'a, T: Page> RouteCastedPtr<'a, T> {
    pub fn leak(self) -> *mut T::Route<'a> {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }

    pub unsafe fn into_inner(self) -> T::Route<'a> {
        let route_ptr = self.leak();
        let route = Box::from_raw(route_ptr);
        *route
    }
}

unsafe impl<'a, T: Page> Send for RouteCastedPtr<'a, T> {}

impl<'a, T: Page> Drop for RouteCastedPtr<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let value: Box<T::Route<'a>> = Box::from_raw(self.0);
            drop(value);
        }
    }
}

impl<'a> RouteUntypedPtr<'a> {
    pub fn new<T: Page>(route: T::Route<'a>) -> Self {
        let boxed_route = Box::new(route);
        let ptr = Box::into_raw(boxed_route) as *mut _;
        RouteUntypedPtr(ptr)
    }

    pub unsafe fn cast<T: Page>(self) -> RouteCastedPtr<'a, T> {
        self.into()
    }

    pub fn leak(self) -> *mut dyn LifetimedAny<'a> {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }
}

impl<'a> Drop for RouteUntypedPtr<'a> {
    fn drop(&mut self) {
        unsafe {
            let value: Box<dyn LifetimedAny> = Box::from_raw(self.0);
            drop(value);
        }
    }
}

unsafe impl<'a> Send for RouteUntypedPtr<'a> {}

// Props ptr wrapper:

pub struct PropsCastedPtr<T: Component>(*mut T::Props);
pub struct PropsUntypedPtr(*mut dyn Any);

pub struct PropsSharedCastedPtr<'a, T: Component>(&'a T::Props);

impl<T: Component> From<PropsUntypedPtr> for PropsCastedPtr<T> {
    fn from(props_ptr: PropsUntypedPtr) -> Self {
        let ptr = props_ptr.leak();
        PropsCastedPtr(ptr as *mut _)
    }
}

impl<T: Component> PropsCastedPtr<T> {
    pub unsafe fn into_inner(self) -> T::Props {
        let ptr = self.leak();
        let props = Box::from_raw(ptr);
        *props
    }

    pub fn leak(self) -> *mut T::Props {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }
}



unsafe impl<T: Component> Send for PropsCastedPtr<T> {}

impl<T: Component> Drop for PropsCastedPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let value: Box<T::Props> = Box::from_raw(self.0);
            drop(value);
        }
    }
}

impl PropsUntypedPtr {
    pub fn new<T: Component>(props: T::Props) -> Self {
        let boxed_props = Box::new(props);
        let ptr = Box::into_raw(boxed_props) as *mut _;
        PropsUntypedPtr(ptr)
    }

    pub fn new_not_found_props(props: NotFoundPageProps) -> Self {
        let boxed_unit = Box::new(props);
        let ptr = Box::into_raw(boxed_unit) as *mut _;
        PropsUntypedPtr(ptr)
    }

    pub unsafe fn cast<T: Component>(self) -> PropsCastedPtr<T> {
        self.into()
    }

    pub fn leak(self) -> *mut dyn Any {
        let ptr = self.0;
        mem::forget(self);
        ptr
    }

    pub unsafe fn to_shared<T: Component>(&self) -> PropsSharedCastedPtr<T> {
        let ptr = self.0 as *mut T::Props;
        let reference = &*ptr;
        PropsSharedCastedPtr(reference)
    }
}

impl Drop for PropsUntypedPtr {
    fn drop(&mut self) {
        unsafe {
            let value: Box<dyn Any> = Box::from_raw(self.0);
            drop(value);
        }
    }
}

// same as RouteUntypePtr, can only be created if T is send
unsafe impl Send for PropsUntypedPtr {}

impl<'a, T: Component> Deref for PropsSharedCastedPtr<'a, T> {
    type Target = T::Props;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
