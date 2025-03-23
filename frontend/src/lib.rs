use wasm_bindgen::prelude::*;

mod app;
mod data;
mod env;
mod pages;
mod services;
mod setting;

#[wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<app::App>::new().render();
}
