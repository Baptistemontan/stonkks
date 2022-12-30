use std::{collections::HashMap, ops::Deref};

use super::pointers::*;

pub trait Route<'a>: Sized + Send + 'a {
    fn try_from_url(url: &UrlInfos<'a>) -> Option<Self>;
}

#[repr(transparent)]
struct Params<'a>(HashMap<&'a str, &'a str>);

impl<'a> Deref for Params<'a> {
    type Target = HashMap<&'a str, &'a str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(transparent)]
struct Segments<'a>(Box<[&'a str]>);

impl<'a> Deref for Segments<'a> {
    type Target = [&'a str];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a> FromIterator<&'a str> for Segments<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let segments = iter.into_iter().collect();
        Segments(segments)
    }
}

impl<'a> Params<'a> {
    pub fn parse(params: &'a str) -> Self {
        let mut params_map = HashMap::new();
        let iter = params.split('&').map(|segment| segment.split_once('='));
        for params in iter {
            // silently ignore malformed params
            // TODO: figure out what to do in this case.
            if let Some((name, value)) = params {
                params_map.insert(name, value);
            }
        }
        Params(params_map)
    }
}

fn parse_segments<'a>(path: &'a str) -> Segments {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let html = ".html";
            if s.ends_with(html) {
                let index = s.len() - html.len();
                &s[..index]
            } else {
                s
            }
        })
        .collect()
}

fn parse_url<'a>(url: &'a str) -> (Segments<'a>, Option<Params<'a>>) {
    let (path, params) = match url.split_once('?') {
        Some((path, params)) => (path, Some(params)),
        None => (url, None),
    };

    let segments = parse_segments(path);

    let params = params.map(Params::parse);

    (segments, params)
}

pub struct UrlInfos<'a> {
    url: &'a str,
    segments: Segments<'a>,
    params: Option<Params<'a>>,
}

impl<'a> UrlInfos<'a> {
    pub fn segments(&self) -> &[&'a str] {
        &self.segments
    }

    pub fn params(&self) -> Option<&HashMap<&'a str, &'a str>> {
        self.params.as_deref()
    }

    pub fn url(&self) -> &'a str {
        self.url
    }

    pub fn parse_from_url(url: &'a str) -> Self {
        let (segments, params) = parse_url(url);
        UrlInfos {
            url,
            segments,
            params,
        }
    }
}

pub trait Routable: Send + Sync {
    type Route<'a>: Route<'a>;

    fn try_match_route<'url>(url_infos: &UrlInfos<'url>) -> Option<Self::Route<'url>> {
        Self::Route::try_from_url(url_infos)
    }
}

pub trait DynRoutable {
    fn try_match_route<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<RouteUntypedPtr<'url>>;
}

impl<T: Routable> DynRoutable for T {
    fn try_match_route<'url>(&self, url_infos: &UrlInfos<'url>) -> Option<RouteUntypedPtr<'url>> {
        let route = <T as Routable>::try_match_route(url_infos)?;
        let route_ptr = RouteUntypedPtr::new::<T>(route);
        Some(route_ptr)
    }
}
