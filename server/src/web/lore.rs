//! Settings & lore pages: the caller's settings, and per-setting note
//! authoring with markdown, tags, and visibility — the web counterpart
//! of the mobile setting list / detail / note editor screens.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::web::auth_ui::LoginRequired;
use crate::web::breadcrumbs::{Breadcrumbs, Crumb};

/// `/lore` — settings the caller owns or collaborates on.
#[component]
pub fn LoreSettingsPage() -> impl IntoView {
    view! {
        <section class="lw-page">
            <Breadcrumbs trail=vec![Crumb::link("Home", "/"), Crumb::here("Settings & lore")]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 class="lw-page-title">"Settings & lore"</h1>
                    <button id="lw-setting-new" class="lw-btn lw-btn-filled" type="button">"New setting"</button>
                </header>
                <ul id="lw-setting-list" class="lw-list"></ul>
                <p id="lw-setting-status" class="lw-picker-status">"Loading…"</p>
            </div>
            <script inner_html=SETTINGS_SCRIPT></script>
        </section>
    }
}

/// `/lore/:uuid` — one setting's notes, with the note editor modal.
#[component]
pub fn LoreSettingDetailPage() -> impl IntoView {
    let params = use_params_map();
    let uuid = move || params.read().get("uuid").unwrap_or_default();
    let initial_uuid = uuid();

    view! {
        <section class="lw-page" data-setting-uuid=initial_uuid>
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Settings & lore", "/lore"),
                Crumb::slot("lw-crumb-leaf"),
            ]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 id="lw-setting-title" class="lw-page-title">"…"</h1>
                    <button id="lw-note-new" class="lw-btn lw-btn-filled" type="button">"New note"</button>
                </header>
                <ul id="lw-note-list" class="lw-list"></ul>
                <p id="lw-note-status" class="lw-picker-status">"Loading…"</p>
            </div>
            <script inner_html=NOTE_EDITOR_SCRIPT></script>
            <script inner_html=SETTING_DETAIL_SCRIPT></script>
        </section>
    }
}

const SETTINGS_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const list = document.getElementById('lw-setting-list');
    const status = document.getElementById('lw-setting-status');
    const newBtn = document.getElementById('lw-setting-new');

    C.requireAuth(function () {
        load();
        newBtn.addEventListener('click', createDialog);
    });

    function load() {
        C.fetchJson('/api/settings').then(function (settings) {
            list.replaceChildren();
            status.textContent = settings.length === 0
                ? 'No settings yet — create one to start writing lore.'
                : '';
            for (const setting of settings) {
                const li = C.el('li', 'lw-list-item');
                const a = C.el('a', 'lw-list-item-link');
                a.href = '/lore/' + setting.uuid;
                const text = C.el('div', 'lw-list-item-text');
                text.appendChild(C.el('div', 'lw-list-item-title', setting.name));
                if (setting.published_as_module_uuid) {
                    text.appendChild(C.el('div', 'lw-list-item-subtitle', 'Published as a module'));
                }
                a.appendChild(text);
                li.appendChild(a);
                list.appendChild(li);
            }
        }).catch(function (err) {
            status.textContent = 'Failed to load: ' + err;
        });
    }

    function createDialog() {
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', 'New setting'));
        const name = C.el('input', 'lw-input');
        name.type = 'text';
        name.placeholder = 'Setting name';
        const field = C.el('label', 'lw-field', 'Name');
        field.appendChild(name);
        modal.panel.appendChild(field);
        const error = C.el('p', 'lw-form-error');
        error.hidden = true;
        modal.panel.appendChild(error);
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const ok = C.el('button', 'lw-btn lw-btn-filled', 'Create');
        ok.type = 'button';
        ok.addEventListener('click', function () {
            const value = name.value.trim();
            if (!value) return;
            fetch('/api/settings', {
                method: 'POST',
                headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
                body: JSON.stringify({ name: value }),
            }).then(function (r) {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            }).then(function (created) {
                location.href = '/lore/' + created.uuid;
            }).catch(function (err) {
                error.hidden = false;
                error.textContent = 'Failed: ' + err;
            });
        });
        actions.appendChild(cancel);
        actions.appendChild(ok);
        modal.panel.appendChild(actions);
        name.focus();
    }
})();
"#;

