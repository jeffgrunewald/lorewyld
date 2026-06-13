use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// The compendium categories, in `lw-content.js` descriptor order.
/// Paths double as the sub-menu hrefs and active-state prefixes.
const COMPENDIUM_SUBITEMS: [(&str, &str); 11] = [
    ("/compendium/spell", "Spells"),
    ("/compendium/creature", "Creatures"),
    ("/compendium/class", "Classes & subclasses"),
    ("/compendium/species", "Species"),
    ("/compendium/background", "Backgrounds"),
    ("/compendium/feat", "Feats"),
    ("/compendium/item", "Items & gear"),
    ("/compendium/weapon", "Weapons"),
    ("/compendium/armor", "Armor"),
    ("/compendium/condition", "Conditions"),
    ("/compendium/language", "Languages"),
];

/// Sidebar navigation, mirroring the mobile home menu: Characters,
/// Lore (Settings & lore), Search, Compendium, Modules, plus the
/// admin group and the dice roller pinned at the bottom. Every item is
/// always visible — gated pages show a login prompt, matching mobile.
#[component]
pub fn Nav() -> impl IntoView {
    let pathname = use_location().pathname;

    let item_class = move |prefix: &'static str| {
        let current = pathname.get();
        let active = if prefix == "/" {
            current == "/"
        } else {
            current.starts_with(prefix)
        };
        if active {
            "lw-nav-item lw-nav-item-active"
        } else {
            "lw-nav-item"
        }
    };

    let home_class = move || item_class("/");
    let characters_class = move || item_class("/characters");
    let lore_class = move || item_class("/lore");
    let search_class = move || item_class("/search");
    let compendium_class = move || item_class("/compendium");
    let modules_class = move || item_class("/modules");

    let dice_class = move || {
        if pathname.get() == "/roll" {
            "lw-nav-dice-btn lw-nav-dice-btn-active"
        } else {
            "lw-nav-dice-btn"
        }
    };

    let settings_class = move || item_class("/settings");

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

    // Compendium sub-items render only while inside /compendium;
    // prefix matching keeps the category highlighted on entry pages.
    let comp_subitem_class = move |path: &'static str| {
        let current = pathname.get();
        if !current.starts_with("/compendium") {
            "lw-nav-subitem lw-hidden"
        } else if current == path || current.starts_with(&format!("{path}/")) {
            "lw-nav-item lw-nav-subitem lw-nav-item-active"
        } else {
            "lw-nav-item lw-nav-subitem"
        }
    };

    view! {
        <nav class="lw-nav">
            <a href="/" class=home_class>"Home"</a>
            <a href="/characters" class=characters_class>"Characters"</a>
            <a href="/lore" class=lore_class>"Settings & lore"</a>
            <a href="/search" class=search_class>"Search"</a>
            <a href="/compendium" class=compendium_class>"Compendium"</a>
            {COMPENDIUM_SUBITEMS
                .into_iter()
                .map(|(path, label)| {
                    let class = move || comp_subitem_class(path);
                    view! { <a href=path class=class>{label}</a> }
                })
                .collect_view()}
            <a href="/modules" class=modules_class>"Modules"</a>
            // Hidden until the auth probe confirms an admin session
            // (see AUTH_SCRIPT); the admin API is the real gate.
            <div id="lw-nav-settings" hidden=true>
                <a href="/settings/users" class=settings_class>"Admin"</a>
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
