//! Character pages: list, the 4-step creation wizard, and the sheet.
//! Ports the mobile wizard's mechanical grants (speed from species,
//! hit dice + saves from class, 1st-level HP) and the sheet's
//! documents-but-never-enforces editing model. Every logged-in user
//! sees every character; editing is owner-or-admin (the sheet renders
//! read-only for everyone else).

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::web::auth_ui::LoginRequired;
use crate::web::breadcrumbs::{Breadcrumbs, Crumb};
use crate::web::lore::NOTE_EDITOR_SCRIPT;

/// `/characters` — every character on the server, with owner
/// attribution.
#[component]
pub fn CharactersPage() -> impl IntoView {
    view! {
        <section class="lw-page">
            <Breadcrumbs trail=vec![Crumb::link("Home", "/"), Crumb::here("Characters")]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 class="lw-page-title">"Characters"</h1>
                    <a href="/characters/new" class="lw-btn lw-btn-filled lw-btn-link">"New character"</a>
                </header>
                <ul id="lw-char-list" class="lw-list"></ul>
                <p id="lw-char-status" class="lw-picker-status">"Loading…"</p>
            </div>
            <script inner_html=LIST_SCRIPT></script>
        </section>
    }
}

/// `/characters/new` — vertical stepper: name → species → class →
/// background & alignment. Only the name is required.
#[component]
pub fn CharacterNewPage() -> impl IntoView {
    view! {
        <section class="lw-page">
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Characters", "/characters"),
                Crumb::here("New character"),
            ]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 class="lw-page-title">"New character"</h1>
                </header>
                <div id="lw-wizard" class="lw-stepper"></div>
            </div>
            <script inner_html=WIZARD_SCRIPT></script>
        </section>
    }
}

/// `/characters/:uuid` — the full sheet, section by section.
#[component]
pub fn CharacterSheetPage() -> impl IntoView {
    let params = use_params_map();
    let uuid = move || params.read().get("uuid").unwrap_or_default();
    let initial_uuid = uuid();

    view! {
        <section class="lw-page" data-character-uuid=initial_uuid>
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Characters", "/characters"),
                Crumb::slot("lw-crumb-leaf"),
            ]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 id="lw-sheet-title" class="lw-page-title">"…"</h1>
                    <div class="lw-toolbar">
                        <button id="lw-sheet-save" class="lw-btn lw-btn-filled" type="button" disabled=true>"Save"</button>
                        <button id="lw-sheet-delete" class="lw-btn lw-btn-danger" type="button">"Delete"</button>
                    </div>
                </header>
                <p id="lw-sheet-status" class="lw-picker-status"></p>
                <div id="lw-sheet"></div>
                <header class="lw-page-header">
                    <h2 class="lw-group-header">"Notes"</h2>
                    <button id="lw-char-note-new" class="lw-btn lw-btn-tonal" type="button" hidden=true>
                        "New note"
                    </button>
                </header>
                <ul id="lw-char-note-list" class="lw-list"></ul>
                <p id="lw-char-note-status" class="lw-picker-status"></p>
            </div>
            <script inner_html=NOTE_EDITOR_SCRIPT></script>
            <script inner_html=SHEET_SCRIPT></script>
        </section>
    }
}

const LIST_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const list = document.getElementById('lw-char-list');
    const status = document.getElementById('lw-char-status');

    C.requireAuth(function (me) {
        C.fetchJson('/api/characters').then(function (sheets) {
            list.replaceChildren();
            if (sheets.length === 0) {
                status.textContent = 'No characters on this server yet — create the first one.';
                return;
            }
            status.textContent = '';
            for (const sheet of sheets) {
                const li = C.el('li', 'lw-list-item');
                const a = C.el('a', 'lw-list-item-link');
                a.href = '/characters/' + sheet.uuid;
                const text = C.el('div', 'lw-list-item-text');
                text.appendChild(C.el('div', 'lw-list-item-title', sheet.name));
                const parts = ['Level ' + (sheet.level || 1)];
                if (sheet.race) parts.push(sheet.race);
                if (sheet.class_name) parts.push(sheet.class_name);
                if (sheet.owner_username && (!me || sheet.owner_user_uuid !== me.id)) {
                    parts.push('by ' + sheet.owner_username);
                }
                text.appendChild(C.el('div', 'lw-list-item-subtitle', parts.join(' ')));
                a.appendChild(text);
                li.appendChild(a);
                list.appendChild(li);
            }
        }).catch(function (err) {
            status.textContent = 'Failed to load: ' + err;
        });
    });
})();
"#;

