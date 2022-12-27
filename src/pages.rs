use std::any::Any;
use super::prelude::*;




pub fn find_dyn_page_and_route<'url, 'pages, I>(url_infos: &UrlInfos, pages: I) -> Option<(&'pages dyn DynPageDyn, Box<dyn Any + Send>)>
    where I: IntoIterator<Item = &'pages dyn DynPageDyn>
{    
    for page in pages {
        if let Some(route) = page.try_match_route(url_infos) {
            return Some((page, route));
        }
    }
    None
}

pub async fn find_dyn_page_and_props<'url, 'pages, I>(url_infos: &UrlInfos<'url>, pages: I) -> Option<(&'pages dyn DynPageDyn, Box<dyn Any>)>
    where I: IntoIterator<Item = &'pages dyn DynPageDyn>
{
    let (page, route) = find_dyn_page_and_route(url_infos, pages)?;
    let props = page.get_server_props(route).await;
    Some((page, props))
}