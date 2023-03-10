mod routes;
mod states;
use routes::counter::CountApi;
use states::counter::CounterState;

use std::{ops::Deref, sync::Arc};

use rocket::{
    fs::{relative, FileServer},
    http::{ContentType as RocketContentType, Method, Status},
    outcome::Outcome,
    response::Responder,
    route::Handler,
    Catcher, Data, Response, Route as RocketRoute,
};
use stonkks::prelude::{ServerResponse as StonkksResponse, *};
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

fn convert_content_type(content_type: ContentType) -> RocketContentType {
    match content_type {
        ContentType::Text => RocketContentType::Text,
        ContentType::Json => RocketContentType::JSON,
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
        let _test = Test::new();
        let url = Uri::from_request(request);
        let result = self.0.respond(&url).await;
        match result {
            Some(Ok(StonkksResponse::Html(html))) => {
                let response = (RocketContentType::HTML, html).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            Some(Ok(StonkksResponse::Api(api_response))) => {
                let content_type = convert_content_type(api_response.content_type);
                let content = api_response.content;
                let response = (content_type, content).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            Some(Ok(StonkksResponse::Props(props))) => {
                let response = (RocketContentType::JSON, props).respond_to(request);
                match response {
                    Ok(rep) => Outcome::Success(rep),
                    Err(status) => Outcome::Failure(status),
                }
            }
            Some(Err(err)) => {
                error_!("An error occured at {} : {}", url.url(), err);
                Outcome::Failure(Status::InternalServerError)
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
        (Status::NotFound, (RocketContentType::HTML, html)).respond_to(request)
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let state = CounterState::default();
    let app = get_app().state_unwrap(state).api(CountApi).into_server();

    app.generate_static_pages().await.unwrap();

    let app = Arc::new(app);
    let server = MyServer(Arc::clone(&app));

    let not_found = NotFound(app);
    let not_found_catcher = Catcher::new(404, not_found);
    let _rocket = rocket::build()
        .mount("/public", FileServer::from(relative!("static")))
        .mount("/", server)
        .register("/", [not_found_catcher])
        .launch()
        .await?;
    Ok(())
}

struct Test(std::time::Instant);

impl Test {
    fn new() -> Self {
        Test(std::time::Instant::now())
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        let elapsed = self.0.elapsed();
        println!("ELAPSED: {:?}", elapsed);
    }
}
