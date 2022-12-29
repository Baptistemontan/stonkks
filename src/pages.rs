use super::prelude::*;
use next_rs_traits::pages::{DynBasePage, DynPageDyn};
use next_rs_traits::pointers::*;

type BoxedDynPage = Box<dyn DynPageDyn>;

#[derive(Default)]
pub struct DynPages(Vec<BoxedDynPage>);

impl DynPages {
    pub fn find_dyn_page_and_route<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynPageDyn, RouteUntypedPtr<'url>)> {
        for page in &self.0 {
            if let Some(route) = page.try_match_route(url_infos) {
                return Some((&**page, route));
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

    pub fn add_dyn_page<T: DynPage + 'static>(&mut self, page: T) {
        self.add_boxed_dyn_page(Box::new(page));
    }

    pub fn add_boxed_dyn_page(&mut self, page: BoxedDynPage) {
        self.0.push(page);
    }

    pub fn add_boxed_dyn_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = BoxedDynPage>,
    {
        self.0.extend(pages)
    }

    pub fn iter_as_base_page(&self) -> impl IntoIterator<Item = &'_ dyn DynBasePage> {
        self.0.iter().map(|page| page.as_dyn_base_page())
    }
}
