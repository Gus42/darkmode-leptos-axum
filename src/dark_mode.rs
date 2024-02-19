use leptos::*;
use leptos_meta::{Meta};
use leptos_router::{ActionForm};
use leptos::logging::log;

/// Toggle the value of darkmode in the cookie
#[server(ToggleDarkMode, "/api")]
pub async fn toggle_dark_mode(prefers_dark: bool) -> Result<bool, ServerFnError> {
    log!("Inside server ToggleDarkMode, pref_dark: {}", prefers_dark);
    use axum::http::header::{HeaderMap, /* HeaderName, */ HeaderValue, SET_COOKIE};
    use leptos_axum::{ResponseOptions, ResponseParts};

    let response: ResponseOptions = use_context::<leptos_axum::ResponseOptions>()
        .expect("to have leptos_axum::ResponsaeOptions provided");

    let mut response_parts: ResponseParts = ResponseParts::default();
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&format!("darkmode={prefers_dark}; Path=/"))
            .expect("to create header value")
    );
    response_parts.headers = headers;

    std::thread::sleep(std::time::Duration::from_secs(1));

    response.overwrite(response_parts); //.await ???
    Ok(prefers_dark)
}

// this macro means something like: this is only for the client
#[cfg(not(feature = "ssr"))]
fn initial_prefers_dark(prefers_dark_default: bool) -> bool {
    log!("Inside initial pref not ssr");
    use wasm_bindgen::JsCast;

    let doc = document().unchecked_into::<web_sys::HtmlDocument>();
    let cookie = doc.cookie().unwrap_or_default();
    if cookie.contains("darkmode") {
        cookie.contains("darkmode=true")
    } else {
        prefers_dark_default == true
    }
}

// This function is diffent from the one in the tutorial video by gbj
// From the leptos 0.6.0-rc1 there is no need anymore to use use_context::<RequestParts>()
// In fact RequestParts does not exists anymore inside leptos_axum.
// We just use http::request::Parts whose now implements Clone.
#[cfg(feature = "ssr")]
fn initial_prefers_dark(prefers_dark_default: bool) -> bool {
    log!("Inside initial pref ssr");
    use axum_extra::extract::cookie::CookieJar;
    use http::request::Parts; 
    use_context::<Parts>()
        .and_then(|req| {
            let cookies = CookieJar::from_headers(&req.headers);
            cookies.get("darkmode").and_then(|v| match v.value() {
                "true" => Some(true),
                "false" => Some(false),
                _ => Some(prefers_dark_default),
            })
        })
        .unwrap_or(false)
}

#[component]
pub fn DarkModeToggle(
    /// Whether the component should initially prefer dark mode
    #[prop(optional)]
    prefers_dark_default: bool
) -> impl IntoView {
    log!("Inside DarkModeToggle");

    let initial = initial_prefers_dark(prefers_dark_default);

    let toggle_dark_mode_action = create_server_action::<ToggleDarkMode>();

    // input is `Some(value)` when pending, and `None` if not pending
    let input = toggle_dark_mode_action.input();
    // value contains most recently-returned value
    let value = toggle_dark_mode_action.value();
    
    let prefers_dark = move || {
        log!("Inside prefers_dark");
        match (input(), value()) {
            // if there's some current input, use that optimistically
            (Some(submission), _) => submission.prefers_dark,
            // otherwise, if there was a previous value confirmed by server, use that
            (_, Some(Ok(value))) => value,
            // otherwise, use the initial value
            _ => initial,
        }
    };

    let color_scheme = move || {
        log!("Inside color_scheme()");
        if prefers_dark()
        {
            "dark".to_string()
        } else {
            "light".to_string()
        }
    };

    view! {
        <Meta name="color-scheme" content=color_scheme/>
        <ActionForm action=toggle_dark_mode_action>
            <input 
                type="hidden" 
                name="prefers_dark" 
                value=move || {
                    log!("Inside the hidden input");
                    (!prefers_dark()).to_string()
                }
            />
            <input 
                type="submit" 
                value=move || {
                    log!("Inside the submit");
                    if prefers_dark() {
                        "Switch to Light Mode"
                    } else {
                        "Switch to Dark Mode"
                    }
                }
            />
        </ActionForm>
    }
}