const SETTING_DETAIL_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const root = document.querySelector('[data-setting-uuid]');
    const settingUuid = root && root.dataset.settingUuid;
    const titleEl = document.getElementById('lw-setting-title');
    const list = document.getElementById('lw-note-list');
    const status = document.getElementById('lw-note-status');
    const newBtn = document.getElementById('lw-note-new');

    C.requireAuth(function () {
        C.fetchJson('/api/settings/' + encodeURIComponent(settingUuid)).then(function (setting) {
            titleEl.textContent = setting.name;
            document.title = setting.name + ' — Lorewyld';
            const crumb = document.getElementById('lw-crumb-leaf');
            if (crumb) crumb.textContent = setting.name;
        }).catch(function () {
            titleEl.textContent = 'Setting not found';
        });
        loadNotes();
        newBtn.addEventListener('click', function () {
            window.lwNoteEditor.open({
                scope: { kind: 'setting', target_uuid: settingUuid },
                onSaved: loadNotes,
            });
        });
    });

    function loadNotes() {
        C.fetchJson(
            '/api/lore-notes?scope_kind=setting&scope_target=' + encodeURIComponent(settingUuid)
        ).then(function (notes) {
            list.replaceChildren();
            status.textContent = notes.length === 0 ? 'No notes yet.' : '';
            for (const entry of notes) {
                list.appendChild(buildNoteRow(entry));
            }
        }).catch(function (err) {
            status.textContent = 'Failed to load: ' + err;
        });
    }

    function buildNoteRow(entry) {
        const note = entry.note;
        const li = C.el('li', 'lw-list-item');
        const btn = C.el('button', 'lw-list-item-link');
        btn.type = 'button';
        const text = C.el('div', 'lw-list-item-text');
        text.appendChild(C.el('div', 'lw-list-item-title', note.title));
        const meta = [];
        if (note.visibility === 'author_only') meta.push('Only me');
        if (note.visibility === 'gamemaster_only') meta.push('GMs only');
        for (const tag of entry.tags || []) meta.push('#' + tag.slug);
        if (meta.length) text.appendChild(C.el('div', 'lw-list-item-subtitle', meta.join(' · ')));
        btn.appendChild(text);
        btn.addEventListener('click', function () {
            window.lwNoteEditor.open({ entry: entry, onSaved: loadNotes });
        });
        li.appendChild(btn);
        return li;
    }
})();
"#;

