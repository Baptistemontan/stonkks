use next_rs_traits::pages::{DynPageDyn, DynBasePage};
use next_rs_traits::pointers::*;
use super::prelude::*;



#[derive(Default)]
pub struct DynPages(Vec<Box<dyn DynPageDyn>>);

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
        self.0.push(Box::new(page));
    }

    pub fn iter_as_base_page(&self) -> impl IntoIterator<Item = &'_ dyn DynBasePage> {
        self.0.iter().map(|page| page.as_dyn_base_page())
    } 
}
