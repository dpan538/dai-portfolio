use crate::dom::*;
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{AddEventListenerOptions, KeyboardEvent, PointerEvent, WheelEvent};

#[derive(Default)]
struct HorizontalGesture {
    last_event: Option<f64>,
    distance: f64,
    changed: bool,
}
impl HorizontalGesture {
    fn push(&mut self, dx: f64, dy: f64, now: f64) -> (bool, Option<i32>) {
        if dx.abs() <= dy.abs() {
            return (false, None);
        }
        if self
            .last_event
            .map(|last| now - last > 180.)
            .unwrap_or(true)
        {
            self.distance = 0.;
            self.changed = false;
        }
        self.last_event = Some(now);
        if self.changed {
            return (true, None);
        }
        self.distance += dx;
        if self.distance.abs() < 48. {
            return (true, None);
        }
        self.changed = true;
        (true, Some(if self.distance > 0. { 1 } else { -1 }))
    }
}
fn swipe_step(dx: i32, dy: i32) -> Option<i32> {
    (dx.abs() >= 48 && dx.abs() as f64 > dy.abs() as f64 * 1.2).then_some(if dx < 0 {
        1
    } else {
        -1
    })
}
fn gestures(step: Rc<dyn Fn(i32)>) {
    if let Some(page) = one("#qo_page") {
        let gesture = RefCell::new(HorizontalGesture::default());
        let wheel_step = step.clone();
        let callback = Closure::wrap(Box::new(move |event: WheelEvent| {
            if event.ctrl_key() || editing(event.as_ref()) {
                return;
            }
            let (dx, dy) = if event.shift_key() && event.delta_y().abs() > event.delta_x().abs() {
                (event.delta_y(), 0.)
            } else {
                (event.delta_x(), event.delta_y())
            };
            let unit = match event.delta_mode() {
                1 => 16.,
                2 => 600.,
                _ => 1.,
            };
            let (handled, direction) =
                gesture
                    .borrow_mut()
                    .push(dx * unit, dy * unit, event.time_stamp());
            if handled {
                event.prevent_default();
                event.stop_propagation();
                if let Some(direction) = direction {
                    wheel_step(direction);
                }
            }
        }) as Box<dyn FnMut(WheelEvent)>);
        let options = AddEventListenerOptions::new();
        options.set_passive(false);
        let _ = page.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            callback.as_ref().unchecked_ref(),
            &options,
        );
        callback.forget();
    }
    let Some(stage) = one(".qo_stage") else {
        return;
    };
    if let Some(image) = one("#qo-image") {
        attr(&image, "draggable", "false");
    }
    let start = Rc::new(Cell::new(None));
    on(&stage, "pointerdown", {
        let (start, stage) = (start.clone(), stage.clone());
        move |event| {
            let Ok(pointer) = event.dyn_into::<PointerEvent>() else {
                return;
            };
            if !pointer.is_primary() || pointer.button() != 0 {
                return;
            }
            start.set(Some((
                pointer.pointer_id(),
                pointer.client_x(),
                pointer.client_y(),
            )));
            let _ = stage.set_pointer_capture(pointer.pointer_id());
        }
    });
    on(&stage, "pointerup", {
        let (start, stage) = (start.clone(), stage.clone());
        move |event| {
            let Ok(pointer) = event.dyn_into::<PointerEvent>() else {
                return;
            };
            let Some((id, x, y)) = start.take() else {
                return;
            };
            let _ = stage.release_pointer_capture(id);
            if id == pointer.pointer_id()
                && let Some(direction) = swipe_step(pointer.client_x() - x, pointer.client_y() - y)
            {
                step(direction);
            }
        }
    });
    on(&stage, "pointercancel", move |_| start.set(None));
}

