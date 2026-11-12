use crate::{dom::*, gallery::Gallery};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{AddEventListenerOptions, Element, KeyboardEvent, PointerEvent, WheelEvent};

// Lock each wheel burst to its first axis and consume horizontal momentum only once.
#[derive(Default)]
struct SpreadGesture {
    last_event: Option<f64>,
    distance: f64,
    axis: i8,
    changed: bool,
}
impl SpreadGesture {
    fn push(&mut self, dx: f64, dy: f64, now: f64) -> (bool, Option<i32>) {
        if self
            .last_event
            .map(|last| now - last > 220.)
            .unwrap_or(true)
        {
            self.distance = 0.;
            self.axis = 0;
            self.changed = false;
        }
        self.last_event = Some(now);
        if self.axis == 0 && dx.abs().max(dy.abs()) >= 3. {
            self.axis = if dx.abs() > dy.abs() * 1.2 { 1 } else { -1 };
        }
        if self.axis != 1 {
            return (false, None);
        }
        if self.changed {
            return (true, None);
        }
        self.distance += dx;
        if self.distance.abs() < 96. {
            return (true, None);
        }
        self.changed = true;
        (true, Some(if self.distance > 0. { 1 } else { -1 }))
    }
}
fn drag_target(start: f64, position: f64, dx: f64, dy: f64, max: f64) -> f64 {
    let nearest = position.round();
    let target = if nearest == start.round() && dx.abs() >= 64. && dx.abs() > dy.abs() * 1.2 {
        nearest + if dx < 0. { 1. } else { -1. }
    } else {
        nearest
    };
    target.clamp(0., max)
}
struct State {
    position: f64,
    target: f64,
    dragging: bool,
    moved: f64,
    origin_x: f64,
    origin_y: f64,
    origin_position: f64,
    axis: i8,
    animating: bool,
}
fn layout(cards: &[Element], position: f64) {
    let (spacing, rotation, decay, depth) = if mobile() {
        (150., 14., 0.1, 90.)
    } else {
        (230., 16., 0.12, 110.)
    };
    for (i, card) in cards.iter().enumerate() {
        let offset = i as f64 - position;
        let abs = offset.abs();
        class(card, "threes__card--hidden", abs > 5.);
        if mobile() {
            style(card, "transform", "");
            style(card, "opacity", "1");
            class(card, "threes__card--hidden", false);
            attr(card, "tabindex", "0");
            continue;
        }
        if abs > 5. {
            continue;
        }
        style(
            card,
            "transform",
            &format!(
                "translate(calc(-50% + {}px), -50%) rotateY({}deg) translateZ({}px) scale({})",
                offset * spacing,
                (-offset * rotation).clamp(-55., 55.),
                -abs * depth,
                (1. - abs * decay).max(0.6)
            ),
        );
        style(card, "opacity", &(1. - abs * 0.15).to_string());
        style(card, "z-index", &(1000. - (abs * 10.).round()).to_string());
        attr(
            card,
            "aria-current",
            if position.round() == i as f64 {
                "true"
            } else {
                "false"
            },
        );
        attr(
            card,
            "tabindex",
            if position.round() == i as f64 {
                "0"
            } else {
                "-1"
            },
        );
    }
    if let Some(section) = one(".threes__flow") {
        class(
            &section,
            "threes__flow--at-end",
            position >= cards.len() as f64 - 1.3,
        );
    }
}
type Raf = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
fn animate(state: Rc<RefCell<State>>, cards: Rc<Vec<Element>>) {
    if state.borrow().animating {
        return;
    }
    state.borrow_mut().animating = true;
    let frame: Raf = Rc::new(RefCell::new(None));
    let next = frame.clone();
    *frame.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
        let mut s = state.borrow_mut();
        let max = cards.len() as f64 - 1.;
        let target = s.target.clamp(0., max);
        let snap = target - s.position;
        let done = s.dragging || reduced() || snap.abs() < 0.001;
        if !s.dragging {
            s.position = if done {
                target
            } else {
                s.position + snap * 0.16
            };
        }
        if done {
            s.animating = false;
        }
        layout(&cards, s.position);
        drop(s);
        if done {
            next.borrow_mut().take();
        } else if let Some(f) = next.borrow().as_ref() {
            let _ = window().request_animation_frame(f.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));
    if let Some(f) = frame.borrow().as_ref() {
        let _ = window().request_animation_frame(f.as_ref().unchecked_ref());
    }
}
pub fn init() {
    for el in all("[data-scroll-to]") {
        on(&el, "click", {
            let el = el.clone();
            move |_| {
                if let Some(target) = el.get_attribute("data-scroll-to").and_then(|s| one(&s)) {
                    let opts = web_sys::ScrollIntoViewOptions::new();
                    opts.set_behavior(if reduced() {
                        web_sys::ScrollBehavior::Auto
                    } else {
                        web_sys::ScrollBehavior::Smooth
                    });
                    opts.set_block(web_sys::ScrollLogicalPosition::Start);
                    target.scroll_into_view_with_scroll_into_view_options(&opts);
                }
            }
        });
    }
    let Some(stage) = one("[data-flow-stage]") else {
        return;
    };
    let cards = Rc::new(within(&stage, "[data-index]"));
    if cards.is_empty() {
        return;
    }
    let Some(gallery) = Gallery::new(
        within(&stage, "img"),
        match one("#lightbox") {
            Some(x) => x,
            None => return,
        },
        match one("#lightbox-image") {
            Some(x) => x,
            None => return,
        },
    ) else {
        return;
    };
    let state = Rc::new(RefCell::new(State {
        position: 0.,
        target: 0.,
        dragging: false,
        moved: 0.,
        origin_x: 0.,
        origin_y: 0.,
        origin_position: 0.,
        axis: 0,
        animating: false,
    }));
    for image in within(&stage, "img") {
        attr(&image, "draggable", "false");
    }
    layout(&cards, 0.);
    for (i, card) in cards.iter().enumerate() {
        on(card, "click", {
            let (g, s) = (gallery.clone(), state.clone());
            move |_| {
                if s.borrow().moved < 6. && !s.borrow().dragging {
                    g.show(i);
                }
            }
        });
    }
    on(&window(), "resize", {
        let (s, c) = (state.clone(), cards.clone());
        move |_| layout(&c, s.borrow().position)
    });
    on(&stage, "pointerdown", {
        let s = state.clone();
        move |e| {
            let Ok(p) = e.dyn_into::<PointerEvent>() else {
                return;
            };
            if mobile() || !p.is_primary() || p.button() != 0 {
                return;
            }
            let mut s = s.borrow_mut();
            s.dragging = true;
            s.moved = 0.;
            s.axis = 0;
            s.origin_x = p.client_x() as f64;
            s.origin_y = p.client_y() as f64;
            s.origin_position = s.position;
            s.target = s.position;
        }
    });
    on(&stage, "pointermove", {
        let (s, c, stage) = (state.clone(), cards.clone(), stage.clone());
        move |e| {
            let Ok(p) = e.dyn_into::<PointerEvent>() else {
                return;
            };
            let mut s = s.borrow_mut();
            if !s.dragging {
                return;
            }
            let dx = p.client_x() as f64 - s.origin_x;
            let dy = p.client_y() as f64 - s.origin_y;
            s.moved = s.moved.max(dx.abs()).max(dy.abs());
            if s.axis == 0 && s.moved > 12. {
                s.axis = if dx.abs() > dy.abs() * 1.2 { 1 } else { -1 };
            }
            if s.axis == 1 {
                let _ = stage.set_pointer_capture(p.pointer_id());
                s.position = (s.origin_position - dx / 280.).clamp(0., c.len() as f64 - 1.);
                layout(&c, s.position);
            }
        }
    });
    for event in ["pointerup", "pointercancel"] {
        on(&window(), event, {
            let (s, c, stage) = (state.clone(), cards.clone(), stage.clone());
            move |e| {
                let Ok(p) = e.dyn_into::<PointerEvent>() else {
                    return;
                };
                let mut state = s.borrow_mut();
                if !state.dragging {
                    return;
                }
                state.dragging = false;
                let _ = stage.release_pointer_capture(p.pointer_id());
                state.target = if state.axis == 1 && p.type_() != "pointercancel" {
                    drag_target(
                        state.origin_position,
                        state.position,
                        p.client_x() as f64 - state.origin_x,
                        p.client_y() as f64 - state.origin_y,
                        c.len() as f64 - 1.,
                    )
                } else {
                    state.position.round()
                };
                drop(state);
                animate(s.clone(), c.clone());
            }
        });
    }
    let gesture = RefCell::new(SpreadGesture::default());
    let wheel_state = state.clone();
    let wheel_cards = cards.clone();
    let callback = Closure::wrap(Box::new(move |event: WheelEvent| {
        if mobile() || event.ctrl_key() || editing(event.as_ref()) || wheel_state.borrow().dragging
        {
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
                let mut state = wheel_state.borrow_mut();
                state.target = (state.target.round() + direction as f64)
                    .clamp(0., wheel_cards.len() as f64 - 1.);
                drop(state);
                animate(wheel_state.clone(), wheel_cards.clone());
            }
        }
    }) as Box<dyn FnMut(WheelEvent)>);
    let options = AddEventListenerOptions::new();
    options.set_passive(false);
    let _ = stage.add_event_listener_with_callback_and_add_event_listener_options(
        "wheel",
        callback.as_ref().unchecked_ref(),
        &options,
    );
    callback.forget();
    if let Some(back) = one("[data-flow-back]") {
        on(&back, "click", {
            let (s, c) = (state.clone(), cards.clone());
            move |_| {
                s.borrow_mut().position = 0.;
                s.borrow_mut().target = 0.;
                layout(&c, 0.);
            }
        });
    }
    on(&doc(), "keydown", {
        let (s, c, g) = (state.clone(), cards.clone(), gallery.clone());
        move |e| {
            if mobile()
                || editing(&e)
                || g.open.get()
                || one("#review-panel:focus-within").is_some()
            {
                return;
            }
            let Ok(k) = e.dyn_into::<KeyboardEvent>() else {
                return;
            };
            let mut state = s.borrow_mut();
            match k.key().as_str() {
                "ArrowLeft" | "ArrowRight" => {
                    k.prevent_default();
                    state.target = (state.target.round()
                        + if k.key() == "ArrowLeft" { -1. } else { 1. })
                    .clamp(0., c.len() as f64 - 1.);
                    drop(state);
                    animate(s.clone(), c.clone());
                }
                "Enter" => {
                    if k.target()
                        .and_then(|e| e.dyn_into::<Element>().ok())
                        .map(|e| e.tag_name() == "BUTTON" || e.tag_name() == "A")
                        .unwrap_or(false)
                    {
                        return;
                    }
                    k.prevent_default();
                    g.show(state.position.round() as usize);
                }
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vertical_reading_does_not_turn_spreads() {
        let mut g = SpreadGesture::default();
        assert_eq!(g.push(2., 140., 0.), (false, None));
        assert_eq!(g.push(110., 12., 30.), (false, None));
    }
    #[test]
    fn horizontal_burst_requires_intent_and_turns_once() {
        let mut g = SpreadGesture::default();
        assert_eq!(g.push(30., 1., 0.), (true, None));
        assert_eq!(g.push(40., 1., 20.), (true, None));
        assert_eq!(g.push(30., 1., 40.), (true, Some(1)));
        assert_eq!(g.push(300., 2., 80.), (true, None));
        assert_eq!(g.push(150., 1., 180.), (true, None));
        assert_eq!(g.push(-100., 1., 500.), (true, Some(-1)));
    }
    #[test]
    fn diagonal_gesture_keeps_page_scroll() {
        assert_eq!(SpreadGesture::default().push(100., 90., 0.), (false, None));
    }
    #[test]
    fn drag_settles_predictably_without_inertial_skipping() {
        assert_eq!(drag_target(2., 2.3, -84., 5., 21.), 3.);
        assert_eq!(drag_target(2., 2.1, -28., 5., 21.), 2.);
        assert_eq!(drag_target(2., 1.4, 168., 5., 21.), 1.);
        assert_eq!(drag_target(0., 0., 100., 2., 21.), 0.);
    }
}
