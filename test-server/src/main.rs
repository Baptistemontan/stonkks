mod routes;
use routes::hello::{Hello, MyCounter};

use std::{ops::Deref, sync::Arc};

use rocket::{
    fs::{relative, FileServer},
    http::{ContentType, Method, Status},
    launch,
    outcome::Outcome,
    response::Responder,
    route::Handler,
    Catcher, Data, Response, Route as RocketRoute,
};
use stonkks::prelude::{Response as StonkksResponse, *};
use test_client::get_app;

use rocket::log::error_;

use rocket::catcher::Result as CatcherResult;

use rocket::request::Request;

struct Uri<'a>(pub OwnedUrlInfos<'a>);

impl<'a> Deref for Uri<'a> {
    type Target = OwnedUrlInfos<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Uri<'a> {
    pub fn from_request(request: &'a Request<'_>) -> Self {
        let url = request.uri().path().as_str();
        let url_infos = OwnedUrlInfos::parse_from_url(url);
        Self(url_infos)
    }
}

#[derive(Clone)]
struct MyServer(Arc<Server>);

#[async_trait::async_trait]
impl Handler for MyServer {
    async fn handle<'r>(
        &self,
        request: &'r Request<'_>,
        data: Data<'r>,
    ) -> Outcome<Response<'r>, Status, Data<'r>> {
        let url = Uri::from_request(request);
        let result = self.0.respond(&url).await;
        match result {
            Some(Ok(StonkksResponse::Html(html))) => {
                let response = (ContentType::HTML, html).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            Some(Ok(StonkksResponse::Api(api_response))) => {
                let response = (ContentType::JSON, api_response).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            Some(Err(err)) => {
                error_!("An error occured at {} : {}", url.url(), err);
                Outcome::Failure(Status::InternalServerError)
            }
            Some(Ok(StonkksResponse::Props(props))) => {
                let response = (ContentType::JSON, props).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            None => Outcome::Forward(data),
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
impl rocket::catcher::Handler for NotFound {
    async fn handle<'r>(&self, _status: Status, request: &'r Request<'_>) -> CatcherResult<'r> {
        let html = self.0.render_not_found();
        let html = match html {
            Ok(html) => html,
            Err(err) => {
                let uri = Uri::from_request(request);
                error_!(
                    "An error occured at {} while rendering the 404 page: {}",
                    uri.url(),
                    err
                );
                return Err(Status::InternalServerError);
            }
        };
        (Status::NotFound, (ContentType::HTML, html)).respond_to(request)
    }
}

#[launch]
fn rocket() -> _ {
    let state = MyCounter(0.into());
    let app = get_app().state_unwrap(state).api(Hello).into_server();

    let app = Arc::new(app);
    let server = MyServer(Arc::clone(&app));
    let not_found = NotFound(app);
    let not_found_catcher = Catcher::new(404, not_found);
    rocket::build()
        .mount("/public", FileServer::from(relative!("static")))
        .mount("/", server)
        .register("/", [not_found_catcher])
}
