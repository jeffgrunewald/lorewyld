use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Nav() -> impl IntoView {
    let pathname = use_location().pathname;

    let home_class = move || {
        if pathname.get() == "/" {
            "lw-nav-item lw-nav-item-active"
        } else {
            "lw-nav-item"
        }
    };

    let modules_class = move || {
        if pathname.get().starts_with("/modules") {
            "lw-nav-item lw-nav-item-active"
        } else {
            "lw-nav-item"
        }
    };

    let dice_class = move || {
        if pathname.get() == "/roll" {
            "lw-nav-dice-btn lw-nav-dice-btn-active"
        } else {
            "lw-nav-dice-btn"
        }
    };

    let settings_class = move || {
        if pathname.get().starts_with("/settings") {
            "lw-nav-item lw-nav-item-active"
        } else {
            "lw-nav-item"
        }
    };

    // Sub-items render only while inside /settings; the active page gets
    // the same highlight treatment as top-level items.
    let subitem_class = move |path: &'static str| {
        let current = pathname.get();
        if !current.starts_with("/settings") {
            "lw-nav-subitem lw-hidden"
        } else if current == path {
            "lw-nav-item lw-nav-subitem lw-nav-item-active"
        } else {
            "lw-nav-item lw-nav-subitem"
        }
    };
    let users_class = move || subitem_class("/settings/users");
    let server_class = move || subitem_class("/settings/server");

    view! {
        <nav class="lw-nav">
            <a href="/" class=home_class>"Home"</a>
            <a href="/modules" class=modules_class>"Modules"</a>
            // Hidden until the auth probe confirms an admin session
            // (see AUTH_SCRIPT); the admin API is the real gate.
            <div id="lw-nav-settings" hidden=true>
                <a href="/settings/users" class=settings_class>"Settings"</a>
                <a href="/settings/users" class=users_class>"Users"</a>
                <a href="/settings/server" class=server_class>"Server"</a>
            </div>
            <div class="lw-nav-spacer"></div>
            <a href="/roll" class=dice_class aria-label="Dice roller">
                <img src="/assets/dice/d20.png" alt="Dice roller"/>
            </a>
        </nav>
    }
}
