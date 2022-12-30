use test_client::get_app;


fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    get_app().into_client().run();
}

// #[tokio::main]
// async fn main() {
//     let url = "counter/56";
//     let url_infos = UrlInfos::parse_from_url(url);
//     let rendered_html = get_app().into_server().render_to_string(&url_infos).await;
//     println!("{}", rendered_html);
// }
