use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{Element, EventTarget, KeyboardEvent, console};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
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

    let keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        console::log_1(&format!("aaa: {}", e.key()).into());
    }) as Box<dyn FnMut(_)>);

    document()
        .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
        .unwrap();

    keydown.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if i > 300 {
            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            body().set_text_content(Some("All done!"));

            let _ = f.borrow_mut().take();
            return;
        }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        let text = format!("requestAnimationFrame has been called {} times.", i);

        body().set_text_content(Some(&text));
        console::log_1(&text.into());

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
