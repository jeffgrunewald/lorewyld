use leptos::prelude::*;

use crate::web::InstanceName;

#[component]
pub fn Home() -> impl IntoView {
    let name = use_context::<InstanceName>()
        .map(|n| n.0)
        .unwrap_or_else(|| "Lorewyld".to_string());

    view! {
        <div class="lw-home">
            <h1 class="lw-home-title">{format!("Welcome to {name}")}</h1>
        </div>
    }
}
