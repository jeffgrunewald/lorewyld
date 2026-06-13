//! Compendium pages: landing grid with global search, per-category
//! listings with the shared filter/sort engine, and entry detail.
//! Mirrors the mobile compendium screens; content endpoints require a
//! session, so every page sits behind the `lwContent.requireAuth` gate.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::web::auth_ui::LoginRequired;
use crate::web::breadcrumbs::{Breadcrumbs, Crumb};

/// `/compendium` — category tiles with counts plus a global search
/// across every category.
#[component]
pub fn CompendiumPage() -> impl IntoView {
    view! {
        <section class="lw-page">
            <Breadcrumbs trail=vec![Crumb::link("Home", "/"), Crumb::here("Compendium")]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 class="lw-page-title">"Compendium"</h1>
                </header>
                <div class="lw-search-row">
                    <input
                        id="lw-comp-search"
                        class="lw-input"
                        type="search"
                        placeholder="Search the compendium…"
                    />
                </div>
                <ul id="lw-comp-tiles" class="lw-tile-grid"></ul>
                <div id="lw-comp-results" hidden=true></div>
            </div>
            <script inner_html=COMPENDIUM_SCRIPT></script>
        </section>
    }
}

/// `/compendium/:category` — one category, searched/filtered/sorted
/// entirely in memory like the mobile category screen.
#[component]
pub fn CompendiumCategoryPage() -> impl IntoView {
    let params = use_params_map();
    let category = move || params.read().get("category").unwrap_or_default();
    let initial = category();

    view! {
        <section class="lw-page" data-category=initial>
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Compendium", "/compendium"),
                Crumb::slot("lw-crumb-leaf"),
            ]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <h1 id="lw-cat-title" class="lw-page-title">"…"</h1>
                </header>
                <div class="lw-search-row">
                    <input id="lw-cat-search" class="lw-input" type="search" placeholder="Search…"/>
                    <button id="lw-cat-filter" class="lw-filter-btn" type="button" aria-label="Filter & sort">"☰"</button>
                </div>
                <p id="lw-cat-count" class="lw-count-line"></p>
                <ul id="lw-cat-list" class="lw-list"></ul>
                <p id="lw-cat-status" class="lw-picker-status">"Loading…"</p>
            </div>
            <script inner_html=CATEGORY_SCRIPT></script>
        </section>
    }
}

/// `/compendium/:category/:uuid` — entry detail: facts card, markdown
/// body, named sections, then a deliberately quiet source footer
/// (provenance is planning metadata, not in-play reference).
#[component]
pub fn CompendiumEntryPage() -> impl IntoView {
    let params = use_params_map();
    let category = move || params.read().get("category").unwrap_or_default();
    let uuid = move || params.read().get("uuid").unwrap_or_default();
    let (initial_category, initial_uuid) = (category(), uuid());
    let category_href = format!("/compendium/{initial_category}");

    view! {
        <section class="lw-page" data-category=initial_category data-entry-uuid=initial_uuid>
            <Breadcrumbs trail=vec![
                Crumb::link("Home", "/"),
                Crumb::link("Compendium", "/compendium"),
                Crumb::link_slot("lw-crumb-category", category_href),
                Crumb::slot("lw-crumb-leaf"),
            ]/>
            <LoginRequired/>
            <div id="lw-page-root" hidden=true>
                <header class="lw-page-header">
                    <div>
                        <h1 id="lw-entry-title" class="lw-page-title">"…"</h1>
                        <p id="lw-entry-subtitle" class="lw-page-subtitle"></p>
                    </div>
                </header>
                <div id="lw-entry-facts" class="lw-card" hidden=true></div>
                <div id="lw-entry-body" class="lw-md"></div>
                <div id="lw-entry-sections"></div>
                <p id="lw-entry-source" class="lw-source-footer" hidden=true></p>
            </div>
            <script inner_html=ENTRY_SCRIPT></script>
        </section>
    }
}

