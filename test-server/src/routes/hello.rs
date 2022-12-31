use std::sync::atomic::AtomicUsize;

use stonkks::prelude::*;

pub struct Hello;

pub struct HelloRoute<'a>(pub &'a str);

impl Routable for Hello {
    type Route<'a> = HelloRoute<'a>;
}

impl<'a> Route<'a> for HelloRoute<'a> {
    fn try_from_url(url: UrlInfos<'_, 'a>) -> Option<Self> {
        let mut iter = url.segments().iter().cloned();
        match (iter.next(), iter.next()) {
            (Some(route), Some(name)) if route == "hello" => Some(HelloRoute(name)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MyRessource(pub AtomicUsize);

#[async_trait::async_trait]
impl Api for Hello {
    type Err<'a> = &'a str;
    type Ressource = RessourceExtractor<MyRessource>;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        MyRessource(counter): &'r MyRessource,
    ) -> Result<String, &'url str> {
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(format!(
            "{{\"name\":\"{}\",\"count\":\"{}\"}}",
            route.0, count
        ))
        // Err(route.0)
    }
}
