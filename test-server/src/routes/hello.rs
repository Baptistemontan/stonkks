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

#[derive(Debug)]
pub struct MyRessource(pub String);

#[async_trait::async_trait]
impl Api for Hello {
    type Err<'a> = &'a str;
    type Ressource = RessourceExtractor<MyRessource>;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        ressource: &'r MyRessource,
    ) -> Result<String, &'url str> {
        Ok(format!("name: {}, ressource: {}", route.0, ressource.0))
        // Err(route.0)
    }
}