const COMPENDIUM_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const tiles = document.getElementById('lw-comp-tiles');
    const results = document.getElementById('lw-comp-results');
    const search = document.getElementById('lw-comp-search');

    C.requireAuth(function () {
        Promise.all([C.loadLookups(), C.fetchJson('/api/content/counts')])
            .then(function (loaded) { init(loaded[0], loaded[1].counts || []); })
            .catch(function (err) {
                tiles.replaceChildren(C.el('li', 'lw-modules-error', 'Failed to load: ' + err));
            });
    });

    function init(lookups, counts) {
        const countBy = {};
        for (const c of counts) countBy[c.category] = c.count;

        tiles.replaceChildren();
        for (const cat of C.categories) {
            const li = C.el('li');
            const a = C.el('a', 'lw-tile');
            a.href = '/compendium/' + cat.table;
            a.appendChild(C.el('span', 'lw-tile-glyph', cat.glyph));
            a.appendChild(C.el('span', 'lw-tile-label', cat.label));
            const n = countBy[cat.table] || 0;
            a.appendChild(C.el('span', 'lw-tile-count', n.toLocaleString() + ' entries'));
            li.appendChild(a);
            tiles.appendChild(li);
        }

        // Debounced, sequence-guarded global search across categories.
        let seq = 0;
        let timer = null;
        search.addEventListener('input', function () {
            clearTimeout(timer);
            timer = setTimeout(runSearch, 250);
        });

        function runSearch() {
            const q = search.value.trim();
            const mySeq = ++seq;
            if (!q) {
                results.hidden = true;
                tiles.hidden = false;
                return;
            }
            Promise.all(C.categories.map(function (cat) {
                return C.fetchJson(
                    '/api/content/' + cat.table + '?q=' + encodeURIComponent(q) + '&limit=25'
                ).then(function (records) { return { cat: cat, records: records }; });
            })).then(function (groups) {
                if (mySeq !== seq) return; // a newer query superseded this one
                results.replaceChildren();
                let any = false;
                for (const group of groups) {
                    if (group.records.length === 0) continue;
                    any = true;
                    results.appendChild(C.el('h2', 'lw-group-header', group.cat.label));
                    const list = C.el('ul', 'lw-list');
                    for (const record of group.records) {
                        list.appendChild(C.buildEntryRow(
                            record, group.cat, lookups,
                            '/compendium/' + group.cat.table + '/' + record.uuid
                        ));
                    }
                    results.appendChild(list);
                }
                if (!any) results.appendChild(C.el('p', 'lw-picker-status', 'No matches.'));
                tiles.hidden = true;
                results.hidden = false;
            }).catch(function () { /* superseded or transient; keep current view */ });
        }
    }
})();
"#;

const CATEGORY_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const root = document.querySelector('[data-category]');
    const table = root && root.dataset.category;
    const title = document.getElementById('lw-cat-title');
    const search = document.getElementById('lw-cat-search');
    const filterBtn = document.getElementById('lw-cat-filter');
    const countLine = document.getElementById('lw-cat-count');
    const list = document.getElementById('lw-cat-list');
    const status = document.getElementById('lw-cat-status');

    const category = C.categoryFor(table);
    if (!category) {
        title.textContent = 'Unknown category';
        status.textContent = '';
        document.getElementById('lw-page-root').hidden = false;
        return;
    }
    title.textContent = category.label;
    document.title = category.label + ' — Lorewyld';
    const crumb = document.getElementById('lw-crumb-leaf');
    if (crumb) crumb.textContent = category.label;

    const dimensions = C.filterDimensionsFor(table);
    const sorts = C.sortOptionsFor(table);
    const state = C.newFilterState(sorts);

    C.requireAuth(function () {
        Promise.all([C.loadLookups(), C.fetchTable(table)])
            .then(function (loaded) { init(loaded[0], loaded[1]); })
            .catch(function (err) { status.textContent = 'Failed to load: ' + err; });
    });

    function init(lookups, records) {
        function render() {
            const visible = C.visibleRecords(records, category, search.value, dimensions, state, lookups);
            list.replaceChildren();
            for (const record of visible) {
                list.appendChild(C.buildEntryRow(
                    record, category, lookups,
                    '/compendium/' + table + '/' + record.uuid
                ));
            }
            countLine.textContent =
                visible.length.toLocaleString() + ' of ' + records.length.toLocaleString();
            status.textContent = visible.length === 0 ? 'No matches.' : '';
            C.decorateFilterButton(filterBtn, state);
        }

        search.addEventListener('input', render);
        filterBtn.addEventListener('click', function () {
            C.openFilterPanel({
                dimensions: dimensions,
                sorts: sorts,
                lookups: lookups,
                records: records,
                state: state,
                onChanged: render,
            });
        });
        render();
    }
})();
"#;

