use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::components::{FlatRoutes, Route, Router};
use leptos_router::StaticSegment;

use crate::web::home::Home;
use crate::web::nav::Nav;
use crate::web::roll::Roll;
use crate::web::InstanceName;

pub fn shell(options: LeptosOptions, instance_name: InstanceName) -> impl IntoView {
    provide_meta_context();
    let title = instance_name.0.clone();
    provide_context(instance_name);
    let _ = options;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <MetaTags/>
                <Title text=title/>
                <Stylesheet id="lw-style" href="/assets/style.css"/>
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
                </header>
                <div class="lw-body">
                    <Nav/>
                    <main class="lw-main">
                        <FlatRoutes fallback=|| view! { <h1>"Not Found"</h1> }>
                            <Route path=StaticSegment("") view=Home/>
                            <Route path=StaticSegment("roll") view=Roll/>
                        </FlatRoutes>
                    </main>
                </div>
            </div>
        </Router>
    }
}