const WIZARD_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const wizard = document.getElementById('lw-wizard');

    const state = {
        step: 0,
        name: '',
        species: null,        // summary record from the picker
        characterClass: null, // FULL record (grants need prof_saving_throws)
        background: null,
        alignment: '',
        creating: false,
    };
    let alignments = [];

    C.requireAuth(function () {
        C.fetchTable('alignment').then(function (records) {
            alignments = records.map(function (r) { return C.humanizeSlug(String(r.name)); });
            render();
        }).catch(function () { render(); });
    });

    function pickerField(label, current, onPick, onClear) {
        const row = C.el('div', 'lw-picker-field');
        const btn = C.el(
            'button',
            'lw-picker-field-btn' + (current ? '' : ' lw-placeholder'),
            current ? current.name : label
        );
        btn.type = 'button';
        btn.addEventListener('click', onPick);
        row.appendChild(btn);
        if (current) {
            const clear = C.el('button', 'lw-picker-field-clear', '✕');
            clear.type = 'button';
            clear.setAttribute('aria-label', 'Clear ' + label);
            clear.addEventListener('click', onClear);
            row.appendChild(clear);
        }
        return row;
    }

    function stepActions(stepIndex, isLast) {
        const actions = C.el('div', 'lw-step-actions');
        if (stepIndex > 0) {
            const back = C.el('button', 'lw-btn lw-btn-text', 'Back');
            back.type = 'button';
            back.addEventListener('click', function () { state.step = stepIndex - 1; render(); });
            actions.appendChild(back);
        }
        const next = C.el('button', 'lw-btn lw-btn-filled', isLast ? 'Create' : 'Next');
        next.type = 'button';
        next.disabled = state.name.trim() === '' || state.creating;
        next.id = isLast ? 'lw-wizard-create' : 'lw-wizard-next-' + stepIndex;
        next.addEventListener('click', function () {
            if (isLast) create();
            else { state.step = stepIndex + 1; render(); }
        });
        actions.appendChild(next);
        return actions;
    }

    function buildStep(index, title, valueLabel, content) {
        const active = state.step === index;
        const step = C.el('div', 'lw-step' + (active ? ' lw-step-active' : state.step > index ? ' lw-step-done' : ''));
        step.appendChild(C.el('span', 'lw-step-num', String(index + 1)));
        const header = C.el('button', 'lw-step-header', title);
        header.type = 'button';
        header.addEventListener('click', function () { state.step = index; render(); });
        if (valueLabel) header.appendChild(C.el('span', 'lw-step-value', valueLabel));
        step.appendChild(header);
        if (active) {
            const body = C.el('div', 'lw-step-content');
            content(body);
            body.appendChild(stepActions(index, index === 3));
            step.appendChild(body);
        }
        return step;
    }

    function render() {
        wizard.replaceChildren();

        wizard.appendChild(buildStep(0, 'Name', state.name || null, function (body) {
            const input = C.el('input', 'lw-input');
            input.type = 'text';
            input.placeholder = 'e.g. Thistle Quickfoot';
            input.value = state.name;
            input.addEventListener('input', function () {
                state.name = input.value;
                // Toggle the action button without re-rendering (focus).
                const next = document.getElementById('lw-wizard-next-0') ||
                    document.getElementById('lw-wizard-create');
                if (next) next.disabled = state.name.trim() === '';
            });
            body.appendChild(input);
            setTimeout(function () { input.focus(); }, 0);
        }));

        wizard.appendChild(buildStep(1, 'Species', state.species && state.species.name, function (body) {
            body.appendChild(pickerField('Choose a species', state.species, function () {
                C.openPicker({ table: 'species', title: 'Choose a species' }).then(function (r) {
                    if (r) { state.species = r; render(); }
                });
            }, function () { state.species = null; render(); }));
        }));

        wizard.appendChild(buildStep(2, 'Class', state.characterClass && state.characterClass.name, function (body) {
            body.appendChild(pickerField('Choose a class', state.characterClass, function () {
                C.openPicker({
                    table: 'class',
                    title: 'Choose a class',
                    recordFilter: function (r) { return r.subclass_of == null; },
                }).then(function (r) {
                    if (!r) return;
                    // Summaries omit prof_saving_throws; grants need the
                    // full record.
                    C.fetchEntry('class', r.uuid).then(function (full) {
                        state.characterClass = full;
                        render();
                    }).catch(function () { state.characterClass = r; render(); });
                });
            }, function () { state.characterClass = null; render(); }));
        }));

        const lastValue = [state.background && state.background.name, state.alignment]
            .filter(Boolean).join(' · ') || null;
        wizard.appendChild(buildStep(3, 'Background & alignment', lastValue, function (body) {
            body.appendChild(pickerField('Choose a background', state.background, function () {
                C.openPicker({ table: 'background', title: 'Choose a background' }).then(function (r) {
                    if (r) { state.background = r; render(); }
                });
            }, function () { state.background = null; render(); }));
            const select = C.el('select', 'lw-input');
            const none = C.el('option', null, 'Alignment (optional)');
            none.value = '';
            select.appendChild(none);
            for (const a of alignments) {
                const opt = C.el('option', null, a);
                opt.value = a;
                select.appendChild(opt);
            }
            select.value = state.alignment;
            select.addEventListener('change', function () { state.alignment = select.value; });
            body.appendChild(select);
        }));
    }

    const ABILITY_KEYS = C.abilityList.map(function (a) { return a.key; });

    function create() {
        const name = state.name.trim();
        if (!name || state.creating) return;
        state.creating = true;
        render();

        // Class grants: saving throws, and 1st-level max HP = hit die
        // maximum + Con modifier (Con starts at 10 → modifier 0).
        // Prefilled, never enforced.
        const cls = state.characterClass;
        const hitDie = cls && typeof cls.hit_dice === 'number' ? Math.trunc(cls.hit_dice) : null;
        const conMod = C.abilityMod(10);
        const startingHp = hitDie != null ? Math.max(1, hitDie + conMod) : 1;
        const saves = cls && Array.isArray(cls.prof_saving_throws)
            ? cls.prof_saving_throws.filter(function (s) { return ABILITY_KEYS.includes(s); })
            : [];

        const sheet = {
            name: name,
            race: state.species ? String(state.species.name) : '',
            class_name: cls ? String(cls.name) : '',
            level: 1,
            background: state.background ? String(state.background.name) : '',
            alignment: state.alignment,
            abilities: {
                strength: 10, dexterity: 10, constitution: 10,
                intelligence: 10, wisdom: 10, charisma: 10,
            },
            saving_throw_proficiencies: saves,
            skill_proficiencies: [],
            armor_class: 10,
            speed: state.species && typeof state.species.speed === 'number'
                ? Math.trunc(state.species.speed) : 30,
            max_hp: startingHp,
            current_hp: startingHp,
            hit_dice: hitDie != null ? '1d' + hitDie : '',
            equipment: [],
            spells: [],
        };

        fetch('/api/characters', {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
            body: JSON.stringify(sheet),
        }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        }).then(function (created) {
            location.href = '/characters/' + created.uuid;
        }).catch(function (err) {
            state.creating = false;
            render();
            alert('Failed to create character: ' + err);
        });
    }
})();
"#;