#[derive(Deserialize, Clone)]
struct Image {
    src: String,
    srcset: Option<String>,
    sizes: Option<String>,
    alt: String,
    width: Option<u32>,
    height: Option<u32>,
}
#[derive(Deserialize)]
struct Data {
    images: Vec<Image>,
    #[serde(rename = "semanticMap")]
    semantic_map: BTreeMap<String, Vec<String>>,
}
pub fn init() {
    let (Some(img), Some(data)) = (one("#qo-image"), one("#qo-image-data")) else {
        return;
    };
    let Ok(data) = serde_json::from_str::<Data>(&data.text_content().unwrap_or_default()) else {
        return;
    };
    let data = Rc::new(data);
    if data.images.is_empty() {
        return;
    }
    let index = Rc::new(Cell::new(0usize));
    let busy = Rc::new(Cell::new(false));
    let semantic: Rc<dyn Fn(usize)> = Rc::new({
        let data = data.clone();
        move |i| {
            for span in all(".semantic") {
                let key = span.get_attribute("data-key").unwrap_or_default();
                let active = data
                    .semantic_map
                    .get(&i.to_string())
                    .map(|keys| keys.contains(&key))
                    .unwrap_or(false);
                class(&span, "semantic-active", active);
            }
            if let (Some(panel), Some(active)) =
                (one(".qo_text"), one(".qo_narrative .semantic-active"))
            {
                let viewport = panel.get_bounding_client_rect();
                let paragraph = active.closest("p").ok().flatten().unwrap_or(active.clone());
                let target =
                    if paragraph.get_bounding_client_rect().height() <= viewport.height() - 24. {
                        paragraph
                    } else {
                        active
                    };
                let bounds = target.get_bounding_client_rect();
                let delta = if bounds.top() < viewport.top() + 12. {
                    bounds.top() - viewport.top() - 12.
                } else if bounds.bottom() > viewport.bottom() - 12. {
                    bounds.bottom() - viewport.bottom() + 12.
                } else {
                    0.
                };
                if delta.abs() > 1. {
                    scroll(&panel, 0., delta);
                }
            }
        }
    });
    semantic(0);
    let step: Rc<dyn Fn(i32)> = Rc::new({
        let (index, busy, data, img, semantic) = (
            index.clone(),
            busy.clone(),
            data.clone(),
            img.clone(),
            semantic.clone(),
        );
        move |delta| {
            if busy.replace(true) {
                return;
            }
            let next = (index.get() as i32 + delta).rem_euclid(data.images.len() as i32) as usize;
            index.set(next);
            let image = data.images[next].clone();
            if !reduced() {
                class(&img, "is-fading-out", true);
            }
            delay(if reduced() { 0 } else { 180 }, {
                let (img, busy, semantic) = (img.clone(), busy.clone(), semantic.clone());
                move || {
                    if let Some(srcset) = image.srcset {
                        attr(&img, "srcset", &srcset);
                    }
                    if let Some(sizes) = image.sizes {
                        attr(&img, "sizes", &sizes);
                    }
                    attr(&img, "src", &image.src);
                    attr(&img, "alt", &image.alt);
                    attr(&img, "data-index", &next.to_string());
                    if let Some(w) = image.width {
                        attr(&img, "width", &w.to_string());
                    }
                    if let Some(h) = image.height {
                        attr(&img, "height", &h.to_string());
                    }
                    class(&img, "is-fading-out", false);
                    class(&img, "is-fading-in", true);
                    semantic(next);
                    busy.set(false);
                }
            });
        }
    });
    for (sel, delta) in [(".qo_prev", -1), (".qo_next", 1)] {
        if let Some(btn) = one(sel) {
            on(&btn, "click", {
                let step = step.clone();
                move |_| step(delta)
            });
        }
    }
    gestures(step.clone());
    on(&doc(), "keydown", {
        let step = step.clone();
        move |e| {
            if editing(&e) || one("#review-panel:focus-within").is_some() {
                return;
            }
            if let Ok(k) = e.dyn_into::<KeyboardEvent>() {
                match k.key().as_str() {
                    "ArrowLeft" | "ArrowUp" => {
                        k.prevent_default();
                        step(-1);
                    }
                    "ArrowRight" | "ArrowDown" => {
                        k.prevent_default();
                        step(1);
                    }
                    _ => {}
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_momentum_changes_one_frame() {
        let mut gesture = HorizontalGesture::default();
        assert_eq!(gesture.push(30., 2., 0.), (true, None));
        assert_eq!(gesture.push(20., 1., 20.), (true, Some(1)));
        assert_eq!(gesture.push(300., 0., 50.), (true, None));
        assert_eq!(gesture.push(12., 0., 120.), (true, None));
    }

    #[test]
    fn a_new_gesture_can_reverse_direction() {
        let mut gesture = HorizontalGesture::default();
        assert_eq!(gesture.push(60., 0., 0.), (true, Some(1)));
        assert_eq!(gesture.push(-60., 0., 250.), (true, Some(-1)));
    }

    #[test]
    fn vertical_wheel_reading_is_preserved() {
        let mut gesture = HorizontalGesture::default();
        assert_eq!(gesture.push(8., 90., 0.), (false, None));
        assert_eq!(gesture.push(0., -100., 20.), (false, None));
        assert_eq!(gesture.push(48., 0., 40.), (true, Some(1)));
    }

    #[test]
    fn dragging_needs_a_deliberate_horizontal_swipe() {
        assert_eq!(swipe_step(-70, 8), Some(1));
        assert_eq!(swipe_step(70, 8), Some(-1));
        assert_eq!(swipe_step(30, 2), None);
        assert_eq!(swipe_step(60, 100), None);
    }
}
