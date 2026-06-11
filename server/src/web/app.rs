use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{FlatRoutes, Route, Router};
use leptos_router::{ParamSegment, StaticSegment};

use crate::web::auth_ui::{AUTH_SCRIPT, AuthModals, HeaderAuth};
use crate::web::home::Home;
use crate::web::modules::{ModuleDetailPage, ModulesPage};
use crate::web::nav::Nav;
use crate::web::roll::Roll;
use crate::web::settings_server::SettingsServerPage;
use crate::web::settings_users::SettingsUsersPage;
use crate::web::{InstanceName, StyleVersion};

pub fn shell(
    options: LeptosOptions,
    instance_name: InstanceName,
    style_version: StyleVersion,
) -> impl IntoView {
    provide_meta_context();
    let title = instance_name.0.clone();
    provide_context(instance_name);
    let stylesheet_href = format!("/assets/style.css?v={}", style_version.0);
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
                        <FlatRoutes fallback=|| view! { <h1>"Not Found"</h1> }>
                            <Route path=StaticSegment("") view=Home/>
                            <Route path=StaticSegment("roll") view=Roll/>
                            <Route path=StaticSegment("modules") view=ModulesPage/>
                            <Route
                                path=(StaticSegment("modules"), ParamSegment("uuid"))
                                view=ModuleDetailPage
                            />
                            <Route
                                path=(StaticSegment("settings"), StaticSegment("users"))
                                view=SettingsUsersPage
                            />
                            <Route
                                path=(StaticSegment("settings"), StaticSegment("server"))
                                view=SettingsServerPage
                            />
                        </FlatRoutes>
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