const SHEET_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const root = document.querySelector('[data-character-uuid]');
    const uuid = root && root.dataset.characterUuid;
    const titleEl = document.getElementById('lw-sheet-title');
    const saveBtn = document.getElementById('lw-sheet-save');
    const deleteBtn = document.getElementById('lw-sheet-delete');
    const statusEl = document.getElementById('lw-sheet-status');
    const sheetEl = document.getElementById('lw-sheet');
    const noteNewBtn = document.getElementById('lw-char-note-new');
    const noteList = document.getElementById('lw-char-note-list');
    const noteStatus = document.getElementById('lw-char-note-status');

    let sheet = null;
    let dirty = false;
    let canEdit = false;

    function markDirty() {
        if (!canEdit) return;
        dirty = true;
        saveBtn.disabled = false;
        statusEl.textContent = 'Unsaved changes';
    }

    window.addEventListener('beforeunload', function (e) {
        if (dirty) e.preventDefault();
    });

    C.requireAuth(function (me) {
        C.fetchJson('/api/characters/' + encodeURIComponent(uuid)).then(function (loaded) {
            sheet = loaded;
            canEdit = !!me && (me.admin || sheet.owner_user_uuid === me.id);
            titleEl.textContent = sheet.name;
            document.title = sheet.name + ' — Lorewyld';
            const crumb = document.getElementById('lw-crumb-leaf');
            if (crumb) crumb.textContent = sheet.name;
            if (!canEdit) {
                saveBtn.hidden = true;
                deleteBtn.hidden = true;
                statusEl.textContent =
                    'Read-only — owned by ' + (sheet.owner_username || 'another user');
            }
            noteNewBtn.hidden = !canEdit;
            render();
        }).catch(function (err) {
            titleEl.textContent = 'Character not found';
            statusEl.textContent = String(err);
        });
        loadNotes();
        noteNewBtn.addEventListener('click', function () {
            window.lwNoteEditor.open({
                scope: { kind: 'character', target_uuid: uuid },
                onSaved: loadNotes,
            });
        });
    });

    function loadNotes() {
        C.fetchJson(
            '/api/lore-notes?scope_kind=character&scope_target=' + encodeURIComponent(uuid)
        ).then(function (notes) {
            noteList.replaceChildren();
            noteStatus.textContent = notes.length === 0 ? 'No notes yet.' : '';
            for (const entry of notes) {
                noteList.appendChild(buildNoteRow(entry));
            }
        }).catch(function (err) {
            noteStatus.textContent = 'Failed to load notes: ' + err;
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

    saveBtn.addEventListener('click', function () {
        saveBtn.disabled = true;
        statusEl.textContent = 'Saving…';
        fetch('/api/characters/' + encodeURIComponent(uuid), {
            method: 'PUT',
            headers: Object.assign({ 'Content-Type': 'application/json' }, window.lw.authHeaders()),
            body: JSON.stringify(sheet),
        }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        }).then(function (saved) {
            sheet = saved;
            dirty = false;
            statusEl.textContent = 'Saved';
            titleEl.textContent = sheet.name;
            render();
        }).catch(function (err) {
            saveBtn.disabled = false;
            statusEl.textContent = 'Save failed: ' + err;
        });
    });

    deleteBtn.addEventListener('click', function () {
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', 'Delete character?'));
        modal.panel.appendChild(C.el('p', null,
            'This permanently deletes "' + sheet.name + '".'));
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const confirm = C.el('button', 'lw-btn lw-btn-danger', 'Delete');
        confirm.type = 'button';
        confirm.addEventListener('click', function () {
            fetch('/api/characters/' + encodeURIComponent(uuid), {
                method: 'DELETE',
                headers: window.lw.authHeaders(),
            }).then(function (r) {
                if (!r.ok && r.status !== 204) throw new Error('HTTP ' + r.status);
                dirty = false;
                location.href = '/characters';
            }).catch(function (err) {
                modal.close();
                statusEl.textContent = 'Delete failed: ' + err;
            });
        });
        actions.appendChild(cancel);
        actions.appendChild(confirm);
        modal.panel.appendChild(actions);
    });

    /* ── widgets ─────────────────────────────────────────────────── */

    function numStepper(value, min, max, step, onChange) {
        const wrap = C.el('span', 'lw-num-stepper');
        const dec = C.el('button', null, '−');
        dec.type = 'button';
        const val = C.el('span', 'lw-num-value', String(value));
        const inc = C.el('button', null, '+');
        inc.type = 'button';
        dec.addEventListener('click', function () {
            const next = Math.max(min, value - step);
            if (next !== value) onChange(next);
        });
        inc.addEventListener('click', function () {
            const next = Math.min(max, value + step);
            if (next !== value) onChange(next);
        });
        wrap.appendChild(dec);
        wrap.appendChild(val);
        wrap.appendChild(inc);
        return wrap;
    }

    function card(title) {
        const c = C.el('div', 'lw-card');
        c.appendChild(C.el('h2', 'lw-card-title', title));
        return c;
    }

    function labeled(label, control) {
        const wrap = C.el('label', 'lw-field', label);
        wrap.appendChild(control);
        return wrap;
    }

    function pickerField(label, current, onPick, onClear) {
        const row = C.el('div', 'lw-picker-field');
        const btn = C.el('button', 'lw-picker-field-btn' + (current ? '' : ' lw-placeholder'),
            current || label);
        btn.type = 'button';
        btn.addEventListener('click', onPick);
        row.appendChild(btn);
        if (current) {
            const clear = C.el('button', 'lw-picker-field-clear', '✕');
            clear.type = 'button';
            clear.addEventListener('click', onClear);
            row.appendChild(clear);
        }
        return row;
    }

    /* ── render ──────────────────────────────────────────────────── */

    function render() {
        sheetEl.replaceChildren();
        renderIdentity();
        renderAbilities();
        renderCombat();
        renderSaves();
        renderSkills();
        renderEquipment();
        renderSpells();
        applyReadOnly();
    }

    // One choke point instead of threading canEdit through every
    // widget builder: viewers get every control disabled.
    function applyReadOnly() {
        if (canEdit) return;
        sheetEl.querySelectorAll('input, select, button, textarea').forEach(function (n) {
            n.disabled = true;
        });
    }

    function renderIdentity() {
        const c = card('Identity');

        const nameInput = C.el('input', 'lw-input');
        nameInput.type = 'text';
        nameInput.value = sheet.name;
        nameInput.addEventListener('input', function () {
            sheet.name = nameInput.value;
            markDirty();
        });
        c.appendChild(labeled('Name', nameInput));

        c.appendChild(labeled('Species', pickerField('Choose a species', sheet.race || null, function () {
            C.openPicker({ table: 'species', title: 'Choose a species' }).then(function (r) {
                if (!r) return;
                sheet.race = String(r.name);
                if (typeof r.speed === 'number') sheet.speed = Math.trunc(r.speed);
                markDirty();
                render();
            });
        }, function () { sheet.race = ''; markDirty(); render(); })));

        c.appendChild(labeled('Class', pickerField('Choose a class', sheet.class_name || null, function () {
            C.openPicker({
                table: 'class',
                title: 'Choose a class',
                recordFilter: function (r) { return r.subclass_of == null; },
            }).then(function (r) {
                if (!r) return;
                C.fetchEntry('class', r.uuid).catch(function () { return r; }).then(function (full) {
                    sheet.class_name = String(full.name);
                    const die = typeof full.hit_dice === 'number' ? Math.trunc(full.hit_dice) : null;
                    if (die != null) sheet.hit_dice = '1d' + die;
                    if (Array.isArray(full.prof_saving_throws)) {
                        const keys = C.abilityList.map(function (a) { return a.key; });
                        sheet.saving_throw_proficiencies =
                            full.prof_saving_throws.filter(function (s) { return keys.includes(s); });
                    }
                    // HP re-derives only while untouched (maxHp <= 1):
                    // documents, never enforces.
                    if (die != null && sheet.max_hp <= 1) {
                        const hp = Math.max(1, die + C.abilityMod(sheet.abilities.constitution));
                        sheet.max_hp = hp;
                        sheet.current_hp = hp;
                    }
                    markDirty();
                    render();
                });
            });
        }, function () { sheet.class_name = ''; markDirty(); render(); })));

        c.appendChild(labeled('Background', pickerField('Choose a background', sheet.background || null, function () {
            C.openPicker({ table: 'background', title: 'Choose a background' }).then(function (r) {
                if (!r) return;
                sheet.background = String(r.name);
                markDirty();
                render();
            });
        }, function () { sheet.background = ''; markDirty(); render(); })));

        const levelRow = C.el('div', 'lw-combat-grid');
        const levelItem = C.el('div', 'lw-combat-item', 'Level');
        levelItem.appendChild(numStepper(sheet.level || 1, 1, 20, 1, function (v) {
            sheet.level = v;
            markDirty();
            render();
        }));
        levelRow.appendChild(levelItem);
        const prof = C.el('div', 'lw-stat-badge', 'Proficiency');
        prof.appendChild(C.el('span', 'lw-stat-value', C.formatBonus(C.proficiencyBonus(sheet.level || 1))));
        levelRow.appendChild(prof);
        c.appendChild(levelRow);

        const select = C.el('select', 'lw-input');
        const none = C.el('option', null, 'No alignment');
        none.value = '';
        select.appendChild(none);
        C.fetchTable('alignment').then(function (records) {
            const names = records.map(function (r) { return C.humanizeSlug(String(r.name)); });
            // A saved free-text value stays selectable.
            if (sheet.alignment && !names.includes(sheet.alignment)) names.unshift(sheet.alignment);
            for (const n of names) {
                const opt = C.el('option', null, n);
                opt.value = n;
                select.appendChild(opt);
            }
            select.value = sheet.alignment || '';
        });
        select.addEventListener('change', function () {
            sheet.alignment = select.value;
            markDirty();
        });
        c.appendChild(labeled('Alignment', select));

        sheetEl.appendChild(c);
    }

    function renderAbilities() {
        const c = card('Abilities');
        const grid = C.el('div', 'lw-ability-grid');
        for (const ability of C.abilityList) {
            const tile = C.el('div', 'lw-ability-tile');
            tile.appendChild(C.el('span', 'lw-ability-name', ability.abbr));
            tile.appendChild(numStepper(sheet.abilities[ability.key], 1, 30, 1, function (v) {
                sheet.abilities[ability.key] = v;
                markDirty();
                render();
            }));
            tile.appendChild(C.el('span', 'lw-ability-mod',
                C.formatBonus(C.abilityMod(sheet.abilities[ability.key]))));
            grid.appendChild(tile);
        }
        c.appendChild(grid);
        sheetEl.appendChild(c);
    }

    function renderCombat() {
        const c = card('Combat');
        const grid = C.el('div', 'lw-combat-grid');

        function statStepper(label, value, min, max, step, onChange) {
            const item = C.el('div', 'lw-combat-item', label);
            item.appendChild(numStepper(value, min, max, step, onChange));
            return item;
        }
        function statBadge(label, value) {
            const badge = C.el('div', 'lw-stat-badge', label);
            badge.appendChild(C.el('span', 'lw-stat-value', value));
            return badge;
        }

        grid.appendChild(statStepper('Armor class', sheet.armor_class, 0, 40, 1, function (v) {
            sheet.armor_class = v; markDirty(); render();
        }));
        grid.appendChild(statBadge('Initiative', C.formatBonus(C.abilityMod(sheet.abilities.dexterity))));
        grid.appendChild(statStepper('Speed', sheet.speed, 0, 200, 5, function (v) {
            sheet.speed = v; markDirty(); render();
        }));
        const perceptionProficient = sheet.skill_proficiencies.includes('perception');
        const passive = 10 + C.abilityMod(sheet.abilities.wisdom) +
            (perceptionProficient ? C.proficiencyBonus(sheet.level || 1) : 0);
        grid.appendChild(statBadge('Passive perception', String(passive)));
        grid.appendChild(statStepper('Current HP', sheet.current_hp, 0, 999, 1, function (v) {
            sheet.current_hp = v; markDirty(); render();
        }));
        grid.appendChild(statStepper('Max HP', sheet.max_hp, 1, 999, 1, function (v) {
            sheet.max_hp = v; markDirty(); render();
        }));

        const hitDice = C.el('div', 'lw-combat-item', 'Hit dice');
        const hdInput = C.el('input', 'lw-input');
        hdInput.type = 'text';
        hdInput.value = sheet.hit_dice || '';
        hdInput.style.width = '90px';
        hdInput.addEventListener('input', function () {
            sheet.hit_dice = hdInput.value;
            markDirty();
        });
        hitDice.appendChild(hdInput);
        grid.appendChild(hitDice);

        c.appendChild(grid);
        sheetEl.appendChild(c);
    }

    function checkRow(label, suffix, checked, bonus, onToggle) {
        const row = C.el('label', 'lw-check-row');
        const box = C.el('input');
        box.type = 'checkbox';
        box.checked = checked;
        box.addEventListener('change', function () { onToggle(box.checked); });
        row.appendChild(box);
        row.appendChild(C.el('span', null, label + (suffix ? ' (' + suffix + ')' : '')));
        row.appendChild(C.el('span', 'lw-check-bonus', bonus));
        return row;
    }

    function renderSaves() {
        const c = card('Saving throws');
        const profBonus = C.proficiencyBonus(sheet.level || 1);
        for (const ability of C.abilityList) {
            const proficient = sheet.saving_throw_proficiencies.includes(ability.key);
            const bonus = C.abilityMod(sheet.abilities[ability.key]) + (proficient ? profBonus : 0);
            c.appendChild(checkRow(ability.label, null, proficient, C.formatBonus(bonus), function (on) {
                sheet.saving_throw_proficiencies = on
                    ? sheet.saving_throw_proficiencies.concat([ability.key])
                    : sheet.saving_throw_proficiencies.filter(function (k) { return k !== ability.key; });
                markDirty();
                render();
            }));
        }
        sheetEl.appendChild(c);
    }

    function renderSkills() {
        const c = card('Skills');
        const profBonus = C.proficiencyBonus(sheet.level || 1);
        for (const skill of C.skillList) {
            const ability = C.abilityList.find(function (a) { return a.key === skill.ability; });
            const proficient = sheet.skill_proficiencies.includes(skill.key);
            const bonus = C.abilityMod(sheet.abilities[skill.ability]) + (proficient ? profBonus : 0);
            c.appendChild(checkRow(skill.label, ability.abbr, proficient, C.formatBonus(bonus), function (on) {
                sheet.skill_proficiencies = on
                    ? sheet.skill_proficiencies.concat([skill.key])
                    : sheet.skill_proficiencies.filter(function (k) { return k !== skill.key; });
                markDirty();
                render();
            }));
        }
        sheetEl.appendChild(c);
    }

    function renderEquipment() {
        const c = card('Equipment');
        const list = C.el('ul', 'lw-list');
        sheet.equipment.forEach(function (item, index) {
            const li = C.el('li', 'lw-list-item');
            const row = C.el('div', 'lw-list-item-link');
            const text = C.el('div', 'lw-list-item-text');
            text.appendChild(C.el('div', 'lw-list-item-title',
                item.name + (item.quantity > 1 ? ' ×' + item.quantity : '')));
            if (item.notes) text.appendChild(C.el('div', 'lw-list-item-subtitle', item.notes));
            row.appendChild(text);
            const remove = C.el('button', 'lw-picker-field-clear', '✕');
            remove.type = 'button';
            remove.setAttribute('aria-label', 'Remove ' + item.name);
            remove.addEventListener('click', function () {
                sheet.equipment.splice(index, 1);
                markDirty();
                render();
            });
            row.appendChild(remove);
            li.appendChild(row);
            list.appendChild(li);
        });
        c.appendChild(list);

        const add = C.el('button', 'lw-btn lw-btn-text', '+ Add item');
        add.type = 'button';
        add.addEventListener('click', function () {
            C.openPicker({ table: 'item', title: 'Add equipment' }).then(function (r) {
                if (!r) return;
                quantityDialog(String(r.name), function (quantity, notes) {
                    sheet.equipment.push({ name: String(r.name), quantity: quantity, notes: notes });
                    markDirty();
                    render();
                });
            });
        });
        c.appendChild(add);
        sheetEl.appendChild(c);
    }

    function quantityDialog(name, onDone) {
        const modal = C.openModal('');
        modal.panel.appendChild(C.el('h2', 'lw-modal-title', name));
        const qty = C.el('input', 'lw-input');
        qty.type = 'number';
        qty.min = '1';
        qty.value = '1';
        modal.panel.appendChild(labeled('Quantity', qty));
        const notes = C.el('input', 'lw-input');
        notes.type = 'text';
        modal.panel.appendChild(labeled('Notes', notes));
        const actions = C.el('div', 'lw-modal-actions');
        const cancel = C.el('button', 'lw-btn lw-btn-text', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', modal.close);
        const ok = C.el('button', 'lw-btn lw-btn-filled', 'Add');
        ok.type = 'button';
        ok.addEventListener('click', function () {
            const n = parseInt(qty.value, 10);
            modal.close();
            onDone(Number.isFinite(n) && n > 0 ? n : 1, notes.value.trim());
        });
        actions.appendChild(cancel);
        actions.appendChild(ok);
        modal.panel.appendChild(actions);
    }

    function renderSpells() {
        const c = card('Spells');
        const sorted = sheet.spells.slice().sort(function (a, b) {
            const byLevel = (a.level || 0) - (b.level || 0);
            return byLevel !== 0 ? byLevel : String(a.name).localeCompare(String(b.name));
        });
        const list = C.el('ul', 'lw-list');
        sorted.forEach(function (spell) {
            const li = C.el('li', 'lw-list-item');
            const row = C.el('div', 'lw-list-item-link');
            const text = C.el('div', 'lw-list-item-text');
            text.appendChild(C.el('div', 'lw-list-item-title', spell.name));
            text.appendChild(C.el('div', 'lw-list-item-subtitle',
                spell.level === 0 ? 'Cantrip' : 'Level ' + spell.level));
            row.appendChild(text);
            const remove = C.el('button', 'lw-picker-field-clear', '✕');
            remove.type = 'button';
            remove.setAttribute('aria-label', 'Remove ' + spell.name);
            remove.addEventListener('click', function () {
                const index = sheet.spells.indexOf(spell);
                if (index >= 0) sheet.spells.splice(index, 1);
                markDirty();
                render();
            });
            row.appendChild(remove);
            li.appendChild(row);
            list.appendChild(li);
        });
        c.appendChild(list);

        const add = C.el('button', 'lw-btn lw-btn-text', '+ Add spell');
        add.type = 'button';
        add.addEventListener('click', function () {
            C.openPicker({ table: 'spell', title: 'Add a spell' }).then(function (r) {
                if (!r) return;
                sheet.spells.push({
                    name: String(r.name),
                    level: typeof r.level === 'number' ? r.level : 0,
                    notes: '',
                });
                markDirty();
                render();
            });
        });
        c.appendChild(add);
        sheetEl.appendChild(c);
    }
})();
"#;
