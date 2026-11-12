use crate::dom::*;

fn destination(path: &str, mobile: bool) -> Option<String> {
    let key = path.trim_matches('/');
    let key = if key.is_empty() { "home" } else { key };
    let base = key.strip_suffix("_mobile").unwrap_or(key);
    if ![
        "home",
        "about",
        "contact",
        "quite_off",
        "the_threes",
        "gallery_design",
        "in_praise_of_time",
        "reframed_still",
        "closing_time",
        "masks",
    ]
    .contains(&base)
    {
        return None;
    }
    if mobile && !key.ends_with("_mobile") {
        Some(format!("/{base}_mobile"))
    } else if !mobile && key.ends_with("_mobile") {
        Some(if base == "home" {
            "/".into()
        } else {
            format!("/{base}")
        })
    } else {
        None
    }
}
pub fn redirect() -> bool {
    let location = window().location();
    let Some(to) = destination(&location.pathname().unwrap_or_default(), mobile()) else {
        return false;
    };
    let _ = location.replace(&format!(
        "{to}{}{}",
        location.search().unwrap_or_default(),
        location.hash().unwrap_or_default()
    ));
    true
}
pub fn init() {
    on(&window(), "resize", move |_| {
        redirect();
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_public_views_have_independent_mobile_routes() {
        for page in [
            "home",
            "about",
            "contact",
            "quite_off",
            "the_threes",
            "gallery_design",
            "in_praise_of_time",
            "reframed_still",
            "closing_time",
            "masks",
        ] {
            let desktop = if page == "home" {
                "/".into()
            } else {
                format!("/{page}")
            };
            let phone = format!("/{page}_mobile");
            assert_eq!(destination(&desktop, true), Some(phone.clone()));
            assert_eq!(destination(&phone, false), Some(desktop));
            assert_eq!(destination(&phone, true), None);
        }
    }
    #[test]
    fn review_and_unknown_routes_are_not_redirected() {
        for path in ["/review", "/review/in_praise_of_time", "/404", "/llms.txt"] {
            assert_eq!(destination(path, true), None);
        }
        assert_eq!(
            destination("/in_praise_of_time/", true),
            Some("/in_praise_of_time_mobile".into())
        );
    }
}
