use super::prelude::*;
use next_rs_traits::pages::{DynPageDyn, DynComponent};
use next_rs_traits::pointers::*;
use super::default::{AppLayout, NotFound};

#[derive(Default)]
pub struct Pages {
    dyn_pages: Vec<Box<dyn DynPageDyn>>,
    layout: AppLayout,
    not_found_page: NotFound,
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

    pub async fn find_page_and_props<'url>(&self, url_infos: &UrlInfos<'url>) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        if let Some((page, props)) = self.find_dyn_page_and_props(url_infos).await {
            return (page.as_dyn_component(), props);
        }
        (&*self.not_found_page, PropsUntypedPtr::new_unit())
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> String {
        let (page, props) = self.find_page_and_props(url_infos).await;
        let html = sycamore::render_to_string(|cx| {
            let props = unsafe { page.render_server(cx, props) };
            self.layout.render_server(cx, props)
        });
        html
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

    // pub fn render_client<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<String> {
    //     let (page, props) = self.find_dyn_page_and_props(url_infos).await?;
    //     let html = sycamore::render(|cx| {
    //         let props = unsafe { page.render_server(cx, props) };
    //         self.layout.render_server(cx, props)
    //     });
    //     Some(html)
    // }
}
