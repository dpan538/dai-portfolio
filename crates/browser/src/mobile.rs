use crate::dom::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

pub fn init() {
    for strip in all("[data-mobile-carousel]") {
        let frames = Rc::new(within(&strip, "figure"));
        if frames.is_empty() {
            continue;
        }
        for image in within(&strip, "img") {
            attr(&image, "draggable", "false");
        }
        let status = strip
            .parent_element()
            .and_then(|p| p.query_selector("[data-carousel-status]").ok().flatten());
        let update: Rc<dyn Fn()> = Rc::new({
            let (strip, frames) = (strip.clone(), frames.clone());
            move || {
                let left = strip.get_bounding_client_rect().left();
                let index = frames
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        (a.get_bounding_client_rect().left() - left)
                            .abs()
                            .total_cmp(&(b.get_bounding_client_rect().left() - left).abs())
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                if let Some(status) = &status {
                    status.set_text_content(Some(&format!("{} / {}", index + 1, frames.len())));
                }
                for (i, frame) in frames.iter().enumerate() {
                    attr(
                        frame,
                        "aria-current",
                        if i == index { "true" } else { "false" },
                    );
                }
            }
        });
        update();
        on(&strip, "scroll", {
            let update = update.clone();
            move |_| update()
        });
        on(&strip, "keydown", {
            let strip = strip.clone();
            move |e| {
                let Ok(k) = e.dyn_into::<KeyboardEvent>() else {
                    return;
                };
                if matches!(k.key().as_str(), "ArrowLeft" | "ArrowRight") {
                    k.prevent_default();
                    k.stop_propagation();
                    let step = strip.get_bounding_client_rect().width() + 24.;
                    scroll(
                        &strip,
                        if k.key() == "ArrowLeft" { -step } else { step },
                        0.,
                    );
                }
            }
        });
    }
}
