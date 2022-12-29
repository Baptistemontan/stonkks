use crate::app::{AppInner, NEXT_RS_WINDOW_OBJECT_KEY, SERIALIZED_PROPS_KEY};

use super::pages::DynPages;
use super::prelude::*;
use next_rs_traits::layout::DynLayout;
use next_rs_traits::pages::DynComponent;
use next_rs_traits::pointers::*;
use sycamore::prelude::*;

pub struct Server {
    inner: AppInner,
}

impl From<AppInner> for Server {
    fn from(inner: AppInner) -> Self {
        Server { inner }
    }
}

impl Server {
    fn dyn_pages(&self) -> &DynPages {
        self.inner.dyn_pages()
    }

    fn not_found_page(&self) -> &dyn DynComponent {
        self.inner.not_found_page()
    }

    fn layout(&self) -> &dyn DynLayout {
        self.inner.layout()
    }

    pub async fn find_page_and_props<'url>(
        &self,
        url_infos: &UrlInfos<'url>,
    ) -> (&'_ dyn DynComponent, PropsUntypedPtr) {
        if let Some((page, props)) = self.dyn_pages().find_dyn_page_and_props(url_infos).await {
            return (page.as_dyn_component(), props);
        }
        (self.not_found_page(), NotFoundPageProps::new_untyped())
    }

    pub async fn render_to_string<'url>(&self, url_infos: &UrlInfos<'url>) -> String {
        let (page, props) = self.find_page_and_props(url_infos).await;
        let serialized_props = unsafe { page.serialize_props(&props).unwrap() };
        let script = Self::window_object_script(&serialized_props);
        let html = sycamore::render_to_string(|cx| {
            let props = unsafe { page.render_server(cx, props) };
            let body = self.layout().render_server(cx, props);
            default_document_view(cx, body, script)
        });
        html
    }

    fn window_object_script(props: &str) -> String {
        format!("window.{0}=window.{0}||{{}};window.{0}.{1}=\'{2}\'", NEXT_RS_WINDOW_OBJECT_KEY, SERIALIZED_PROPS_KEY, props)
    }
}

fn default_document_view<G: Html>(cx: Scope, body: View<G>, script: String) -> View<G> {
    view! { cx,
        html(lang = "en") {
            head {
                meta(charset = "UTF-8")
                meta(http-equiv="X-UA-Compatible", content="IE=edge")
                meta(name="viewport", content="width=device-width, initial-scale=1.0")
                script {
                    (script)
                }
            }
            body {
                (body)
            }
        }
    }
}