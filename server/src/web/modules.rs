use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// List of installed content modules. Skeleton rendered server-side;
/// JS populates from `/api/server-info` on load.
#[component]
pub fn ModulesPage() -> impl IntoView {
    view! {
        <section class="lw-modules">
            <h1 class="lw-modules-title">"Content modules"</h1>
            <p class="lw-modules-help">
                "Published content packs on this server. Tap any module to browse its lore notes."
            </p>
            <ul id="lw-modules-list" class="lw-modules-list">
                <li class="lw-modules-loading">"Loading modules…"</li>
            </ul>
            <script inner_html=MODULES_LIST_SCRIPT></script>
        </section>
    }
}

/// Detail page for a single module — shows metadata + all its
/// module-scope lore notes. URL: `/modules/:uuid`.
#[component]
pub fn ModuleDetailPage() -> impl IntoView {
    let params = use_params_map();
    let uuid = move || params.read().get("uuid").unwrap_or_default();
    let initial_uuid = uuid();

    view! {
        <section class="lw-module-detail" data-module-uuid=initial_uuid>
            <header class="lw-module-detail-header">
                <h1 id="lw-module-name" class="lw-module-detail-title">"Loading…"</h1>
                <div id="lw-module-meta" class="lw-module-detail-meta"></div>
            </header>
            <div id="lw-module-description" class="lw-module-detail-description"></div>
            <h2 class="lw-module-detail-section">"Lore notes"</h2>
            <div id="lw-module-notes" class="lw-module-notes">
                <p class="lw-modules-loading">"Loading notes…"</p>
            </div>
            <script inner_html=MODULE_DETAIL_SCRIPT></script>
        </section>
    }
}

// Inline JS uses pure-DOM construction (createElement / textContent)
// throughout — no innerHTML assignment with user-derived content, so
// untrusted markdown / tag names can't escape into the DOM.
const MODULES_LIST_SCRIPT: &str = r#"
(function () {
    const list = document.getElementById('lw-modules-list');
    fetch('/api/server-info')
        .then(r => {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        })
        .then(data => {
            const modules = data.modules || [];
            list.replaceChildren();
            if (modules.length === 0) {
                const empty = document.createElement('li');
                empty.className = 'lw-modules-empty';
                empty.textContent = 'No modules published yet. Use the mobile app to author a setting and promote it to a module.';
                list.appendChild(empty);
                return;
            }
            for (const m of modules) {
                list.appendChild(buildModuleCard(m));
            }
        })
        .catch(err => {
            list.replaceChildren();
            const errEl = document.createElement('li');
            errEl.className = 'lw-modules-error';
            errEl.textContent = 'Failed to load: ' + String(err);
            list.appendChild(errEl);
        });

    function buildModuleCard(m) {
        const card = document.createElement('li');
        card.className = 'lw-module-card';

        const link = document.createElement('a');
        link.className = 'lw-module-card-link';
        link.href = '/modules/' + encodeURIComponent(m.uuid);

        const name = document.createElement('span');
        name.className = 'lw-module-card-name';
        name.textContent = m.name;
        link.appendChild(name);

        const version = document.createElement('span');
        version.className = 'lw-module-card-version';
        version.textContent = 'v' + (m.version_string || '');
        link.appendChild(version);

        card.appendChild(link);

        const meta = document.createElement('div');
        meta.className = 'lw-module-card-meta';
        const license = document.createElement('span');
        license.className = 'lw-module-card-license';
        license.textContent = m.license || '';
        meta.appendChild(license);
        const authors = document.createElement('span');
        authors.className = 'lw-module-card-authors';
        authors.textContent = 'by ' + ((m.authors || []).join(', ') || 'unknown');
        meta.appendChild(authors);
        card.appendChild(meta);

        if (m.description) {
            const desc = document.createElement('p');
            desc.className = 'lw-module-card-desc';
            desc.textContent = m.description;
            card.appendChild(desc);
        }
        return card;
    }
})();
"#;

const MODULE_DETAIL_SCRIPT: &str = r#"
(function () {
    const root = document.querySelector('[data-module-uuid]');
    const uuid = root && root.dataset.moduleUuid;
    const nameEl = document.getElementById('lw-module-name');
    const metaEl = document.getElementById('lw-module-meta');
    const descEl = document.getElementById('lw-module-description');
    const notesEl = document.getElementById('lw-module-notes');

    if (!uuid) {
        nameEl.textContent = 'Missing module UUID in URL';
        return;
    }

    fetch('/api/modules/' + encodeURIComponent(uuid))
        .then(r => {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        })
        .then(data => {
            const m = data.module;
            const notes = data.notes || [];
            document.title = m.name + ' — Lorewyld';
            nameEl.textContent = m.name;

            metaEl.replaceChildren();
            metaEl.appendChild(buildMetaSpan('lw-module-detail-version', 'v' + m.version_string));
            metaEl.appendChild(buildMetaSpan('lw-module-detail-license', m.license));
            metaEl.appendChild(buildMetaSpan('lw-module-detail-authors',
                'by ' + ((m.authors || []).join(', ') || 'unknown')));

            descEl.replaceChildren();
            if (m.description) {
                const p = document.createElement('p');
                p.textContent = m.description;
                descEl.appendChild(p);
            }

            notesEl.replaceChildren();
            if (notes.length === 0) {
                const empty = document.createElement('p');
                empty.className = 'lw-modules-empty';
                empty.textContent = 'No lore notes in this module.';
                notesEl.appendChild(empty);
                return;
            }
            for (const n of notes) {
                notesEl.appendChild(buildNote(n));
            }
        })
        .catch(err => {
            nameEl.textContent = 'Module not found';
            notesEl.replaceChildren();
            const errEl = document.createElement('p');
            errEl.className = 'lw-modules-error';
            errEl.textContent = 'Failed to load: ' + String(err);
            notesEl.appendChild(errEl);
        });

    function buildMetaSpan(cls, text) {
        const el = document.createElement('span');
        el.className = cls;
        el.textContent = text || '';
        return el;
    }

    function buildNote(n) {
        const article = document.createElement('article');
        article.className = 'lw-lore-note';

        const title = document.createElement('h3');
        title.className = 'lw-lore-note-title';
        title.textContent = n.note.title;
        article.appendChild(title);

        if ((n.tags || []).length > 0) {
            const tagsRow = document.createElement('div');
            tagsRow.className = 'lw-lore-note-tags';
            for (const t of n.tags) {
                const chip = document.createElement('span');
                chip.className = 'lw-tag-chip';
                chip.textContent = t.display_name;
                tagsRow.appendChild(chip);
            }
            article.appendChild(tagsRow);
        }

        const body = document.createElement('pre');
        body.className = 'lw-lore-note-body';
        body.textContent = n.note.body_markdown || '';
        article.appendChild(body);

        return article;
    }
})();
"#;
