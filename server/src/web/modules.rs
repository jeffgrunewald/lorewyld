//! Content module pages. Public browse stays as before; admins
//! additionally manage the module lifecycle: install by uploading a
//! ContentBundle package, disable/reinstall any module, and fully
//! uninstall non-bundled modules behind a two-step confirmation.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::web::breadcrumbs::{Breadcrumbs, Crumb};

/// List of content modules. Skeleton rendered server-side; JS
/// populates from `/api/server-info` (public) or `/api/admin/modules`
/// (admin session).
#[component]
pub fn ModulesPage() -> impl IntoView {
    view! {
        <section class="lw-page lw-modules">
            <Breadcrumbs trail=vec![Crumb::link("Home", "/"), Crumb::here("Modules")]/>
            <header class="lw-page-header">
                <h1 class="lw-page-title">"Content modules"</h1>
                <div id="lw-modules-admin-toolbar" class="lw-toolbar" hidden=true>
                    <input id="lw-module-file" type="file" accept="application/json,.json" hidden=true/>
                    <button id="lw-module-install" class="lw-btn lw-btn-filled" type="button">
                        "Install module…"
                    </button>
                </div>
            </header>
            <p class="lw-modules-help">
                "Content packs on this server. Tap any module for details and lore notes."
            </p>
            <div id="lw-module-install-progress" hidden=true>
                <div class="lw-progress"></div>
                <p class="lw-picker-status">"Installing — large bundles can take a minute…"</p>
            </div>
            <p id="lw-module-install-error" class="lw-form-error" hidden=true></p>
            <ul id="lw-modules-list" class="lw-modules-list">
                <li class="lw-modules-loading">"Loading modules…"</li>
            </ul>
            <script inner_html=MODULES_LIST_SCRIPT></script>
        </section>
    }
}

