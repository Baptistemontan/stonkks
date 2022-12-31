use super::prelude::*;
use next_rs_traits::pages::{DynBasePage, DynPageDyn, DynStaticPage, StaticPage};
use next_rs_traits::pointers::*;
use next_rs_traits::routes::UrlInfos;

type BoxedDynPage = Box<dyn DynPageDyn>;
type BoxedStaticPage = Box<dyn DynStaticPage>;

#[derive(Default)]
pub struct DynPages(Vec<BoxedDynPage>);

impl DynPages {
    pub fn find_dyn_page_and_route<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<(&'_ dyn DynPageDyn, RouteUntypedPtr<'url>)> {
        for page in &self.0 {
            if let Some(route) = page.try_match_route(url_infos) {
                return Some((&**page, route));
            }
        }
        None
    }
    pub async fn find_dyn_page_and_props<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<Result<(&'_ dyn DynPageDyn, PropsUntypedPtr), String>> {
        let (page, route) = self.find_dyn_page_and_route(url_infos)?;
        let props_result = unsafe { page.get_server_props(route).await };
        Some(props_result.map(|props| (page, props)))
    }

    pub fn add_page<T: DynPage>(&mut self, page: T) {
        self.add_boxed_page(Box::new(page));
    }

    pub fn add_boxed_page(&mut self, page: BoxedDynPage) {
        self.0.push(page);
    }

    pub fn add_boxed_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = BoxedDynPage>,
    {
        self.0.extend(pages)
    }

    pub fn iter_as_base_page(&self) -> impl Iterator<Item = &'_ dyn DynBasePage> {
        self.0.iter().map(|page| page.as_dyn_base_page())
    }
}

#[derive(Default)]
pub struct StaticPages(Vec<BoxedStaticPage>);

impl StaticPages {
    pub fn find_static_page<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<&'_ dyn DynStaticPage> {
        for page in &self.0 {
            if let Some(_) = page.try_match_route(url_infos) {
                return Some(&**page);
            }
        }
        None
    }

    pub fn add_page<T: StaticPage>(&mut self, page: T) {
        self.add_boxed_page(Box::new(page));
    }

    pub fn add_boxed_page(&mut self, page: BoxedStaticPage) {
        self.0.push(page);
    }

    pub fn add_boxed_pages<I>(&mut self, pages: I)
    where
        I: IntoIterator<Item = BoxedStaticPage>,
    {
        self.0.extend(pages)
    }

    pub fn iter_as_base_page(&self) -> impl Iterator<Item = &'_ dyn DynBasePage> {
        self.0.iter().map(|page| page.as_dyn_base_page())
    }
}
