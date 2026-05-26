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

    let dice_class = move || {
        if pathname.get() == "/roll" {
            "lw-nav-dice-btn lw-nav-dice-btn-active"
        } else {
            "lw-nav-dice-btn"
        }
    };

    view! {
        <nav class="lw-nav">
            <a href="/" class=home_class>"Home"</a>
            <div class="lw-nav-spacer"></div>
            <a href="/roll" class=dice_class aria-label="Dice roller">
                <img src="/assets/dice/d20.png" alt="Dice roller"/>
            </a>
        </nav>
    }
}