/// Shared note-editor modal, exposed as `window.lwNoteEditor.open()`.
/// Used by the setting detail page and the search page; injected once
/// per page that needs it.
pub const NOTE_EDITOR_SCRIPT: &str = r#"
(function () {
    if (window.lwNoteEditor) return;
    const C = window.lwContent;

    const VISIBILITIES = [
        ['visible', 'Visible to everyone'],
        ['author_only', 'Only me'],
        ['gamemaster_only', 'GMs only'],
    ];

    /* opts: { entry?, scope?, onSaved? } — entry = LoreNoteWithTags to
     * edit; scope = NoteScope for a brand-new note. */
    function open(opts) {
        const existing = opts.entry ? opts.entry.note : null;
        const tags = (opts.entry ? opts.entry.tags : []).map(function (t) { return t.slug; });

        const modal = C.openModal('lw-picker-modal');
        const panel = modal.panel;
        panel.appendChild(C.el('h2', 'lw-modal-title', existing ? 'Edit note' : 'New note'));

        const title = C.el('input', 'lw-input');
        title.type = 'text';
        title.placeholder = 'Title';
        title.value = existing ? existing.title : '';
        const titleField = C.el('label', 'lw-field', 'Title');
        titleField.appendChild(title);
        panel.appendChild(titleField);

        const body = C.el('textarea', 'lw-input lw-note-body');
        body.rows = 12;
        body.placeholder = 'Write your lore in markdown…';
        body.value = existing ? existing.body_markdown : '';
        const bodyField = C.el('label', 'lw-field', 'Body (markdown)');
        bodyField.appendChild(body);
        panel.appendChild(bodyField);

        // Removable tag chips + commit-on-Enter/comma input.
        const tagField = C.el('label', 'lw-field', 'Tags');
        const chipRow = C.el('div', 'lw-chip-row');
        const tagInput = C.el('input', 'lw-input');
        tagInput.type = 'text';
        tagInput.placeholder = 'Add a tag and press Enter';
        function renderChips() {
            chipRow.replaceChildren();
            for (const slug of tags) {
                const chip = C.el('span', 'lw-chip lw-chip-selected', '#' + slug);
                const x = C.el('button', 'lw-chip-remove', '✕');
                x.type = 'button';
                x.setAttribute('aria-label', 'Remove tag ' + slug);
                x.addEventListener('click', function () {
                    tags.splice(tags.indexOf(slug), 1);
                    renderChips();
                });
                chip.appendChild(x);
                chipRow.appendChild(chip);
            }
        }
        tagInput.addEventListener('keydown', function (e) {
            if (e.key !== 'Enter' && e.key !== ',') return;
            e.preventDefault();
            const slug = tagInput.value.trim().toLowerCase().replace(/\s+/g, '-').replace(/,/g, '');
            if (slug && !tags.includes(slug)) {
                tags.push(slug);
                renderChips();
            }
            tagInput.value = '';
        });
        renderChips();
        tagField.appendChild(chipRow);
        tagField.appendChild(tagInput);
        panel.appendChild(tagField);

        const visibility = C.el('select', 'lw-input');
        for (const v of VISIBILITIES) {
            const opt = C.el('option', null, v[1]);
            opt.value = v[0];
            visibility.appendChild(opt);
        }
        visibility.value = existing ? existing.visibility : 'visible';
        const visField = C.el('label', 'lw-field', 'Visibility');
        visField.appendChild(visibility);
        panel.appendChild(visField);

        const error = C.el('p', 'lw-form-error');
        error.hidden = true;
        panel.appendChild(error);

        const actions = C.el('div', 'lw-modal-actions');
        if (existing) {
            const del = C.el('button', 'lw-btn lw-btn-danger', 'Delete');
            del.type = 'button';
            del.addEventListener('click', function () {
                if (!confirm('Delete "' + existing.title + '"?')) return;
                fetch('/api/lore-notes/' + existing.uuid, {
                    method: 'DELETE',
                    headers: window.lw.authHeaders(),
                }).then(function (r) {
                    if (!r.ok && r.status !== 204) throw new Error('HTTP ' + r.status);
                    modal.close();
                    if (opts.onSaved) opts.onSaved();
                }).catch(function (err) {
                    error.hidden = false;
                    error.textContent = 'Delete failed: ' + err;
                });
            });
            actions.appendChild(del);
        }
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        actions.appendChild(cancel);
        const save = C.el('button', 'lw-btn lw-btn-filled', 'Save');
        save.type = 'button';
        save.addEventListener('click', function () {
            const titleValue = title.value.trim();
            if (!titleValue) {
                error.hidden = false;
                error.textContent = 'A title is required.';
                return;
            }
            save.disabled = true;
            const request = existing
                ? fetch('/api/lore-notes/' + existing.uuid, {
                    method: 'PATCH',
                    headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
                    body: JSON.stringify({
                        title: titleValue,
                        body_markdown: body.value,
                        visibility: visibility.value,
                        tag_slugs: tags,
                    }),
                })
                : fetch('/api/lore-notes', {
                    method: 'POST',
                    headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
                    body: JSON.stringify({
                        title: titleValue,
                        body_markdown: body.value,
                        scope: opts.scope,
                        visibility: visibility.value,
                        tag_slugs: tags,
                    }),
                });
            request.then(function (r) {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                modal.close();
                if (opts.onSaved) opts.onSaved();
            }).catch(function (err) {
                save.disabled = false;
                error.hidden = false;
                error.textContent = 'Save failed: ' + err;
            });
        });
        actions.appendChild(save);
        panel.appendChild(actions);
        title.focus();
    }

    window.lwNoteEditor = { open: open };
})();
"#;
