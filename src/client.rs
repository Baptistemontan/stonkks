use crate::app::AppInner;

use super::pages::DynPages;
use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::{DynBasePage, DynComponent};
use next_rs_traits::pointers::*;
use serde_json::Error;

pub struct Client {
    inner: AppInner,
}

impl From<AppInner> for Client {
    fn from(inner: AppInner) -> Self {
        Client { inner }
    }
}

impl Client {
    fn dyn_pages(&self) -> &DynPages {
        self.inner.dyn_pages()
    }

    fn not_found_page(&self) -> &dyn DynComponent {
        self.inner.not_found_page()
    }

    fn layout(&self) -> &dyn DynLayout {
        self.inner.layout()
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

    fn find_page<'url>(&self, url_infos: &UrlInfos<'url>) -> &'_ dyn DynComponent {
        let dyn_pages = self.dyn_pages().iter_as_base_page();
        Self::find_any_page(dyn_pages, url_infos).unwrap_or(self.not_found_page())
    }

    fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
        serialized_props: &str,
    ) -> Result<(&'_ dyn DynComponent, PropsUntypedPtr), Error> {
        let page = self.find_page(url_infos);
        let props = page.deserialize_props(serialized_props)?;
        Ok((page, props))
    }

    pub fn hydrate<'url>(&self, url: &'url str, serialized_props: &str) {
        let url_infos = UrlInfos::parse_from_url(url);
        let (page, props) = self
            .find_page_and_props(&url_infos, serialized_props)
            .expect("Error appened deserializing the props");

        sycamore::hydrate(|cx| {
            let props = unsafe { page.hydrate(cx, props) };
            self.layout().hydrate(cx, props)
        })
    }
}
