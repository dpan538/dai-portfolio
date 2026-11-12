use crate::dom::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

pub fn init() {
    let (Some(btn), Some(overlay), Some(backdrop)) = (
        one("#mobile_menu_btn"),
        one("#mobile_overlay"),
        one("#mobile_overlay_backdrop"),
    ) else {
        return;
    };
    attr(&overlay, "inert", "");
    let open = Rc::new(Cell::new(false));
    let scroll = Rc::new(Cell::new(0.));
    let return_focus = Rc::new(RefCell::new(btn.clone()));
    let toggle: Rc<dyn Fn(bool)> = Rc::new({
        let (btn, overlay, open, scroll) =
            (btn.clone(), overlay.clone(), open.clone(), scroll.clone());
        let return_focus = return_focus.clone();
        move |show| {
            if open.replace(show) == show {
                return;
            }
            attr(&overlay, "data-open", if show { "true" } else { "false" });
            attr(&overlay, "aria-hidden", if show { "false" } else { "true" });
            attr(&btn, "aria-expanded", if show { "true" } else { "false" });
            for trigger in all("[data-mobile-menu-open]") {
                attr(
                    &trigger,
                    "aria-expanded",
                    if show { "true" } else { "false" },
                );
            }
            attr(
                &btn,
                "aria-label",
                if show { "Close menu" } else { "Open menu" },
            );
            for panel in all("#review-panel,.project-info") {
                style(&panel, "display", if show { "none" } else { "" });
            }
            if show {
                let _ = overlay.remove_attribute("inert");
                scroll.set(window().scroll_y().unwrap_or(0.));
                if let Some(body) = doc().body() {
                    class(body.as_ref(), "mobile-overlay-open", true);
                    style(body.as_ref(), "position", "fixed");
                    style(body.as_ref(), "top", &format!("-{}px", scroll.get()));
                    style(body.as_ref(), "width", "100%");
                    style(body.as_ref(), "overflow", "hidden");
                }
                if let Some(shell) = one(".shell") {
                    attr(&shell, "inert", "");
                }
                if let Some(link) = overlay.query_selector("a").ok().flatten() {
                    focus(&link);
                }
            } else {
                attr(&overlay, "inert", "");
                if let Some(shell) = one(".shell") {
                    let _ = shell.remove_attribute("inert");
                }
                if let Some(body) = doc().body() {
                    class(body.as_ref(), "mobile-overlay-open", false);
                    for prop in ["position", "top", "width", "overflow"] {
                        style(body.as_ref(), prop, "");
                    }
                }
                window().scroll_to_with_x_and_y(0., scroll.get());
                focus(&return_focus.borrow());
            }
        }
    });
    for trigger in all("#mobile_menu_btn,[data-mobile-menu-open]") {
        on(&trigger.clone(), "click", {
            let (toggle, open, return_focus) = (toggle.clone(), open.clone(), return_focus.clone());
            move |_| {
                if !open.get() {
                    *return_focus.borrow_mut() = trigger.clone();
                }
                toggle(!open.get());
            }
        });
    }
    on(&backdrop, "click", {
        let toggle = toggle.clone();
        move |_| toggle(false)
    });
    on(&overlay, "click", {
        let (toggle, overlay) = (toggle.clone(), overlay.clone());
        move |e| {
            if e.target().as_ref() == Some(overlay.as_ref()) {
                toggle(false);
            }
        }
    });
    on(&doc(), "keydown", {
        let (toggle, open, overlay) = (toggle.clone(), open.clone(), overlay.clone());
        move |e| {
            if !open.get() {
                return;
            }
            if let Ok(k) = e.dyn_into::<KeyboardEvent>() {
                if k.key() == "Escape" {
                    toggle(false);
                } else {
                    trap(&k, &overlay);
                }
            }
        }
    });
    on(&window(), "resize", move |_| {
        if !mobile() {
            toggle(false);
        }
    });
}
