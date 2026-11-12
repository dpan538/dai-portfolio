use crate::dom::*;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent};

pub fn init() {
    let (Some(details), Some(summary)) = (one("#project-info"), one("#project-info>summary"))
    else {
        return;
    };
    let pinned = Rc::new(Cell::new(false));
    let hovered = Rc::new(Cell::new(false));
    let ticket = Rc::new(Cell::new(0u64));
    on(&summary, "click", {
        let (details, pinned) = (details.clone(), pinned.clone());
        move |e| {
            e.prevent_default();
            let show = !details.has_attribute("open");
            pinned.set(show);
            if show {
                attr(&details, "open", "");
            } else {
                let _ = details.remove_attribute("open");
            }
        }
    });
    let close: Rc<dyn Fn()> = Rc::new({
        let (details, summary, pinned) = (details.clone(), summary.clone(), pinned.clone());
        move || {
            pinned.set(false);
            let _ = details.remove_attribute("open");
            focus(&summary);
        }
    });
    if let Some(btn) = one(".project-info-close") {
        on(&btn, "click", {
            let close = close.clone();
            move |_| close()
        });
    }
    on(&details, "keydown", {
        let close = close.clone();
        move |e| {
            if let Ok(k) = e.dyn_into::<KeyboardEvent>()
                && k.key() == "Escape"
            {
                k.prevent_default();
                close();
            }
        }
    });
    on(&details, "pointerenter", {
        let (details, hovered, ticket) = (details.clone(), hovered.clone(), ticket.clone());
        move |_| {
            let fine = window()
                .match_media("(hover:hover) and (pointer:fine)")
                .ok()
                .flatten()
                .map(|m| m.matches())
                .unwrap_or(false);
            if mobile() || !fine {
                return;
            }
            hovered.set(true);
            ticket.set(ticket.get().wrapping_add(1));
            let token = ticket.get();
            delay(160, {
                let (details, hovered, ticket) = (details.clone(), hovered.clone(), ticket.clone());
                move || {
                    if hovered.get() && ticket.get() == token {
                        attr(&details, "open", "");
                    }
                }
            });
        }
    });
    on(&details, "pointerleave", {
        let (details, hovered, ticket, pinned) = (
            details.clone(),
            hovered.clone(),
            ticket.clone(),
            pinned.clone(),
        );
        move |_| {
            hovered.set(false);
            ticket.set(ticket.get().wrapping_add(1));
            let token = ticket.get();
            delay(220, {
                let (details, hovered, ticket, pinned) = (
                    details.clone(),
                    hovered.clone(),
                    ticket.clone(),
                    pinned.clone(),
                );
                move || {
                    let focused = doc()
                        .active_element()
                        .map(|a| details.contains(Some(&a)))
                        .unwrap_or(false);
                    if !hovered.get() && !pinned.get() && !focused && ticket.get() == token {
                        let _ = details.remove_attribute("open");
                    }
                }
            });
        }
    });
    on(&doc(), "pointerdown", move |e| {
        let inside = e
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .map(|e| details.contains(Some(&e)))
            .unwrap_or(false);
        if !inside {
            pinned.set(false);
            let _ = details.remove_attribute("open");
        }
    });
}
