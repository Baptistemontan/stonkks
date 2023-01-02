use stonkks_traits::api::DynApi;
use stonkks_traits::pointers::*;
use stonkks_traits::predule::*;
use stonkks_traits::routes::UrlInfos;
use stonkks_traits::states::StatesMap;

#[derive(Default)]
pub struct ApiRoutes(Vec<Box<dyn DynApi>>);

impl ApiRoutes {
    pub fn add_route<T: Api>(&mut self, route: T) {
        let route: Box<dyn DynApi> = Box::new(route);
        self.add_boxed_route(route);
    }

    pub fn add_routes<I>(&mut self, routes: I)
    where
        I: IntoIterator<Item = Box<dyn DynApi>>,
    {
        self.0.extend(routes);
    }

    pub fn add_boxed_route(&mut self, route: Box<dyn DynApi>) {
        self.0.push(route);
    }

    pub fn find_api<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
    ) -> Option<(&'_ dyn DynApi, RouteUntypedPtr<'url>)> {
        for api in &self.0 {
            if let Some(route) = api.try_match_route(url_infos) {
                return Some((&**api, route));
            }
        }
        None
    }

    pub async fn find_and_respond<'a, 'url>(
        &self,
        url_infos: UrlInfos<'a, 'url>,
        states: &StatesMap,
    ) -> Option<Result<String, String>> {
        let (api, route) = self.find_api(url_infos)?;
        let response = unsafe { api.respond(route, states).await };
        Some(response)
    }
}
