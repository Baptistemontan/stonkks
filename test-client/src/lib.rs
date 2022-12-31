use stonkks::prelude::*;

mod counter;
mod index;

pub fn get_app() -> App {
    App::new()
        .dyn_page(counter::Counter)
        .static_page(index::Index)
}
