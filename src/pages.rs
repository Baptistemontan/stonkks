use crate::utils::{DynPageAndRoute, PageAndProps, StaticPageAndRoute};

use super::prelude::*;
use std::hash::Hash;
use stonkks_core::pages::{DynBasePage, DynPageDyn, DynStaticPage, StaticPage};
use stonkks_core::routes::UrlInfos;
use stonkks_core::states::StatesMap;

type BoxedDynPage = Box<dyn DynPageDyn>;
type BoxedStaticPage = Box<dyn DynStaticPage>;

#[derive(Default)]
pub struct DynPages(Vec<BoxedDynPage>);

impl DynPages {
    pub(crate) fn find_dyn_page_and_route<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<DynPageAndRoute<'_, 'url>> {
        self.0
            .iter()
            .find_map(|page| DynPageAndRoute::try_match_route(&**page, url_infos))
    }

    pub(crate) async fn find_dyn_page_and_props<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
        states: &StatesMap,
    ) -> Option<Result<PageAndProps<'_>, String>> {
        let page_and_route = self.find_dyn_page_and_route(url_infos)?;
        let result = page_and_route.get_props(states).await;
        Some(result)
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
    pub(crate) fn find_static_page<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<StaticPageAndRoute<'_, 'url>> {
        self.0
            .iter()
            .find_map(move |page| StaticPageAndRoute::try_match_route(&**page, url_infos))
    }

    pub fn add_page<T: StaticPage>(&mut self, page: T)
    where
        for<'a> <T as Routable>::Route<'a>: Hash,
    {
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

    pub fn iter(&self) -> impl Iterator<Item = &'_ dyn DynStaticPage> {
        self.0.iter().map(|page| &**page)
    }
}
