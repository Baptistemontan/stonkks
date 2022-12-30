use next_rs_traits::api::DynApi;
use next_rs_traits::pointers::*;
use next_rs_traits::predule::*;


#[derive(Default)]
pub struct ApiRoutes(Vec<Box<dyn DynApi>>);

impl ApiRoutes {
    pub fn add_route<T: Api>(&mut self, route: T) {
        let route: Box<dyn DynApi> = Box::new(route);
        self.add_boxed_route(route);
    }

    pub fn add_boxed_route(&mut self, route: Box<dyn DynApi>) {
        self.0.push(route);
    }

    pub fn find_api<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<(&'_ dyn DynApi, RouteUntypedPtr<'url>)> {
        for api in &self.0 {
            if let Some(route) = api.try_match_route(url_infos) {
                return Some((&**api, route));
            }
        }
        None
    }

    pub async fn find_and_respond<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> Option<String> {
        let (api, route) = self.find_api(url_infos)?;
        let response = unsafe {
            api.respond(route).await
        };
        Some(response)
    }
}