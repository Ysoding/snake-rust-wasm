use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, console};

mod game;
use game::*;
mod render;
mod utils;
use render::*;

thread_local! {
    static GAME: RefCell<Option<Game<WebPlatformRenderer>>> = RefCell::new(None);
    static PREV_TIMESTAMP: RefCell<f32> = RefCell::new(0.0);
}

#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();

    let keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        GAME.with(|game| {
            let mut game_ref = game.borrow_mut();
            let game = game_ref.as_mut().unwrap();
            game.keydown(&e.key());
        });
    }) as Box<dyn FnMut(_)>);

    document()
        .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
        .unwrap();

    keydown.forget();

    let canvas = document().get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    GAME.with(|game| {
        let mut g = Game::new(WebPlatformRenderer::new(ctx));
        g.restart(canvas.width(), canvas.height());
        *game.borrow_mut() = Some(g);
    });

    game_loop_fn_start();
}

fn game_loop_fn_start() {
    window()
        .request_animation_frame(
            Closure::wrap(Box::new(|ts| game_loop_fn(ts)) as Box<dyn FnMut(f32)>)
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap();
}

fn game_loop_fn(timestamp: f32) {
    PREV_TIMESTAMP.with(|prev| {
        let dt = (timestamp - *prev.borrow()) as f32 / 1000.0;
        *prev.borrow_mut() = timestamp;

        GAME.with(|game| {
            let mut game_ref = game.borrow_mut();
            let game = game_ref.as_mut().unwrap();
            game.update(dt);
            game.render();
        });
    });

    game_loop_fn_start();
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window().document().expect("no document")
}
