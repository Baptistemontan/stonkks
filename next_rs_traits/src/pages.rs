use async_trait::async_trait;
// use sycamore::prelude::*;
use std::any::Any;

use crate::routes::{Params, Segments};

pub trait BasePage {
    fn try_match_route(&self, segments: &Segments, params: Option<&Params>) -> Option<Box<dyn Any>>; // in this case Box<IndexRoute>
    // fn render(cx: Scope, props: Box<dyn Any>) -> View<G>; // IndexPageProps -> View<G>
    // fn hydrate(&self, props: Box<dyn Any>);
}

#[async_trait]
pub trait DynPage: BasePage {
    async fn get_server_props(&self, route: Box<dyn Any>) -> Box<dyn Any>;  // IndexRoute -> IndexPageProps
}
