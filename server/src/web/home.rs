use leptos::prelude::*;

use crate::web::InstanceName;

/// Welcome screen. Logged-in users additionally get a "Recent Activity"
/// panel of the newest compendium entries, linked to their detail
/// pages; the content API is login-gated, so the panel stays hidden
/// for anonymous visitors.
#[component]
pub fn Home() -> impl IntoView {
    let name = use_context::<InstanceName>()
        .map(|n| n.0)
        .unwrap_or_else(|| "Lorewyld".to_string());

    view! {
        <div class="lw-home">
            <h1 class="lw-home-title">{format!("Welcome to {name}")}</h1>
            <section id="lw-home-recent" class="lw-home-recent" hidden=true>
                <h2 class="lw-group-header">"Recent Activity"</h2>
                <ul id="lw-home-recent-list" class="lw-list"></ul>
            </section>
            <script inner_html=HOME_RECENT_SCRIPT></script>
        </div>
    }
}

const HOME_RECENT_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const section = document.getElementById('lw-home-recent');
    const list = document.getElementById('lw-home-recent-list');

    function onAuth(me) {
        if (!me) return;
        C.fetchJson('/api/content/recent?limit=10').then(function (data) {
            const items = data.items || [];
            if (items.length === 0) return;
            list.replaceChildren();
            for (const item of items) {
                list.appendChild(buildRow(item));
            }
            section.hidden = false;
        }).catch(function () { /* non-fatal: the panel just stays hidden */ });
    }
    if (window.lw && window.lw.ready) onAuth(window.lw.me);
    else document.addEventListener('lw-auth-ready', function (e) { onAuth(e.detail); });

    function buildRow(item) {
        const category = C.categoryFor(item.category);
        const li = C.el('li', 'lw-list-item');
        const a = C.el('a', 'lw-list-item-link');
        a.href = '/compendium/' + encodeURIComponent(item.category) +
            '/' + encodeURIComponent(item.uuid);
        const text = C.el('div', 'lw-list-item-text');
        text.appendChild(C.el('div', 'lw-list-item-title', item.name));
        const parts = [category ? category.label : item.category];
        if (item.module_name) parts.push(item.module_name);
        text.appendChild(C.el('div', 'lw-list-item-subtitle', parts.join(' · ')));
        a.appendChild(text);
        if (item.created_at) {
            const date = new Date(item.created_at);
            if (!isNaN(date)) {
                a.appendChild(C.el('div', 'lw-list-item-meta', date.toLocaleDateString()));
            }
        }
        li.appendChild(a);
        return li;
    }
})();
"#;
