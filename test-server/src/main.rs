use std::{ops::Deref, sync::Arc};

use next_rs::prelude::*;
use rocket::{
    fs::{relative, FileServer},
    Route as RocketRoute,
    get, launch,
    response::{content::RawHtml, Responder},
    routes, State, outcome::Outcome, route::Handler, Data, Response, http::{Status, ContentType, Method},
};
use test_client::get_app;

use rocket::request::Request;

struct Uri<'a>(pub UrlInfos<'a>);

impl<'a> Deref for Uri<'a> {
    type Target = UrlInfos<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Uri<'a> {
    pub fn from_request(request: &'a Request<'_>) -> Self {
        let url = request.uri().path().as_str();
        let url_infos = UrlInfos::parse_from_url(url);
        Self(url_infos)
    }
}

#[derive(Clone)]
struct MyServer(Arc<Server>);

#[async_trait::async_trait]
impl Handler for MyServer {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<Response<'r>, Status, Data<'r>>{
        let url = Uri::from_request(request);
        let result = self.0.try_render_to_string(&url).await;
        match result {
            Some(html) => {
                let response = (ContentType::HTML, html).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status)
                }
            },
            None => Outcome::Forward(data)
        }
    }
}

impl Into<Vec<RocketRoute>> for MyServer {
    fn into(self) -> Vec<RocketRoute> {
        vec![RocketRoute::new(Method::Get, "/<_..>", self)]
    }
}

#[derive(Clone)]
struct NotFound(Arc<Server>);

#[async_trait::async_trait]
impl Handler for NotFound {
    async fn handle<'r>(&self, request: &'r Request<'_>, _data: Data<'r>) -> Outcome<Response<'r>, Status, Data<'r>>{
        let html = self.0.render_not_found();
        let response = (Status::NotFound, (ContentType::HTML, html)).respond_to(request);
        match response {
            Ok(rep) => Outcome::Success(rep),
            Err(status) => Outcome::Failure(status)
        }
    }
}

impl Into<Vec<RocketRoute>> for NotFound {
    fn into(self) -> Vec<RocketRoute> {
        vec![RocketRoute::ranked(isize::MAX - 1, Method::Get, "/<_..>", self)]
    }
}


#[launch]
fn rocket() -> _ {
    let app = get_app().into_server();
    let app = Arc::new(app);
    let server = MyServer(Arc::clone(&app));
    let not_found = NotFound(app);
    rocket::build()
        .mount("/public", FileServer::from(relative!("static")).rank(2))
        .mount("/", server)
        .mount("/", not_found)
}
