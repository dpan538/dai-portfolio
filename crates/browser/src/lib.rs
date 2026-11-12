mod dom;
#[cfg(feature = "review")]
mod editorial;
mod flow;
mod gallery;
mod info;
mod menu;
mod mobile;
mod quite_off;
mod routes;
use dom::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    if routes::redirect() {
        return;
    }
    routes::init();
    #[cfg(feature = "review")]
    editorial::init();
    menu::init();
    info::init();
    gallery::init();
    quite_off::init();
    flow::init();
    mobile::init();
    if let Some(hint) = one("#home_hint") {
        delay(3600, move || class(&hint, "is-visible", true));
    }
    if let Some(body) = doc().body() {
        attr(body.as_ref(), "data-runtime", "rust-wasm");
    }
}
