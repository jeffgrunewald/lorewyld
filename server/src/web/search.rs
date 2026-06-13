//! Lore-note search: free text + tag chips + scope facet, against the
//! existing `POST /api/search` FTS endpoint. Mirrors the mobile search
//! screen; results open the shared note editor in place.

use leptos::prelude::*;

use crate::web::auth_ui::LoginRequired;
use crate::web::breadcrumbs::{Breadcrumbs, Crumb};
use crate::web::lore::NOTE_EDITOR_SCRIPT;

/// `/search`.
#[component]
pub fn SearchPage() -> impl IntoView {
    view! {
        <section class="lw-page">
            <Breadcrumbs trail=vec![Crumb::link("Home", "/"), Crumb::here("Search")]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 class="lw-page-title">"Search lore"</h1>
                </header>
                <div class="lw-search-row">
                    <input id="lw-search-q" class="lw-input" type="search" placeholder="Search lore notes…"/>
                    <button id="lw-search-go" class="lw-btn lw-btn-filled" type="button">"Search"</button>
                </div>
                <div class="lw-field">
                    <span>"Filter by tag"</span>
                    <div id="lw-search-tags" class="lw-chip-row"></div>
                    <input id="lw-search-tag-input" class="lw-input" type="text"
                        placeholder="Add a tag filter and press Enter"/>
                </div>
                <div id="lw-search-scopes" class="lw-chip-row"></div>
                <ul id="lw-search-results" class="lw-list"></ul>
                <p id="lw-search-status" class="lw-picker-status"></p>
            </div>
            <script inner_html=NOTE_EDITOR_SCRIPT></script>
            <script inner_html=SEARCH_SCRIPT></script>
        </section>
    }
}

const SEARCH_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const q = document.getElementById('lw-search-q');
    const go = document.getElementById('lw-search-go');
    const tagRow = document.getElementById('lw-search-tags');
    const tagInput = document.getElementById('lw-search-tag-input');
    const scopeRow = document.getElementById('lw-search-scopes');
    const results = document.getElementById('lw-search-results');
    const status = document.getElementById('lw-search-status');

    const SCOPES = [
        [null, 'All scopes'],
        ['setting', 'Settings'],
        ['module', 'Modules'],
    ];
    const tags = [];
    let scopeKind = null;

    C.requireAuth(function () {
        renderScopes();
        go.addEventListener('click', run);
        q.addEventListener('keydown', function (e) {
            if (e.key === 'Enter') run();
        });
        tagInput.addEventListener('keydown', function (e) {
            if (e.key !== 'Enter' && e.key !== ',') return;
            e.preventDefault();
            const slug = tagInput.value.trim().toLowerCase().replace(/\s+/g, '-').replace(/,/g, '');
            if (slug && !tags.includes(slug)) {
                tags.push(slug);
                renderTags();
                run();
            }
            tagInput.value = '';
        });
    });

    function renderTags() {
        tagRow.replaceChildren();
        for (const slug of tags) {
            const chip = C.el('span', 'lw-chip lw-chip-selected', '#' + slug);
            const x = C.el('button', 'lw-chip-remove', '✕');
            x.type = 'button';
            x.setAttribute('aria-label', 'Remove tag ' + slug);
            x.addEventListener('click', function () {
                tags.splice(tags.indexOf(slug), 1);
                renderTags();
                run();
            });
            chip.appendChild(x);
            tagRow.appendChild(chip);
        }
    }

    function renderScopes() {
        scopeRow.replaceChildren();
        for (const scope of SCOPES) {
            const chip = C.el('button',
                'lw-chip' + (scopeKind === scope[0] ? ' lw-chip-selected' : ''), scope[1]);
            chip.type = 'button';
            chip.addEventListener('click', function () {
                scopeKind = scope[0];
                renderScopes();
                run();
            });
            scopeRow.appendChild(chip);
        }
    }

    function run() {
        const query = q.value.trim();
        if (!query && tags.length === 0 && !scopeKind) {
            results.replaceChildren();
            status.textContent = '';
            return;
        }
        status.textContent = 'Searching…';
        const body = { tag_slugs: tags, limit: 50 };
        if (query) body.q = query;
        if (scopeKind) body.scope_kind = scopeKind;
        fetch('/api/search', {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
            body: JSON.stringify(body),
        }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        }).then(function (data) {
            const notes = data.notes || [];
            results.replaceChildren();
            status.textContent = notes.length === 0 ? 'No matches.' : '';
            for (const entry of notes) {
                results.appendChild(buildResult(entry));
            }
        }).catch(function (err) {
            status.textContent = 'Search failed: ' + err;
        });
    }

    function buildResult(entry) {
        const note = entry.note;
        const li = C.el('li', 'lw-list-item');
        const btn = C.el('button', 'lw-list-item-link');
        btn.type = 'button';
        const text = C.el('div', 'lw-list-item-text');
        text.appendChild(C.el('div', 'lw-list-item-title', note.title));
        const meta = ['in ' + note.scope.kind];
        for (const tag of entry.tags || []) meta.push('#' + tag.slug);
        text.appendChild(C.el('div', 'lw-list-item-subtitle', meta.join(' · ')));
        btn.appendChild(text);
        btn.addEventListener('click', function () {
            window.lwNoteEditor.open({ entry: entry, onSaved: run });
        });
        li.appendChild(btn);
        return li;
    }
})();
"#;
