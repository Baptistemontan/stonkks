use next_rs::prelude::*;

pub struct Hello;

pub struct HelloRoute<'a>(pub &'a str);

impl Routable for Hello {
    type Route<'a> = HelloRoute<'a>;
}

impl<'a> Route<'a> for HelloRoute<'a> {
    fn try_from_url(url: &UrlInfos<'a>) -> Option<Self> {
        let mut iter = url.segments().iter().skip(1);
        match (iter.next(), iter.next()) {
            (Some(route), Some(name)) if route == &"hello" => Some(HelloRoute(name)),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl Api for Hello {
    type Err<'a> = &'a str;
    type Ressource = ();
    async fn respond<'url, 'r>(route: Self::Route<'url>, _ressources: ()) -> Result<String, &'url str>{
        Ok(format!("name: {}", route.0))
        // Err(route.0)
    }
}
