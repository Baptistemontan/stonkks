use super::default::{AppLayout, NotFound};
use super::prelude::*;
use next_rs_traits::pages::{DynBasePage, DynComponent, DynPageDyn};
use next_rs_traits::pointers::*;

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
    ) -> Option<(&'_ dyn DynPageDyn, RouteUntypedPtr<'url>)> {
        for page in &self.dyn_pages {
            if let Some(route) = page.try_match_route(url_infos) {
                return Some((&**page, route));
            }
        }
        None
    }

    fn find_any_page<'url, 'a, I: IntoIterator<Item = &'a dyn DynBasePage>>(
        pages: I,
        url_infos: &UrlInfos<'url>,
    ) -> Option<&'a dyn DynComponent> {
        pages.into_iter().find_map(|page| {
            page.try_match_route(url_infos)
                .map(|_| page.as_dyn_component())
        })
    }

    pub fn find_page<'url>(&self, url_infos: &UrlInfos<'url>) -> &'_ dyn DynComponent {
        let dyn_pages = self.dyn_pages.iter().map(|page| page.as_dyn_base_page());
        Self::find_any_page(dyn_pages, url_infos).unwrap_or(&*self.not_found_page)
    }

    pub async fn find_dyn_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, PropsUntypedPtr)> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props = unsafe { page.get_server_props(route).await };
        Some((page, props))
    }

    pub async fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        if let Some((page, props)) = self.find_dyn_page_and_props(url_infos).await {
            return (page.as_dyn_component(), props);
        }
        (
            &*self.not_found_page,
            PropsUntypedPtr::new_not_found_props(NotFoundPageProps),
        )
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> (String, String) {
        let (page, props) = self.find_page_and_props(url_infos).await;
        let serialized_props = unsafe { page.serialize_props(&props).unwrap() };
        let html = sycamore::render_to_string(|cx| {
            let props = unsafe { page.render_server(cx, props) };
            self.layout.render_server(cx, props)
        });
        (html, serialized_props)
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

    pub fn not_found<T: NotFoundPage>(mut self, not_found: T) -> Self {
        self.not_found_page = not_found.into();
        self
    }
}
