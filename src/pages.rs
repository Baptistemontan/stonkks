use std::ops::Deref;

use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::DynPageDyn;
use next_rs_traits::pointers::*;

use sycamore::prelude::*;

struct DefaultLayout;

impl Layout for DefaultLayout {
    fn render<'a, G: Html>(_: Scope<'a>, props: View<G>) -> View<G> {
        props
    }
}

struct AppLayout(Box<dyn DynLayout>);

impl Default for AppLayout {
    fn default() -> Self {
        let layout = Box::new(DefaultLayout);
        Self(layout)
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

#[derive(Default)]
pub struct Pages {
    dyn_pages: Vec<Box<dyn DynPageDyn>>,
    layout: AppLayout,
}

impl Pages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_dyn_page_and_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, RouteUntypedPtr)> {
        for page in &self.dyn_pages {
            unsafe {
                if let Some(route) = page.try_match_route(url_infos) {
                    return Some((&**page, route));
                }
            }
        }
        None
    }

    pub async fn find_dyn_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, PropsUntypedPtr)> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props = unsafe { page.get_server_props(route).await };
        Some((page, props))
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<String> {
        let (page, props) = self.find_dyn_page_and_props(url_infos).await?;
        let html = sycamore::render_to_string(|cx| {
            let props = unsafe { page.render_server(cx, props) };
            self.layout.render_server(cx, props)
        });
        Some(html)
    }

    pub fn dyn_page<T: DynPage + 'static>(mut self, page: T) -> Self
    where
        T::Props: Send,
    {
        self.dyn_pages.push(Box::new(page));
        self
    }

    pub fn with_layout<T: Layout>(mut self, layout: T) -> Self {
        self.layout = layout.into();
        self
    }
}
