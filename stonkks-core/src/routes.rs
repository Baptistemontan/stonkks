use std::{collections::HashMap, hash::Hash, ops::Deref};

use super::pointers::*;

pub trait Route<'url>: Sized + Send + 'url + Hash {
    // the Hash trait bound is only needed for Static pages, but contraining it later
    // on the Static page trait is not possible currently.

    fn try_from_url(url: UrlInfos<'_, 'url>) -> Option<Self>;
}

#[derive(Debug)]
struct OwnedParams<'a>(HashMap<&'a str, &'a str>);

impl<'a> Deref for OwnedParams<'a> {
    type Target = HashMap<&'a str, &'a str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Params<'a, 'url>(&'a OwnedParams<'url>);

impl<'a, 'url> Deref for Params<'a, 'url> {
    type Target = HashMap<&'url str, &'url str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct OwnedSegments<'a>(Box<[&'a str]>);

impl<'a> Deref for OwnedSegments<'a> {
    type Target = [&'a str];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a> FromIterator<&'a str> for OwnedSegments<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let segments = iter.into_iter().collect();
        OwnedSegments(segments)
    }
}

impl<'a> OwnedParams<'a> {
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
        OwnedParams(params_map)
    }

    pub fn to_shared(&self) -> Params<'_, 'a> {
        Params(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Segments<'a, 'url>(&'a [&'url str]);

impl<'a, 'url> Deref for Segments<'a, 'url> {
    type Target = [&'url str];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> OwnedSegments<'a> {
    pub fn to_shared(&self) -> Segments<'_, 'a> {
        Segments(&*self)
    }

    pub fn to_shared_shifted(&self) -> Option<(&'a str, Segments<'_, 'a>)> {
        self.split_first().map(|(first, rest)| {
            let segments = Segments(rest);
            (*first, segments)
        })
    }
}

fn parse_segments<'a>(path: &'a str) -> OwnedSegments {
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

fn parse_url<'a>(url: &'a str) -> (OwnedSegments<'a>, Option<OwnedParams<'a>>) {
    let (path, params) = match url.split_once('?') {
        Some((path, params)) => (path, Some(params)),
        None => (url, None),
    };

    let segments = parse_segments(path);

    let params = params.map(OwnedParams::parse);

    (segments, params)
}

pub struct OwnedUrlInfos<'a> {
    url: &'a str,
    segments: OwnedSegments<'a>,
    params: Option<OwnedParams<'a>>,
}

impl<'a> OwnedUrlInfos<'a> {
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
        OwnedUrlInfos {
            url,
            segments,
            params,
        }
    }

    pub fn to_shared(&self) -> UrlInfos<'_, 'a> {
        let segments = self.segments.to_shared();
        let params = self.params.as_ref().map(|params| params.to_shared());
        UrlInfos {
            url: self.url,
            segments,
            params,
        }
    }

    pub fn to_shared_shifted(&self) -> Option<(&'a str, UrlInfos<'_, 'a>)> {
        let (first, segments) = self.segments.to_shared_shifted()?;
        let params = self.params.as_ref().map(|params| params.to_shared());
        let infos = UrlInfos {
            url: self.url,
            segments,
            params,
        };
        Some((first, infos))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UrlInfos<'a, 'url> {
    url: &'url str,
    segments: Segments<'a, 'url>,
    params: Option<Params<'a, 'url>>,
}

impl<'a, 'url> UrlInfos<'a, 'url> {
    pub fn segments(&self) -> &[&'url str] {
        &self.segments
    }

    pub fn params(&self) -> Option<&HashMap<&'url str, &'url str>> {
        self.params.as_deref()
    }

    pub fn url(&self) -> &'url str {
        self.url
    }
}

pub trait Routable: Send + Sync + 'static {
    type Route<'a>: Route<'a>;

    fn try_match_route<'a, 'url>(url_infos: UrlInfos<'a, 'url>) -> Option<Self::Route<'url>> {
        Self::Route::try_from_url(url_infos)
    }
}

pub trait DynRoutable: Send + Sync + 'static {
    fn try_match_route<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<RouteUntypedPtr<'url>>;
}

impl<T: Routable> DynRoutable for T {
    fn try_match_route<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<RouteUntypedPtr<'url>> {
        let route = <T as Routable>::try_match_route(url_infos)?;
        let route_ptr = RouteUntypedPtr::new::<T>(route);
        Some(route_ptr)
    }
}
