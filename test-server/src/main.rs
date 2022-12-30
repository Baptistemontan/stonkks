use next_rs::prelude::*;
use rocket::{
    fs::{relative, FileServer},
    get, launch,
    response::content::RawHtml,
    routes, State,
};
use test_client::get_app;

use rocket::request::{FromRequest, Outcome, Request};

struct Uri<'a>(pub UrlInfos<'a>);

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for Uri<'r> {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let url = request.uri().path().as_str();
        let url_infos = UrlInfos::parse_from_url(url);
        Outcome::Success(Self(url_infos))
    }
}

#[get("/<_..>", rank = 11)]
async fn hello(uri: Uri<'_>, app: &State<Server>) -> RawHtml<String> {
    let Uri(url_infos) = uri;
    println!("{}", url_infos.url());
    let html = app.render_to_string(&url_infos).await;
    RawHtml(html)
}

#[launch]
fn rocket() -> _ {
    let server = get_app().into_server();
    rocket::build()
        .manage(server)
        .mount("/public", FileServer::from(relative!("static")))
        .mount("/", routes![hello])
}