/// Detail page for a single module — metadata, facts, record counts
/// (admin), lore notes, and the admin lifecycle actions.
/// URL: `/modules/:uuid`.
#[component]
pub fn ModuleDetailPage() -> impl IntoView {
    let params = use_params_map();
    let uuid = move || params.read().get("uuid").unwrap_or_default();
    let initial_uuid = uuid();

    view! {
        <section class="lw-page lw-module-detail" data-module-uuid=initial_uuid>
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Modules", "/modules"),
                Crumb::slot("lw-crumb-leaf"),
            ]/>
            <header class="lw-module-detail-header">
                <h1 id="lw-module-name" class="lw-page-title">"Loading…"</h1>
                <div id="lw-module-meta" class="lw-module-detail-meta"></div>
            </header>
            <div id="lw-module-description" class="lw-module-detail-description"></div>
            <div id="lw-module-facts" class="lw-card" hidden=true></div>
            <div id="lw-module-counts" class="lw-chip-row"></div>
            <div id="lw-module-actions" class="lw-card" hidden=true>
                <h2 class="lw-card-title">"Manage"</h2>
                <p id="lw-module-state-line" class="lw-page-subtitle"></p>
                <div class="lw-toolbar">
                    <button id="lw-module-disable" class="lw-btn lw-btn-tonal" type="button" hidden=true>
                        "Disable"
                    </button>
                    <button id="lw-module-reinstall" class="lw-btn lw-btn-filled" type="button" hidden=true>
                        "Reinstall"
                    </button>
                    <button id="lw-module-uninstall" class="lw-btn lw-btn-danger" type="button" hidden=true>
                        "Uninstall"
                    </button>
                </div>
                <p id="lw-module-action-note" class="lw-picker-status"></p>
            </div>
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
    const C = window.lwContent;
    const list = document.getElementById('lw-modules-list');
    const toolbar = document.getElementById('lw-modules-admin-toolbar');
    const fileInput = document.getElementById('lw-module-file');
    const installBtn = document.getElementById('lw-module-install');
    const progress = document.getElementById('lw-module-install-progress');
    const installError = document.getElementById('lw-module-install-error');

    document.addEventListener('lw-auth-ready', function (e) {
        if (e.detail && e.detail.admin) {
            toolbar.hidden = false;
            installBtn.addEventListener('click', function () { fileInput.click(); });
            fileInput.addEventListener('change', onFilePicked);
            loadAdmin();
        } else {
            loadPublic();
        }
    });

    function loadPublic() {
        fetch('/api/server-info')
            .then(function (r) {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            })
            .then(function (data) {
                renderList((data.modules || []).map(function (m) {
                    return { module: m, origin: null, record_counts: null };
                }));
            })
            .catch(showLoadError);
    }

    function loadAdmin() {
        C.fetchJson('/api/admin/modules')
            .then(function (summaries) { renderList(summaries); })
            .catch(showLoadError);
    }

    function showLoadError(err) {
        list.replaceChildren();
        list.appendChild(C.el('li', 'lw-modules-error', 'Failed to load: ' + err));
    }

    function renderList(summaries) {
        list.replaceChildren();
        if (summaries.length === 0) {
            list.appendChild(C.el('li', 'lw-modules-empty',
                'No modules installed. Upload a content package or publish a setting from the mobile app.'));
            return;
        }
        for (const summary of summaries) {
            list.appendChild(buildModuleCard(summary));
        }
    }

    function buildModuleCard(summary) {
        const m = summary.module;
        const card = C.el('li', 'lw-module-card' + (m.is_active === false ? ' lw-module-disabled' : ''));

        const link = C.el('a', 'lw-module-card-link');
        link.href = '/modules/' + encodeURIComponent(m.uuid);
        link.appendChild(C.el('span', 'lw-module-card-name', m.name));
        link.appendChild(C.el('span', 'lw-module-card-version', 'v' + (m.version_string || '')));
        card.appendChild(link);

        const meta = C.el('div', 'lw-module-card-meta');
        if (m.license) {
            meta.appendChild(C.el('span', 'lw-module-card-license', m.license));
        }
        meta.appendChild(C.el('span', 'lw-module-card-authors',
            'by ' + ((m.authors || []).join(', ') || 'unknown')));
        if (summary.origin === 'bundled') {
            meta.appendChild(C.el('span', 'lw-module-tag', 'Bundled'));
        }
        if (m.is_active === false) {
            meta.appendChild(C.el('span', 'lw-module-tag lw-module-tag-disabled',
                'Disabled ' + (m.updated_at ? new Date(m.updated_at).toLocaleDateString() : '')));
        }
        card.appendChild(meta);

        if (m.description) {
            card.appendChild(C.el('p', 'lw-module-card-desc', m.description));
        }

        if (summary.record_counts && summary.record_counts.length) {
            const chips = C.el('div', 'lw-chip-row');
            for (const rc of summary.record_counts) {
                if (rc.count > 0) {
                    chips.appendChild(C.el('span', 'lw-module-tag',
                        rc.count.toLocaleString() + ' ' + C.categoryPlural(rc.category)));
                }
            }
            card.appendChild(chips);
        }
        return card;
    }

    function onFilePicked() {
        const file = fileInput.files && fileInput.files[0];
        fileInput.value = '';
        if (!file) return;
        installError.hidden = true;
        file.text().then(function (text) {
            let bundle;
            try {
                bundle = JSON.parse(text);
            } catch (e) {
                throw new Error('not valid JSON');
            }
            if (!bundle.schema || !Array.isArray(bundle.modules)) {
                throw new Error('not a Lorewyld content bundle (missing schema/modules)');
            }
            progress.hidden = false;
            installBtn.disabled = true;
            return fetch('/api/admin/modules/install', {
                method: 'POST',
                headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
                body: text,
            }).then(function (r) {
                return r.json().catch(function () { return {}; }).then(function (body) {
                    if (!r.ok) throw new Error(body.message || ('HTTP ' + r.status));
                    return body;
                });
            });
        }).then(function (response) {
            if (!response) return;
            progress.hidden = true;
            installBtn.disabled = false;
            loadAdmin();
        }).catch(function (err) {
            progress.hidden = true;
            installBtn.disabled = false;
            installError.hidden = false;
            installError.textContent = 'Install failed: ' + (err.message || err);
        });
    }
})();
"#;

