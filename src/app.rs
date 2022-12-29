use super::default::{AppLayout, NotFound};
use super::prelude::*;
use next_rs_traits::pages::{DynBasePage, DynComponent};
use next_rs_traits::pointers::*;
use super::pages::{DynPages};

#[derive(Default)]
pub struct App {
    dyn_pages: DynPages,
    layout: AppLayout,
    not_found_page: NotFound,
}

impl App {
    pub fn new() -> Self {
        Self::default()
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
        let dyn_pages = self.dyn_pages.iter_as_base_page();
        Self::find_any_page(dyn_pages, url_infos).unwrap_or(&*self.not_found_page)
    }

    pub async fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        if let Some((page, props)) = self.dyn_pages.find_dyn_page_and_props(url_infos).await {
            return (page.as_dyn_component(), props);
        }
        (
            &*self.not_found_page,
            NotFoundPageProps::new_untyped()
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

    pub fn dyn_page<T: DynPage + 'static>(mut self, page: T) -> Self {
        self.dyn_pages.add_dyn_page(page);
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
