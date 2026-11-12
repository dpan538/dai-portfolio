use crate::dom::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlImageElement, KeyboardEvent, PointerEvent};

pub struct Gallery {
    container: Element,
    view: Element,
    sources: Vec<(String, String)>,
    index: Cell<usize>,
    pub open: Cell<bool>,
    navigate: bool,
    opener: RefCell<Option<Element>>,
    old_overflow: RefCell<String>,
    scroll_y: Cell<Option<f64>>,
}
impl Gallery {
    pub fn new(images: Vec<Element>, container: Element, view: Element) -> Option<Rc<Self>> {
        let sources: Vec<_> = images
            .iter()
            .map(|e| {
                let img = e.dyn_ref::<HtmlImageElement>();
                (
                    e.get_attribute("data-lightbox-src")
                        .or_else(|| {
                            img.map(|i| {
                                let current = i.current_src();
                                if current.is_empty() { i.src() } else { current }
                            })
                        })
                        .unwrap_or_default(),
                    e.get_attribute("alt").unwrap_or_default(),
                )
            })
            .collect();
        if sources.is_empty() {
            return None;
        }
        attr(&container, "role", "dialog");
        attr(&container, "aria-modal", "true");
        attr(&container, "aria-label", "Image preview");
        attr(&container, "aria-hidden", "true");
        attr(&container, "tabindex", "-1");
        let navigate = container.get_attribute("data-navigation").as_deref() != Some("single");
        let gallery = Rc::new(Self {
            container,
            view,
            sources,
            index: Cell::new(0),
            open: Cell::new(false),
            navigate,
            opener: RefCell::new(None),
            old_overflow: RefCell::new(String::new()),
            scroll_y: Cell::new(None),
        });
        for (sel, step) in [(".prev,.lb_prev", -1), (".next,.lb_next", 1)] {
            for el in within(&gallery.container, sel) {
                attr(
                    &el,
                    "aria-label",
                    if step < 0 {
                        "Previous image"
                    } else {
                        "Next image"
                    },
                );
                on(&el, "click", {
                    let g = gallery.clone();
                    move |e| {
                        e.stop_propagation();
                        g.step(step);
                    }
                });
            }
        }
        for el in within(
            &gallery.container,
            "[data-close],.lightbox__close,.masks-lightbox__close",
        ) {
            on(&el, "click", {
                let g = gallery.clone();
                move |_| g.close()
            });
        }
        on(&gallery.container, "click", {
            let g = gallery.clone();
            move |e| {
                if e.target().as_ref() == Some(g.container.as_ref()) {
                    g.close();
                }
            }
        });
        on(&doc(), "keydown", {
            let g = gallery.clone();
            move |e| {
                if !g.open.get() {
                    return;
                }
                if let Ok(k) = e.dyn_into::<KeyboardEvent>() {
                    match k.key().as_str() {
                        "Escape" => {
                            k.prevent_default();
                            g.close();
                        }
                        "ArrowLeft" => {
                            k.prevent_default();
                            g.step(-1);
                        }
                        "ArrowRight" => {
                            k.prevent_default();
                            g.step(1);
                        }
                        _ => trap(&k, &g.container),
                    }
                }
            }
        });
        let start = Rc::new(Cell::new(0.));
        on(&gallery.container, "pointerdown", {
            let start = start.clone();
            move |e| {
                if let Ok(p) = e.dyn_into::<PointerEvent>() {
                    start.set(p.client_x() as f64);
                }
            }
        });
        on(&gallery.container, "pointerup", {
            let (g, start) = (gallery.clone(), start.clone());
            move |e| {
                if let Ok(p) = e.dyn_into::<PointerEvent>() {
                    let dx = p.client_x() as f64 - start.get();
                    if dx.abs() > 60. {
                        g.step(if dx > 0. { -1 } else { 1 });
                    }
                }
            }
        });
        let width = Cell::new(
            window()
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.),
        );
        on(&window(), "resize", {
            let g = gallery.clone();
            move |_| {
                let next = window()
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.);
                let changed = width.replace(next) != next;
                // Safari's browser chrome changes viewport height during a tap or scroll.
                // Only a width/breakpoint change invalidates the photographic layout.
                if changed && g.open.get() {
                    g.close();
                }
            }
        });
        Some(gallery)
    }
    pub fn bind(self: &Rc<Self>, images: Vec<Element>) {
        for (i, img) in images.into_iter().enumerate() {
            let opener = img.closest("button").ok().flatten().unwrap_or(img);
            attr(&opener, "tabindex", "0");
            attr(&opener, "aria-haspopup", "dialog");
            if opener.tag_name() != "BUTTON" {
                attr(&opener, "role", "button");
            }
            on(&opener, "click", {
                let g = self.clone();
                move |_| g.show(i)
            });
            if opener.tag_name() != "BUTTON" {
                on(&opener, "keydown", {
                    let g = self.clone();
                    move |e| {
                        if let Ok(k) = e.dyn_into::<KeyboardEvent>()
                            && (k.key() == "Enter" || k.key() == " ")
                        {
                            k.prevent_default();
                            g.show(i);
                        }
                    }
                });
            }
        }
    }
    pub fn show(&self, i: usize) {
        if !self.open.replace(true) {
            if one("body[data-view='mobile']").is_some() {
                let y = window().scroll_y().unwrap_or(0.);
                self.scroll_y.set(Some(y));
                if let Some(body) = doc().body() {
                    style(body.as_ref(), "top", &format!("-{y}px"));
                }
            }
            *self.opener.borrow_mut() = doc().active_element();
            *self.old_overflow.borrow_mut() = doc()
                .document_element()
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                .map(|e| e.style().get_property_value("overflow").unwrap_or_default())
                .unwrap_or_default();
        }
        if let Some(body) = doc().body() {
            class(body.as_ref(), "image-preview-open", true);
        }
        if let Some(theme) = one("meta[name='theme-color']") {
            attr(&theme, "content", "#000000");
        }
        self.index.set(i.min(self.sources.len() - 1));
        self.update();
        for panel in all("#review-panel,.project-info") {
            style(&panel, "display", "none");
        }
        class(&self.container, "hidden", false);
        class(&self.container, "is_open", true);
        attr(&self.container, "aria-hidden", "false");
        if let Some(html) = doc().document_element() {
            class(&html, "image-preview-open", true);
            style(&html, "overflow", "hidden");
        }
        let target = self
            .container
            .query_selector("button")
            .ok()
            .flatten()
            .unwrap_or(self.container.clone());
        focus(&target);
    }
    pub fn step(&self, delta: i32) {
        if !self.navigate {
            return;
        }
        self.index
            .set((self.index.get() as i32 + delta).rem_euclid(self.sources.len() as i32) as usize);
        self.update();
    }
    fn update(&self) {
        let (src, alt) = &self.sources[self.index.get()];
        attr(&self.view, "src", src);
        attr(&self.view, "alt", alt);
    }
    pub fn close(&self) {
        if !self.open.replace(false) {
            return;
        }
        if let Some(body) = doc().body() {
            class(body.as_ref(), "image-preview-open", false);
        }
        if let Some(theme) = one("meta[name='theme-color']") {
            attr(&theme, "content", "#ffffff");
        }
        class(&self.container, "hidden", true);
        class(&self.container, "is_open", false);
        attr(&self.container, "aria-hidden", "true");
        if let Some(html) = doc().document_element() {
            class(&html, "image-preview-open", false);
            style(&html, "overflow", &self.old_overflow.borrow());
        }
        if let Some(y) = self.scroll_y.take() {
            if let Some(body) = doc().body() {
                style(body.as_ref(), "top", "");
            }
            window().scroll_to_with_x_and_y(0., y);
        }
        for panel in all("#review-panel,.project-info") {
            style(&panel, "display", "");
        }
        if let Some(el) = self.opener.borrow().as_ref() {
            focus(el);
        }
    }
}
pub fn setup(images: &str, container: &str, view: &str) -> Option<Rc<Gallery>> {
    let imgs = all(images);
    let g = Gallery::new(imgs.clone(), one(container)?, one(view)?)?;
    g.bind(imgs);
    Some(g)
}
pub fn init() -> Vec<Rc<Gallery>> {
    let mut galleries = Vec::new();
    if let Some(g) = setup(
        "[data-mobile-gallery]",
        "#mobile-lightbox",
        "#mobile-lightbox .lb_view",
    ) {
        galleries.push(g);
    }
    if let Some(g) = setup(".ct__img", "#lightbox", "#lightbox-image") {
        galleries.push(g);
    }
    if let Some(g) = setup(
        ".masks-desktop .image img",
        "#lightbox-desktop",
        "#lightbox-image-desktop",
    ) {
        galleries.push(g);
    }
    if let Some(g) = setup(
        ".masks-mobile .masks-image img",
        ".masks-lightbox",
        ".masks-lightbox__img",
    ) {
        galleries.push(g);
    }
    if let Some(g) = setup(
        ".reframed-desktop .lb_img",
        ".lightbox-desktop",
        ".lightbox-desktop .lb_view",
    ) {
        galleries.push(g);
    }
    if let Some(g) = setup(
        ".reframed-mobile .lb_img",
        "#reframed-lightbox",
        "#reframed-lightbox .lb_view",
    ) {
        galleries.push(g);
    }
    if let Some(g) = setup(
        ".time-project .time-photo",
        "#time-lightbox",
        "#time-lightbox .lb_view",
    ) {
        galleries.push(g);
    }
    for suffix in ["-desktop", ""] {
        if let (Some(strip), Some(hint)) = (
            one(&format!("#reframed-strip{suffix}")),
            one(&format!("#reframed-hint-text{suffix}")),
        ) {
            on(&strip, "mouseover", {
                let hint = hint.clone();
                move |e| {
                    if let Some(frame) = e
                        .target()
                        .and_then(|t| t.dyn_into::<Element>().ok())
                        .and_then(|e| e.closest("[data-hint]").ok().flatten())
                    {
                        hint.set_text_content(frame.get_attribute("data-hint").as_deref());
                    }
                }
            });
            on(&strip, "mouseout", move |_| {
                hint.set_text_content(Some(if mobile() {
                    "Swipe or click to browse"
                } else {
                    "Use ←/→ to scroll · Hover for a note"
                }))
            });
        }
    }
    on(&doc(), "keydown", move |e| {
        if editing(&e) {
            return;
        }
        let Ok(k) = e.dyn_into::<KeyboardEvent>() else {
            return;
        };
        if one(".is_open[role='dialog']").is_some() {
            return;
        }
        if matches!(k.key().as_str(), "ArrowLeft" | "ArrowRight") {
            let id = if mobile() {
                "#reframed-strip"
            } else {
                "#reframed-strip-desktop"
            };
            if let Some(strip) = one(id) {
                k.prevent_default();
                scroll(
                    &strip,
                    if k.key() == "ArrowLeft" { -280. } else { 280. },
                    0.,
                );
            }
        }
    });
    galleries
}