const MODULE_DETAIL_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const root = document.querySelector('[data-module-uuid]');
    const uuid = root && root.dataset.moduleUuid;
    const nameEl = document.getElementById('lw-module-name');
    const metaEl = document.getElementById('lw-module-meta');
    const descEl = document.getElementById('lw-module-description');
    const factsEl = document.getElementById('lw-module-facts');
    const countsEl = document.getElementById('lw-module-counts');
    const actionsEl = document.getElementById('lw-module-actions');
    const stateLine = document.getElementById('lw-module-state-line');
    const disableBtn = document.getElementById('lw-module-disable');
    const reinstallBtn = document.getElementById('lw-module-reinstall');
    const uninstallBtn = document.getElementById('lw-module-uninstall');
    const actionNote = document.getElementById('lw-module-action-note');
    const notesEl = document.getElementById('lw-module-notes');

    if (!uuid) {
        nameEl.textContent = 'Missing module UUID in URL';
        return;
    }

    // Public metadata + notes render for everyone; the admin summary
    // (origin, counts, lifecycle actions) layers on top.
    fetch('/api/modules/' + encodeURIComponent(uuid))
        .then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        })
        .then(function (data) { renderPublic(data.module, data.notes || []); })
        .catch(function (err) {
            nameEl.textContent = 'Module not found';
            notesEl.replaceChildren(C.el('p', 'lw-modules-error', 'Failed to load: ' + err));
        });

    document.addEventListener('lw-auth-ready', function (e) {
        if (e.detail && e.detail.admin) loadAdminSummary();
    });

    function renderPublic(m, notes) {
        document.title = m.name + ' — Lorewyld';
        nameEl.textContent = m.name;
        const crumb = document.getElementById('lw-crumb-leaf');
        if (crumb) crumb.textContent = m.name;

        // Skip empty values so the CSS dot separators never dangle.
        metaEl.replaceChildren();
        function appendMeta(cls, text) {
            if (text) metaEl.appendChild(C.el('span', cls, text));
        }
        appendMeta('lw-module-detail-version', m.version_string ? 'v' + m.version_string : '');
        appendMeta('lw-module-detail-license', m.license);
        appendMeta('lw-module-detail-authors',
            'by ' + ((m.authors || []).join(', ') || 'unknown'));

        descEl.replaceChildren();
        if (m.description) {
            descEl.appendChild(C.el('p', null, m.description));
        }

        const facts = [];
        if (m.license) facts.push(['License', m.license]);
        if (m.publisher) facts.push(['Publisher', m.publisher]);
        if ((m.authors || []).length) facts.push(['Authors', m.authors.join(', ')]);
        if (m.website_url) facts.push(['Website', m.website_url]);
        if (m.license_url) facts.push(['License URL', m.license_url]);
        if (m.release_date) facts.push(['Released', m.release_date]);
        if (facts.length) {
            factsEl.hidden = false;
            const wrap = C.el('div', 'lw-facts');
            for (const fact of facts) {
                const row = C.el('div', 'lw-fact-row');
                row.appendChild(C.el('span', 'lw-fact-label', fact[0]));
                if (fact[0] === 'Website' || fact[0] === 'License URL') {
                    const value = C.el('span', 'lw-fact-value');
                    const a = C.el('a', null, fact[1]);
                    if (/^https?:\/\//.test(fact[1])) a.href = fact[1];
                    a.rel = 'noopener noreferrer';
                    a.target = '_blank';
                    value.appendChild(a);
                    row.appendChild(value);
                } else {
                    row.appendChild(C.el('span', 'lw-fact-value', fact[1]));
                }
                wrap.appendChild(row);
            }
            factsEl.replaceChildren(wrap);
        }

        notesEl.replaceChildren();
        if (notes.length === 0) {
            notesEl.appendChild(C.el('p', 'lw-modules-empty', 'No lore notes in this module.'));
        }
        for (const n of notes) {
            notesEl.appendChild(buildNote(n));
        }
    }

    function loadAdminSummary() {
        C.fetchJson('/api/admin/modules').then(function (summaries) {
            const summary = summaries.find(function (s) { return s.module.uuid === uuid; });
            if (summary) renderAdmin(summary);
        }).catch(function () { /* non-fatal: admin extras just don't render */ });
    }

    function renderAdmin(summary) {
        const m = summary.module;
        const totalRecords = (summary.record_counts || [])
            .reduce(function (n, rc) { return n + rc.count; }, 0);

        countsEl.replaceChildren();
        for (const rc of summary.record_counts || []) {
            if (rc.count > 0) {
                countsEl.appendChild(C.el('span', 'lw-module-tag',
                    rc.count.toLocaleString() + ' ' + C.categoryPlural(rc.category)));
            }
        }
        if (summary.lore_note_count > 0) {
            countsEl.appendChild(C.el('span', 'lw-module-tag',
                summary.lore_note_count.toLocaleString() + ' lore notes'));
        }

        actionsEl.hidden = false;
        stateLine.textContent =
            (m.is_active ? 'Active' : 'Disabled') +
            ' · ' + (summary.origin === 'bundled' ? 'bundled with the server' : summary.origin);

        // The SRD module is pinned — every other module references its
        // shared rules vocabulary — so it can never be disabled. The
        // server enforces this; the disabled button mirrors it.
        const pinned = m.slug === 'srd';
        disableBtn.hidden = !m.is_active;
        disableBtn.disabled = pinned;
        reinstallBtn.hidden = m.is_active;
        uninstallBtn.hidden = summary.origin === 'bundled';
        actionNote.textContent = pinned
            ? 'Required module — it provides the shared rules vocabulary every other ' +
              'module references, and cannot be disabled.'
            : summary.origin === 'bundled'
                ? 'Bundled modules can only be disabled, not uninstalled.'
                : '';

        disableBtn.onclick = pinned ? null : function () { confirmDisable(summary, totalRecords); };
        reinstallBtn.onclick = function () { setActive(true); };
        uninstallBtn.onclick = function () { confirmUninstall(summary, totalRecords); };
    }

    function setActive(isActive) {
        actionNote.textContent = isActive ? 'Reinstalling…' : 'Disabling…';
        fetch('/api/admin/modules/' + encodeURIComponent(uuid), {
            method: 'PATCH',
            headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
            body: JSON.stringify({ is_active: isActive }),
        }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            location.reload();
        }).catch(function (err) {
            actionNote.textContent = 'Failed: ' + err;
        });
    }

    function confirmDisable(summary, totalRecords) {
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', 'Disable module?'));
        modal.panel.appendChild(C.el('p', null,
            '"' + summary.module.name + '" and its ' + totalRecords.toLocaleString() +
            ' records will be hidden from everyone on this server until it is reinstalled. ' +
            'Nothing is deleted.'));
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const ok = C.el('button', 'lw-btn lw-btn-filled', 'Disable');
        ok.type = 'button';
        ok.addEventListener('click', function () {
            modal.close();
            setActive(false);
        });
        actions.appendChild(cancel);
        actions.appendChild(ok);
        modal.panel.appendChild(actions);
    }

    // Two distinct confirmations: consequences, then a type-the-slug
    // gate before the destructive call.
    function confirmUninstall(summary, totalRecords) {
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', 'Uninstall module?'));
        modal.panel.appendChild(C.el('p', null,
            'This permanently removes "' + summary.module.name + '", its ' +
            totalRecords.toLocaleString() + ' content records, and its ' +
            (summary.lore_note_count || 0) + ' lore notes from the server. ' +
            'This cannot be undone. To make the module temporarily unavailable, disable it instead.'));
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const next = C.el('button', 'lw-btn lw-btn-danger', 'Continue');
        next.type = 'button';
        next.addEventListener('click', function () {
            modal.close();
            confirmUninstallSlug(summary);
        });
        actions.appendChild(cancel);
        actions.appendChild(next);
        modal.panel.appendChild(actions);
    }

    function confirmUninstallSlug(summary) {
        const slug = summary.module.slug;
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', 'Confirm uninstall'));
        modal.panel.appendChild(C.el('p', null,
            'Type the module slug "' + slug + '" to confirm permanent removal.'));
        const input = C.el('input', 'lw-input');
        input.type = 'text';
        input.placeholder = slug;
        modal.panel.appendChild(input);
        const error = C.el('p', 'lw-form-error');
        error.hidden = true;
        modal.panel.appendChild(error);
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const confirm = C.el('button', 'lw-btn lw-btn-danger', 'Permanently uninstall');
        confirm.type = 'button';
        confirm.disabled = true;
        input.addEventListener('input', function () {
            confirm.disabled = input.value.trim() !== slug;
        });
        confirm.addEventListener('click', function () {
            confirm.disabled = true;
            fetch('/api/admin/modules/' + encodeURIComponent(uuid), {
                method: 'DELETE',
                headers: window.lw.authHeaders(),
            }).then(function (r) {
                if (!r.ok && r.status !== 204) {
                    return r.json().catch(function () { return {}; }).then(function (body) {
                        throw new Error(body.message || ('HTTP ' + r.status));
                    });
                }
                location.href = '/modules';
            }).catch(function (err) {
                confirm.disabled = false;
                error.hidden = false;
                error.textContent = 'Uninstall failed: ' + (err.message || err);
            });
        });
        actions.appendChild(cancel);
        actions.appendChild(confirm);
        modal.panel.appendChild(actions);
        input.focus();
    }

    function buildNote(n) {
        const article = C.el('article', 'lw-lore-note');
        article.appendChild(C.el('h3', 'lw-lore-note-title', n.note.title));

        if ((n.tags || []).length > 0) {
            const tagsRow = C.el('div', 'lw-lore-note-tags');
            for (const t of n.tags) {
                tagsRow.appendChild(C.el('span', 'lw-tag-chip', t.display_name));
            }
            article.appendChild(tagsRow);
        }

        const body = C.el('div', 'lw-lore-note-body lw-md');
        body.appendChild(C.renderMarkdown(n.note.body_markdown || ''));
        article.appendChild(body);

        return article;
    }
})();
"#;
