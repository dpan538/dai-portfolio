use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Document, Element, Event, EventTarget, HtmlElement, Window};

pub fn window() -> Window {
    web_sys::window().expect("browser window")
}
pub fn doc() -> Document {
    window().document().expect("document")
}
pub fn one(s: &str) -> Option<Element> {
    doc().query_selector(s).ok().flatten()
}
pub fn all(s: &str) -> Vec<Element> {
    within(&doc().document_element().unwrap(), s)
}
pub fn within(root: &Element, s: &str) -> Vec<Element> {
    root.query_selector_all(s)
        .ok()
        .map(|nodes| {
            (0..nodes.length())
                .filter_map(|i| nodes.item(i)?.dyn_into().ok())
                .collect()
        })
        .unwrap_or_default()
}
pub fn attr(el: &Element, name: &str, value: &str) {
    let _ = el.set_attribute(name, value);
}
pub fn style(el: &Element, name: &str, value: &str) {
    if let Some(el) = el.dyn_ref::<HtmlElement>() {
        let _ = el.style().set_property(name, value);
    }
}
pub fn class(el: &Element, name: &str, on: bool) {
    let _ = el.class_list().toggle_with_force(name, on);
}
pub fn focus(el: &Element) {
    if let Some(el) = el.dyn_ref::<HtmlElement>() {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        let _ = el.focus_with_options(&options);
    }
}
// Safari can retain :focus-visible after a programmatic focus following a tap.
// Track the initiating input before component handlers move focus into a dialog.
pub fn init_input_modality() {
    let Some(root) = doc().document_element() else {
        return;
    };
    attr(&root, "data-input", "pointer");
    for event in ["pointerdown", "keydown"] {
        let root = root.clone();
        let cb = Closure::wrap(Box::new(move |e: Event| {
            if event == "keydown" {
                let Some(key) = e.dyn_ref::<web_sys::KeyboardEvent>() else {
                    return;
                };
                if key.meta_key()
                    || key.ctrl_key()
                    || key.alt_key()
                    || matches!(key.key().as_str(), "Shift" | "Control" | "Alt" | "Meta")
                {
                    return;
                }
            }
            attr(
                &root,
                "data-input",
                if event == "keydown" {
                    "keyboard"
                } else {
                    "pointer"
                },
            );
        }) as Box<dyn FnMut(Event)>);
        let options = web_sys::AddEventListenerOptions::new();
        options.set_capture(true);
        let _ = doc().add_event_listener_with_callback_and_add_event_listener_options(
            event,
            cb.as_ref().unchecked_ref(),
            &options,
        );
        cb.forget();
    }
}
pub fn on<T: AsRef<EventTarget> + ?Sized>(el: &T, event: &str, f: impl FnMut(Event) + 'static) {
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut(Event)>);
    let _ = el
        .as_ref()
        .add_event_listener_with_callback(event, cb.as_ref().unchecked_ref());
    cb.forget();
}
pub fn delay(ms: i32, f: impl FnOnce() + 'static) {
    let cb = Closure::once_into_js(f);
    let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), ms);
}
pub fn mobile() -> bool {
    // iOS can expand innerWidth to include overflowing intrinsic image widths.
    // Use the same viewport as the CSS breakpoint, not the overflow area.
    window()
        .match_media("(max-width: 768px)")
        .ok()
        .flatten()
        .map(|query| query.matches())
        .unwrap_or_else(|| {
            doc()
                .document_element()
                .is_some_and(|root| (1..=768).contains(&root.client_width()))
        })
}
pub fn reduced() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}
pub fn editing(e: &Event) -> bool {
    e.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|e| {
            e.closest("input,textarea,select,[contenteditable='true']")
                .ok()
                .flatten()
        })
        .is_some()
}
#[cfg(feature = "review")]
pub fn query(name: &str) -> Option<String> {
    window()
        .location()
        .search()
        .ok()?
        .trim_start_matches('?')
        .split('&')
        .find_map(|part| {
            let (k, v) = part.split_once('=')?;
            (k == name).then(|| v.to_owned())
        })
}
pub fn trap(event: &web_sys::KeyboardEvent, container: &Element) {
    if event.key() != "Tab" {
        return;
    }
    let targets: Vec<_> = within(container, "button,a[href],[tabindex='0']")
        .into_iter()
        .filter(|e| e.get_bounding_client_rect().width() > 0.)
        .collect();
    let Some(first) = targets.first() else {
        return;
    };
    let last = targets.last().unwrap();
    let active = doc().active_element();
    if event.shift_key() && (active.as_ref() == Some(first) || active.as_ref() == Some(container)) {
        event.prevent_default();
        focus(last);
    } else if !event.shift_key() && active.as_ref() == Some(last) {
        event.prevent_default();
        focus(first);
    }
}
pub fn scroll(el: &Element, left: f64, top: f64) {
    let options = web_sys::ScrollToOptions::new();
    options.set_left(left);
    options.set_top(top);
    options.set_behavior(if reduced() {
        web_sys::ScrollBehavior::Auto
    } else {
        web_sys::ScrollBehavior::Smooth
    });
    el.scroll_by_with_scroll_to_options(&options);
}
