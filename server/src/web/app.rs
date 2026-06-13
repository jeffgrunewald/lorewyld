use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{FlatRoutes, Route, Router};
use leptos_router::{ParamSegment, StaticSegment};

use crate::web::auth_ui::{AUTH_SCRIPT, AuthModals, HeaderAuth};
use crate::web::characters::{CharacterNewPage, CharacterSheetPage, CharactersPage};
use crate::web::compendium::{CompendiumCategoryPage, CompendiumEntryPage, CompendiumPage};
use crate::web::home::Home;
use crate::web::lore::{LoreSettingDetailPage, LoreSettingsPage};
use crate::web::modules::{ModuleDetailPage, ModulesPage};
use crate::web::nav::Nav;
use crate::web::roll::Roll;
use crate::web::search::SearchPage;
use crate::web::settings_server::SettingsServerPage;
use crate::web::settings_users::SettingsUsersPage;
use crate::web::{InstanceName, StyleVersion};

pub fn shell(
    options: LeptosOptions,
    instance_name: InstanceName,
    style_version: StyleVersion,
    script_version: StyleVersion,
) -> impl IntoView {
    provide_meta_context();
    let title = instance_name.0.clone();
    provide_context(instance_name);
    let stylesheet_href = format!("/assets/style.css?v={}", style_version.0);
    // Loaded synchronously (no defer): inline page scripts reference
    // window.lwContent at top level.
    let content_js_src = format!("/assets/lw-content.js?v={}", script_version.0);
    let _ = options;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <MetaTags/>
                <Title text=title/>
                <Stylesheet id="lw-style" href=stylesheet_href/>
                <script src=content_js_src></script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="lw-shell">
                <header class="lw-header">
                    <div class="lw-header-brand">
                        <img
                            class="lw-header-wordmark"
                            src="/assets/branding/wordmark.png"
                            alt="Lorewyld"
                        />
                    </div>
                    <HeaderAuth/>
                </header>
                <div class="lw-body">
                    <Nav/>
                    <main class="lw-main">
                        <AppRoutes/>
                    </main>
                </div>
                // Rendered after the page content so page scripts can
                // register their lw-auth-ready listeners before the
                // session probe in AUTH_SCRIPT runs.
                <AuthModals/>
                <script inner_html=AUTH_SCRIPT></script>
            </div>
        </Router>
    }
}

/// The route table, type-erased behind its own component. Routes and
/// their views are `into_any()`-boxed: with this many routes the fully
/// nested view type otherwise mangles into a symbol long enough to
/// trip the macOS linker's name-length assertion.
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <FlatRoutes fallback=|| view! { <h1>"Not Found"</h1> }>
                            <Route path=StaticSegment("") view=|| Home().into_any()/>
                            <Route path=StaticSegment("roll") view=|| Roll().into_any()/>
                            <Route path=StaticSegment("modules") view=|| ModulesPage().into_any()/>
                            <Route
                                path=(StaticSegment("modules"), ParamSegment("uuid"))
                                view=|| ModuleDetailPage().into_any()
                            />
                            <Route
                                path=StaticSegment("compendium")
                                view=|| CompendiumPage().into_any()
                            />
                            <Route
                                path=(StaticSegment("compendium"), ParamSegment("category"))
                                view=|| CompendiumCategoryPage().into_any()
                            />
                            <Route
                                path=(
                                    StaticSegment("compendium"),
                                    ParamSegment("category"),
                                    ParamSegment("uuid"),
                                )
                                view=|| CompendiumEntryPage().into_any()
                            />
                            <Route
                                path=StaticSegment("characters")
                                view=|| CharactersPage().into_any()
                            />
                            // Static "new" registers before the uuid param
                            // route so it can't be captured as a uuid.
                            <Route
                                path=(StaticSegment("characters"), StaticSegment("new"))
                                view=|| CharacterNewPage().into_any()
                            />
                            <Route
                                path=(StaticSegment("characters"), ParamSegment("uuid"))
                                view=|| CharacterSheetPage().into_any()
                            />
                            <Route path=StaticSegment("lore") view=|| LoreSettingsPage().into_any()/>
                            <Route
                                path=(StaticSegment("lore"), ParamSegment("uuid"))
                                view=|| LoreSettingDetailPage().into_any()
                            />
                            <Route path=StaticSegment("search") view=|| SearchPage().into_any()/>
                            <Route
                                path=(StaticSegment("settings"), StaticSegment("users"))
                                view=|| SettingsUsersPage().into_any()
                            />
                            <Route
                                path=(StaticSegment("settings"), StaticSegment("server"))
                                view=|| SettingsServerPage().into_any()
                            />
                        </FlatRoutes>
    }
    .into_any()
}
