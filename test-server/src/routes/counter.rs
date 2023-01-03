use serde::Serialize;
use stonkks::prelude::*;

use crate::states::counter::CounterState;

pub struct CountApi;

#[derive(Hash)]
pub struct CountRoute<'a> {
    name: &'a str,
}

impl Routable for CountApi {
    type Route<'a> = CountRoute<'a>;
}

impl<'a> Route<'a> for CountRoute<'a> {
    fn try_from_url(url: UrlInfos<'_, 'a>) -> Option<Self> {
        let mut iter = url.segments().iter().cloned();
        match (iter.next(), iter.next()) {
            (Some(route), Some(name)) if route == "hello" => Some(CountRoute { name }),
            _ => None,
        }
    }
}

#[derive(Serialize)]
pub struct CounterResponse<'a> {
    name: &'a str,
    count: usize,
}

#[async_trait::async_trait]
impl Api for CountApi {
    type Err<'a> = &'a str;
    type State<'r> = State<&'r CounterState>;
    type Output<'url> = Json<CounterResponse<'url>>;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        counter: State<&'r CounterState>,
    ) -> Result<Self::Output<'url>, Self::Err<'url>> {
        let CountRoute { name } = route;
        if name == "world" {
            return Err(name);
        }

        let count = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let response = Json(CounterResponse { count, name });
        Ok(response)
    }
}
