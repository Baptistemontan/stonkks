use std::any::Any;

use super::prelude::*;

fn parse_segments<'a>(path: &'a str) -> Segments {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

fn parse_url<'a>(url: &'a str) -> (Segments<'a>, Option<Params<'a>>) {
    let (path, params) = match url.split_once('?') {
        Some((path, params)) => (path, Some(params)),
        None => (url, None)
    };
    
    let segments = parse_segments(path);
    // for now if params fails to parse due to invalid url parameters, continue without params.
    // TODO: figure out what to do.
    let params = params.and_then(Params::try_parse);

    (segments, params)
}


pub fn find_dyn_page<'pages>(url: &str, pages: &'pages [Box<dyn DynPage>]) -> Option<(&'pages dyn DynPage, Box<dyn Any>)> {
    let (segments, params) = parse_url(url);
    
    for page in pages {
        if let Some(route) = page.try_match_route(&segments, params.as_ref()) {
            return Some((&**page, route));
        }
    }
    None
}

pub async fn find_dyn_page_and_props<'pages>(url: &str, pages: &'pages [Box<dyn DynPage>]) -> Option<(&'pages dyn DynPage, Box<dyn Any>)> {
    let (page, route) = find_dyn_page(url, pages)?;
    let props = page.get_server_props(route).await;
    Some((page, props))
}