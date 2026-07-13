/* Homebrew content authoring for the Lorewyld web app.
 *
 * A schema-driven form builder: every control is derived from the
 * category's FieldSchema (single-sourced in lorewyld-domain, reached here
 * via window.lwContent.fieldSchema / validateRecord, which call the shared
 * WASM core). No per-type form code lives here — adding a content type is
 * a schema change, not a UI change.
 *
 * Depends on window.lwContent (loaded first as a blocking script). All DOM
 * goes through C.el (createElement/textContent) so authored text can't
 * escape into markup.
 */
window.lwAuthoring = (function () {
    'use strict';

    const C = window.lwContent;

    function authHeaders() {
        return window.lw && window.lw.authHeaders ? window.lw.authHeaders() : {};
    }

    function jsonHeaders() {
        return Object.assign({ 'content-type': 'application/json' }, authHeaders());
    }

    function submit(method, url, body) {
        return fetch(url, { method: method, headers: jsonHeaders(), body: JSON.stringify(body) })
            .then(function (r) {
                return r.json()
                    .catch(function () { return null; })
                    .then(function (data) { return { ok: r.ok, status: r.status, data: data }; });
            });
    }

    /* The local (homebrew) modules content may live in. Memoized per page;
     * pass force=true to refresh after creating a custom module. */
    let _editable = null;
    function editableModules(force) {
        if (!_editable || force) {
            _editable = C.fetchJson('/api/modules/editable').catch(function (err) {
                _editable = null;
                throw err;
            });
        }
        return _editable;
    }

    function recordIsEditable(record) {
        if (!record || typeof record.content_module_uuid !== 'string') return Promise.resolve(false);
        return editableModules()
            .then(function (mods) {
                return mods.some(function (m) { return m.uuid === record.content_module_uuid; });
            })
            .catch(function () { return false; });
    }

    /* select_lookup fields reference one of the lookup tables already loaded
     * by C.loadLookups — map the table name to that uuid→name dict. */
    const LOOKUP_MAP = {
        spell_school: 'spellSchools',
        size: 'sizes',
        creature_type: 'creatureTypes',
        item_category: 'itemCategories',
        class: 'classes',
        species: 'species',
    };

    function lookupEntries(table, lookups) {
        const mapName = LOOKUP_MAP[table];
        const map = mapName ? lookups[mapName] : null;
        if (!map) return [];
        return Object.keys(map)
            .map(function (uuid) { return { value: uuid, label: map[uuid] }; })
            .sort(function (a, b) { return String(a.label).localeCompare(String(b.label)); });
    }

    function initialFor(field, record) {
        if (record && Object.prototype.hasOwnProperty.call(record, field.key)) {
            return record[field.key];
        }
        if (field.kind === 'json') return field.template !== undefined ? field.template : null;
        return undefined;
    }

    /* Builds one field control. Returns { key, el, collect, showError }
     * where collect() reads the live DOM and yields {value} or {error}. */
    function buildField(field, initial, lookups) {
        const wrap = C.el('label', 'lw-field', field.label + (field.required ? ' *' : ''));
        let control;
        let collect;

        switch (field.kind) {
            case 'markdown': {
                control = C.el('textarea', 'lw-input lw-authoring-area');
                control.rows = 6;
                if (typeof initial === 'string') control.value = initial;
                collect = function () { return { value: control.value }; };
                break;
            }
            case 'int':
            case 'float': {
                control = C.el('input', 'lw-input');
                control.type = 'number';
                control.step = field.kind === 'int' ? '1' : 'any';
                if (typeof field.min === 'number') control.min = String(field.min);
                if (typeof field.max === 'number') control.max = String(field.max);
                if (typeof initial === 'number') control.value = String(initial);
                collect = function () {
                    const t = control.value.trim();
                    if (t === '') return { value: null };
                    const n = Number(t);
                    // Pass the raw string when non-numeric so validation flags it.
                    return { value: Number.isNaN(n) ? control.value : n };
                };
                break;
            }
            case 'bool': {
                control = C.el('input');
                control.type = 'checkbox';
                control.checked = initial === true;
                collect = function () { return { value: control.checked }; };
                break;
            }
            case 'select_enum': {
                control = C.el('select', 'lw-input');
                if (!field.required) {
                    const none = C.el('option', null, '— none —');
                    none.value = '';
                    control.appendChild(none);
                }
                (field.options || []).forEach(function (opt) {
                    const o = C.el('option', null, opt.label);
                    o.value = opt.value;
                    control.appendChild(o);
                });
                if (typeof initial === 'string') control.value = initial;
                collect = function () { return { value: control.value === '' ? null : control.value }; };
                break;
            }
            case 'select_lookup': {
                control = C.el('select', 'lw-input');
                const none = C.el('option', null, field.required ? '— select —' : '— none —');
                none.value = '';
                control.appendChild(none);
                lookupEntries(field.table, lookups).forEach(function (opt) {
                    const o = C.el('option', null, opt.label);
                    o.value = opt.value;
                    control.appendChild(o);
                });
                if (typeof initial === 'string') control.value = initial;
                collect = function () { return { value: control.value === '' ? null : control.value }; };
                break;
            }
            case 'enum_list': {
                control = C.el('div', 'lw-chip-row');
                const selected = new Set(Array.isArray(initial) ? initial : []);
                (field.options || []).forEach(function (opt) {
                    const chip = C.el(
                        'button',
                        'lw-chip' + (selected.has(opt.value) ? ' lw-chip-selected' : ''),
                        opt.label
                    );
                    chip.type = 'button';
                    chip.addEventListener('click', function () {
                        if (selected.has(opt.value)) {
                            selected.delete(opt.value);
                            chip.classList.remove('lw-chip-selected');
                        } else {
                            selected.add(opt.value);
                            chip.classList.add('lw-chip-selected');
                        }
                    });
                    control.appendChild(chip);
                });
                collect = function () { return { value: Array.from(selected) }; };
                break;
            }
            case 'json': {
                control = C.el('textarea', 'lw-input lw-authoring-area lw-authoring-json');
                control.rows = 5;
                const init = initial === undefined || initial === null
                    ? (field.template !== undefined ? field.template : null)
                    : initial;
                control.value = init === null || init === undefined ? '' : JSON.stringify(init, null, 2);
                collect = function () {
                    const t = control.value.trim();
                    if (t === '') return { value: field.required ? null : undefined };
                    try {
                        return { value: JSON.parse(t) };
                    } catch (e) {
                        return { error: 'Invalid JSON: ' + e.message };
                    }
                };
                break;
            }
            case 'text':
            default: {
                control = C.el('input', 'lw-input');
                control.type = 'text';
                if (typeof initial === 'string') control.value = initial;
                collect = function () { return { value: control.value }; };
                break;
            }
        }

        const errorEl = C.el('p', 'lw-form-error');
        errorEl.hidden = true;

        if (field.kind === 'bool') {
            // Checkbox sits before its label text.
            wrap.textContent = '';
            wrap.classList.add('lw-field-check');
            wrap.appendChild(control);
            wrap.appendChild(C.el('span', null, field.label));
        } else {
            wrap.appendChild(control);
        }
        if (field.help) wrap.appendChild(C.el('p', 'lw-field-help', field.help));
        wrap.appendChild(errorEl);

        return {
            key: field.key,
            el: wrap,
            collect: collect,
            showError: function (msg) {
                if (msg) { errorEl.textContent = msg; errorEl.hidden = false; }
                else { errorEl.hidden = true; }
            },
        };
    }

    function applyErrors(fields, generalError, errors) {
        const byKey = {};
        fields.forEach(function (f) { byKey[f.key] = f; });
        const general = [];
        (errors || []).forEach(function (e) {
            if (e && e.field && byKey[e.field]) byKey[e.field].showError(e.message);
            else general.push((e && e.message) || String(e));
        });
        if (general.length) {
            generalError.textContent = general.join('; ');
            generalError.hidden = false;
        }
    }

    /* Create/edit form. `record` null → create; otherwise edit. */
    function openForm(category, record, onSaved) {
        const schema = C.fieldSchema(category);
        if (!schema) {
            window.alert('"' + category + '" is not an authorable content type.');
            return;
        }
        const mode = record ? 'edit' : 'create';
        const modal = C.openModal('lw-authoring-modal');
        const panel = modal.panel;
        panel.appendChild(
            C.el('h2', 'lw-modal-title', (mode === 'create' ? 'New ' : 'Edit ') + C.humanizeSlug(category))
        );
        const form = C.el('div', 'lw-authoring-form');
        panel.appendChild(form);
        form.appendChild(C.el('p', 'lw-picker-status', 'Loading…'));

        const needs = mode === 'create'
            ? Promise.all([C.loadLookups(), editableModules().catch(function () { return []; })])
            : C.loadLookups().then(function (l) { return [l, null]; });

        needs.then(function (res) {
            const lookups = res[0];
            const mods = res[1];
            form.replaceChildren();

            let moduleSelect = null;
            if (mode === 'create') {
                const mwrap = C.el('label', 'lw-field', 'Save to module');
                moduleSelect = C.el('select', 'lw-input');
                const def = C.el('option', null, 'Homebrew (default)');
                def.value = '';
                moduleSelect.appendChild(def);
                (mods || [])
                    .filter(function (m) { return m.slug !== 'homebrew'; })
                    .forEach(function (m) {
                        const o = C.el('option', null, m.name);
                        o.value = m.uuid;
                        moduleSelect.appendChild(o);
                    });
                mwrap.appendChild(moduleSelect);
                form.appendChild(mwrap);
            }

            const fields = schema.fields.map(function (f) {
                const fb = buildField(f, initialFor(f, record), lookups);
                form.appendChild(fb.el);
                return fb;
            });

            const generalError = C.el('p', 'lw-form-error');
            generalError.hidden = true;
            form.appendChild(generalError);

            const actions = C.el('div', 'lw-modal-actions');
            const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
            cancel.type = 'button';
            cancel.addEventListener('click', modal.close);
            const save = C.el('button', 'lw-btn lw-btn-filled', mode === 'create' ? 'Create' : 'Save');
            save.type = 'button';
            actions.appendChild(cancel);
            actions.appendChild(save);
            form.appendChild(actions);

            save.addEventListener('click', function () {
                generalError.hidden = true;
                fields.forEach(function (f) { f.showError(null); });

                const input = {};
                let bad = false;
                fields.forEach(function (f) {
                    const c = f.collect();
                    if (c.error) { f.showError(c.error); bad = true; }
                    else if (c.value !== undefined) { input[f.key] = c.value; }
                });
                if (bad) return;

                // Pre-submit validation via the same lorewyld-domain rules
                // the server enforces.
                const verrs = C.validateRecord(category, input);
                if (verrs && verrs.length) { applyErrors(fields, generalError, verrs); return; }

                save.disabled = true;
                const url = mode === 'create'
                    ? '/api/content/' + encodeURIComponent(category)
                    : '/api/content/' + encodeURIComponent(category) + '/' + encodeURIComponent(record.uuid);
                const method = mode === 'create' ? 'POST' : 'PATCH';
                const body = mode === 'create'
                    ? { fields: input, module_uuid: moduleSelect && moduleSelect.value ? moduleSelect.value : null }
                    : { fields: input };

                submit(method, url, body).then(function (resp) {
                    if (resp.ok) { modal.close(); onSaved(resp.data); return; }
                    save.disabled = false;
                    if (resp.data && Array.isArray(resp.data.errors)) {
                        applyErrors(fields, generalError, resp.data.errors);
                    } else {
                        generalError.textContent =
                            (resp.data && resp.data.message) || ('Save failed (' + resp.status + ')');
                        generalError.hidden = false;
                    }
                }).catch(function (e) {
                    save.disabled = false;
                    generalError.textContent = 'Save failed: ' + e;
                    generalError.hidden = false;
                });
            });
        }).catch(function (e) {
            form.replaceChildren(C.el('p', 'lw-form-error', 'Failed to load form: ' + e));
        });
    }

    function moveModule(category, record, onSaved) {
        editableModules().then(function (mods) {
            const modal = C.openModal('lw-authoring-modal');
            const panel = modal.panel;
            panel.appendChild(C.el('h2', 'lw-modal-title', 'Move to module'));
            const wrap = C.el('label', 'lw-field', 'Module');
            const sel = C.el('select', 'lw-input');
            mods.forEach(function (m) {
                const o = C.el('option', null, m.slug === 'homebrew' ? 'Homebrew' : m.name);
                o.value = m.uuid;
                if (m.uuid === record.content_module_uuid) o.selected = true;
                sel.appendChild(o);
            });
            wrap.appendChild(sel);
            panel.appendChild(wrap);
            const err = C.el('p', 'lw-form-error');
            err.hidden = true;
            panel.appendChild(err);

            const actions = C.el('div', 'lw-modal-actions');
            const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
            cancel.type = 'button';
            cancel.addEventListener('click', modal.close);
            const ok = C.el('button', 'lw-btn lw-btn-filled', 'Move');
            ok.type = 'button';
            actions.appendChild(cancel);
            actions.appendChild(ok);
            panel.appendChild(actions);

            ok.addEventListener('click', function () {
                if (!sel.value) { modal.close(); return; }
                ok.disabled = true;
                submit(
                    'PATCH',
                    '/api/content/' + encodeURIComponent(category) + '/' + encodeURIComponent(record.uuid),
                    { module_uuid: sel.value }
                ).then(function (resp) {
                    if (resp.ok) { modal.close(); onSaved(resp.data); }
                    else {
                        ok.disabled = false;
                        err.textContent = (resp.data && resp.data.message) || 'Move failed';
                        err.hidden = false;
                    }
                }).catch(function (e) {
                    ok.disabled = false;
                    err.textContent = 'Move failed: ' + e;
                    err.hidden = false;
                });
            });
        }).catch(function (e) { window.alert('Failed to load modules: ' + e); });
    }

    function deleteRecord(category, uuid, onDeleted) {
        const modal = C.openModal('lw-authoring-modal');
        const panel = modal.panel;
        panel.appendChild(C.el('h2', 'lw-modal-title', 'Delete entry'));
        panel.appendChild(C.el('p', 'lw-confirm-text', 'This permanently deletes the entry. This cannot be undone.'));
        const err = C.el('p', 'lw-form-error');
        err.hidden = true;
        panel.appendChild(err);

        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const del = C.el('button', 'lw-btn lw-btn-danger', 'Delete');
        del.type = 'button';
        actions.appendChild(cancel);
        actions.appendChild(del);
        panel.appendChild(actions);

        del.addEventListener('click', function () {
            del.disabled = true;
            fetch(
                '/api/content/' + encodeURIComponent(category) + '/' + encodeURIComponent(uuid),
                { method: 'DELETE', headers: authHeaders() }
            ).then(function (r) {
                if (r.ok) { modal.close(); onDeleted(); }
                else {
                    del.disabled = false;
                    err.textContent = 'Delete failed (' + r.status + ')';
                    err.hidden = false;
                }
            }).catch(function (e) {
                del.disabled = false;
                err.textContent = 'Delete failed: ' + e;
                err.hidden = false;
            });
        });
    }

    /* ── custom content modules ─────────────────────────────────── */

    const LICENSE_OPTIONS = [
        { value: 'unlicensed', label: 'Unlicensed (homebrew)' },
        { value: 'cc-by-4.0', label: 'CC-BY-4.0' },
        { value: 'ogl-1.0a', label: 'OGL 1.0a' },
    ];

    function slugify(name) {
        return String(name).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
    }

    /* Create/edit a custom module. `module` null → create. */
    function openModuleForm(module, onSaved) {
        const mode = module ? 'edit' : 'create';
        const modal = C.openModal('lw-authoring-modal');
        const panel = modal.panel;
        panel.appendChild(C.el('h2', 'lw-modal-title', mode === 'create' ? 'New module' : 'Edit module'));
        const form = C.el('div', 'lw-authoring-form');
        panel.appendChild(form);

        function field(label, control) {
            const w = C.el('label', 'lw-field', label);
            w.appendChild(control);
            form.appendChild(w);
            return control;
        }
        const name = field('Name', C.el('input', 'lw-input'));
        name.type = 'text';
        if (module) name.value = module.name || '';
        const slug = field('Slug', C.el('input', 'lw-input'));
        slug.type = 'text';
        slug.value = module ? module.slug || '' : '';
        // Auto-fill the slug from the name until the user edits it directly.
        let slugTouched = mode === 'edit';
        slug.addEventListener('input', function () { slugTouched = true; });
        name.addEventListener('input', function () {
            if (!slugTouched) slug.value = slugify(name.value);
        });
        if (mode === 'edit') slug.disabled = true; // slug is identity; not editable
        const license = field('License', C.el('select', 'lw-input'));
        LICENSE_OPTIONS.forEach(function (opt) {
            const o = C.el('option', null, opt.label);
            o.value = opt.value;
            license.appendChild(o);
        });
        if (module && module.license) license.value = module.license;
        const desc = field('Description', C.el('textarea', 'lw-input lw-authoring-area'));
        desc.rows = 3;
        if (module && module.description) desc.value = module.description;
        const authors = field('Authors (comma-separated)', C.el('input', 'lw-input'));
        authors.type = 'text';
        if (module && Array.isArray(module.authors)) authors.value = module.authors.join(', ');
        const website = field('Website URL', C.el('input', 'lw-input'));
        website.type = 'url';
        if (module && module.website_url) website.value = module.website_url;

        const generalError = C.el('p', 'lw-form-error');
        generalError.hidden = true;
        form.appendChild(generalError);

        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const save = C.el('button', 'lw-btn lw-btn-filled', mode === 'create' ? 'Create' : 'Save');
        save.type = 'button';
        actions.appendChild(cancel);
        actions.appendChild(save);
        form.appendChild(actions);

        save.addEventListener('click', function () {
            generalError.hidden = true;
            const authorList = authors.value.split(',').map(function (s) { return s.trim(); }).filter(Boolean);
            if (!name.value.trim()) {
                generalError.textContent = 'Name is required.';
                generalError.hidden = false;
                return;
            }
            save.disabled = true;
            const body = mode === 'create'
                ? {
                    name: name.value.trim(),
                    slug: slug.value.trim() || slugify(name.value),
                    license: license.value,
                    description: desc.value.trim() || null,
                    authors: authorList,
                    website_url: website.value.trim() || null,
                }
                : {
                    name: name.value.trim(),
                    license: license.value,
                    description: desc.value.trim() || null,
                    authors: authorList,
                    website_url: website.value.trim() || null,
                };
            const url = mode === 'create' ? '/api/modules/custom' : '/api/modules/' + encodeURIComponent(module.uuid);
            const method = mode === 'create' ? 'POST' : 'PATCH';
            submit(method, url, body).then(function (resp) {
                if (resp.ok) { editableModules(true); modal.close(); onSaved(resp.data); return; }
                save.disabled = false;
                generalError.textContent = (resp.data && resp.data.message) || ('Save failed (' + resp.status + ')');
                generalError.hidden = false;
            }).catch(function (e) {
                save.disabled = false;
                generalError.textContent = 'Save failed: ' + e;
                generalError.hidden = false;
            });
        });
    }

    function deleteModule(uuid, onDeleted) {
        const modal = C.openModal('lw-authoring-modal');
        const panel = modal.panel;
        panel.appendChild(C.el('h2', 'lw-modal-title', 'Delete module'));
        panel.appendChild(C.el('p', 'lw-confirm-text',
            'This permanently removes the module and all its content records. This cannot be undone.'));
        const err = C.el('p', 'lw-form-error');
        err.hidden = true;
        panel.appendChild(err);
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const del = C.el('button', 'lw-btn lw-btn-danger', 'Delete');
        del.type = 'button';
        actions.appendChild(cancel);
        actions.appendChild(del);
        panel.appendChild(actions);
        del.addEventListener('click', function () {
            del.disabled = true;
            fetch('/api/modules/' + encodeURIComponent(uuid), { method: 'DELETE', headers: authHeaders() })
                .then(function (r) {
                    if (r.ok) { editableModules(true); modal.close(); onDeleted(); }
                    else {
                        del.disabled = false;
                        return r.json().catch(function () { return {}; }).then(function (b) {
                            err.textContent = (b && b.message) || ('Delete failed (' + r.status + ')');
                            err.hidden = false;
                        });
                    }
                }).catch(function (e) {
                    del.disabled = false;
                    err.textContent = 'Delete failed: ' + e;
                    err.hidden = false;
                });
        });
    }

    /* Downloads a module as a .lorebundle file (fetch + blob so the
     * Authorization header rides along, unlike a plain link). */
    function exportModule(uuid, slug) {
        return fetch('/api/modules/' + encodeURIComponent(uuid) + '/export', { headers: authHeaders() })
            .then(function (r) {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.blob();
            })
            .then(function (blob) {
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = (slug || 'module') + '.lorebundle';
                document.body.appendChild(a);
                a.click();
                a.remove();
                URL.revokeObjectURL(url);
            });
    }

    /* Imports a .lorebundle file as an uploaded module. Returns a promise
     * resolving to the install response (or rejecting with a message). */
    function importBundleFile(file) {
        return file.text().then(function (text) {
            let bundle;
            try { bundle = JSON.parse(text); } catch (e) { throw new Error('not valid JSON'); }
            if (!bundle.schema || !Array.isArray(bundle.modules)) {
                throw new Error('not a Lorewyld content bundle');
            }
            return fetch('/api/modules/import', {
                method: 'POST',
                headers: jsonHeaders(),
                body: text,
            }).then(function (r) {
                return r.json().catch(function () { return {}; }).then(function (b) {
                    if (!r.ok) throw new Error((b && b.message) || ('HTTP ' + r.status));
                    return b;
                });
            });
        });
    }

    return {
        openCreate: function (category, opts) {
            openForm(category, null, (opts && opts.onSaved) || function () {});
        },
        openEdit: function (category, record, opts) {
            openForm(category, record, (opts && opts.onSaved) || function () {});
        },
        moveModule: moveModule,
        deleteRecord: deleteRecord,
        editableModules: editableModules,
        recordIsEditable: recordIsEditable,
        openCreateModule: function (opts) {
            openModuleForm(null, (opts && opts.onSaved) || function () {});
        },
        openEditModule: function (module, opts) {
            openModuleForm(module, (opts && opts.onSaved) || function () {});
        },
        deleteModule: deleteModule,
        exportModule: exportModule,
        importBundleFile: importBundleFile,
    };
})();