const ENTRY_SCRIPT: &str = r#"
(function () {
    const C = window.lwContent;
    const root = document.querySelector('[data-entry-uuid]');
    const table = root && root.dataset.category;
    const uuid = root && root.dataset.entryUuid;
    const titleEl = document.getElementById('lw-entry-title');
    const subtitleEl = document.getElementById('lw-entry-subtitle');
    const factsEl = document.getElementById('lw-entry-facts');
    const bodyEl = document.getElementById('lw-entry-body');
    const sectionsEl = document.getElementById('lw-entry-sections');
    const sourceEl = document.getElementById('lw-entry-source');

    const category = C.categoryFor(table);
    if (!category || !uuid) {
        titleEl.textContent = 'Not found';
        return;
    }

    C.requireAuth(function () {
        Promise.all([C.loadLookups(), C.fetchEntry(table, uuid)])
            .then(function (loaded) { render(loaded[0], loaded[1]); })
            .catch(function () { titleEl.textContent = 'Entry not found'; });
    });

    function spellComponents(r) {
        const parts = [];
        if (r.verbal) parts.push('V');
        if (r.somatic) parts.push('S');
        if (r.material) {
            parts.push(
                typeof r.material_specified === 'string' && r.material_specified
                    ? 'M (' + r.material_specified + ')'
                    : 'M'
            );
        }
        return parts.length ? parts.join(', ') : 'None';
    }

    function speeds(r) {
        if (!r.speed || typeof r.speed !== 'object') return '';
        return Object.entries(r.speed)
            .filter(function (e) { return typeof e[1] === 'number' && e[1] > 0; })
            .map(function (e) { return e[0] + ' ' + Math.trunc(e[1]) + ' ft.'; })
            .join(', ');
    }

    function abilityLine(scores) {
        return ['strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma']
            .filter(function (k) { return typeof scores[k] === 'number'; })
            .map(function (k) { return k.slice(0, 3).toUpperCase() + ' ' + Math.trunc(scores[k]); })
            .join(' · ');
    }

    function weaponProperties(r, lookups) {
        return (Array.isArray(r.properties) ? r.properties : [])
            .filter(function (p) { return p && typeof p.property_uuid === 'string'; })
            .map(function (p) {
                const name = lookups.weaponProperties[p.property_uuid] || 'Unknown';
                return typeof p.detail === 'string' && p.detail ? name + ' (' + p.detail + ')' : name;
            })
            .join(', ');
    }

    function facts(r, lookups) {
        const out = [];
        const push = function (label, value) {
            if (value !== null && value !== undefined && value !== '') out.push([label, String(value)]);
        };
        switch (table) {
            case 'spell':
                if (typeof r.casting_time === 'string') push('Casting time', C.humanizeSlug(r.casting_time));
                if (typeof r.range_text === 'string') push('Range', r.range_text);
                if (typeof r.duration === 'string') push('Duration', C.humanizeSlug(r.duration));
                push('Components', spellComponents(r));
                if (r.concentration === true) push('Concentration', 'Yes');
                if (r.ritual === true) push('Ritual', 'Yes');
                break;
            case 'creature':
                if (typeof r.armor_class === 'number') {
                    push('Armor class', Math.trunc(r.armor_class) +
                        (typeof r.armor_detail === 'string' && r.armor_detail ? ' (' + r.armor_detail + ')' : ''));
                }
                if (typeof r.hit_points === 'number') {
                    push('Hit points', Math.trunc(r.hit_points) +
                        (typeof r.hit_dice === 'string' ? ' (' + r.hit_dice + ')' : ''));
                }
                push('Speed', speeds(r));
                if (r.ability_scores && typeof r.ability_scores === 'object') {
                    push('Abilities', abilityLine(r.ability_scores));
                }
                if (typeof r.experience_points === 'number') push('XP', Math.trunc(r.experience_points));
                if (typeof r.languages === 'string') push('Languages', r.languages);
                break;
            case 'class':
                if (r.hit_dice != null) push('Hit die', 'd' + r.hit_dice);
                if (typeof r.prof_saving_throws === 'string') push('Saving throws', r.prof_saving_throws);
                else if (Array.isArray(r.prof_saving_throws)) {
                    push('Saving throws', r.prof_saving_throws.map(C.humanizeSlug).join(', '));
                }
                if (typeof r.prof_armor === 'string') push('Armor', r.prof_armor);
                if (typeof r.prof_weapons === 'string') push('Weapons', r.prof_weapons);
                if (typeof r.prof_skills === 'string') push('Skills', r.prof_skills);
                break;
            case 'species':
                push('Size', lookups.nameOf(lookups.sizes, r.size));
                if (typeof r.speed === 'number') push('Speed', Math.trunc(r.speed) + ' ft.');
                if (typeof r.asi_desc === 'string') push('Ability scores', r.asi_desc);
                break;
            case 'feat':
                if (r.has_prerequisite === true) push('Prerequisite', r.prerequisite);
                break;
            case 'item':
                push('Category', lookups.nameOf(lookups.itemCategories, r.category_uuid));
                if (typeof r.cost === 'string') push('Cost', r.cost + ' gp');
                if (typeof r.weight === 'number' || typeof r.weight === 'string') push('Weight', r.weight + ' lb.');
                if (r.is_magic === true && typeof r.rarity === 'string') push('Rarity', C.humanizeSlug(r.rarity));
                if (r.requires_attunement === true) push('Attunement', 'Required');
                break;
            case 'weapon':
                push('Category', r.is_simple === true ? 'Simple' : 'Martial');
                if (typeof r.damage_dice === 'string') {
                    push('Damage', (r.damage_dice + ' ' + (r.damage_type || '')).trim());
                }
                push('Properties', weaponProperties(r, lookups));
                break;
            case 'armor':
                if (typeof r.category === 'string') push('Category', C.humanizeSlug(r.category));
                if (typeof r.ac_display === 'string') push('Armor class', r.ac_display);
                if (r.grants_stealth_disadvantage === true) push('Stealth', 'Disadvantage');
                break;
        }
        return out;
    }

    function namedList(value) {
        return (Array.isArray(value) ? value : [])
            .filter(function (e) { return e && typeof e.name === 'string' && typeof e.desc === 'string'; })
            .map(function (e) { return [e.name, e.desc]; });
    }

    function sections(r) {
        const bySection = {
            species: [['Traits', namedList(r.traits)]],
            background: [['Benefits', namedList(r.benefits)]],
            feat: [['Benefits', namedList(r.benefits)]],
            class: [['Features', namedList(r.features)]],
            creature: [['Actions', namedList(r.actions)]],
        };
        return (bySection[table] || []).filter(function (s) { return s[1].length > 0; });
    }

    function description(r) {
        if (table === 'spell') {
            const parts = [r.description || ''];
            if (typeof r.higher_level === 'string' && r.higher_level) {
                parts.push('**At higher levels.** ' + r.higher_level);
            }
            return parts.join('\n\n');
        }
        return typeof r.desc === 'string' ? r.desc : null;
    }

    function render(lookups, record) {
        const name = category.displayName(record);
        titleEl.textContent = name;
        document.title = name + ' — Lorewyld';
        const categoryCrumb = document.getElementById('lw-crumb-category');
        if (categoryCrumb) categoryCrumb.textContent = category.label;
        const leafCrumb = document.getElementById('lw-crumb-leaf');
        if (leafCrumb) leafCrumb.textContent = name;
        subtitleEl.textContent = category.subtitle(record, lookups) || '';

        const factRows = facts(record, lookups);
        if (factRows.length > 0) {
            factsEl.hidden = false;
            const wrap = C.el('div', 'lw-facts');
            for (const fact of factRows) {
                const row = C.el('div', 'lw-fact-row');
                row.appendChild(C.el('span', 'lw-fact-label', fact[0]));
                row.appendChild(C.el('span', 'lw-fact-value', fact[1]));
                wrap.appendChild(row);
            }
            factsEl.replaceChildren(wrap);
        }

        const desc = description(record);
        if (desc) bodyEl.appendChild(C.renderMarkdown(desc));

        for (const section of sections(record)) {
            sectionsEl.appendChild(C.el('h2', 'lw-group-header', section[0]));
            for (const entry of section[1]) {
                const card = C.el('div', 'lw-card');
                card.appendChild(C.el('h3', 'lw-card-title', entry[0]));
                const md = C.el('div', 'lw-md');
                md.appendChild(C.renderMarkdown(entry[1]));
                card.appendChild(md);
                sectionsEl.appendChild(card);
            }
        }

        // Provenance trails the playable content — valuable when
        // planning, not during a session.
        const source = lookups.sourceNameOf(record);
        if (source) {
            sourceEl.hidden = false;
            sourceEl.textContent = 'Source: ' + source;
        }
    }
})();
"#;
