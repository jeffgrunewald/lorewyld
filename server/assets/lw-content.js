/* Shared content engine for the Lorewyld web app.
 *
 * Ports the mobile app's compendium logic 1:1 (categories.dart,
 * filters.dart, filter_sheet.dart, content_picker.dart) so listings,
 * filters, sorts, and pickers behave identically on both clients.
 *
 * Loaded as a blocking <head> script: inline page scripts reference
 * window.lwContent at top level. All DOM construction goes through
 * createElement/textContent — never innerHTML with data — so content
 * text can't escape into markup.
 */
window.lwContent = (function () {
    'use strict';

    /* ── helpers ────────────────────────────────────────────────── */

    function el(tag, className, text) {
        const node = document.createElement(tag);
        if (className) node.className = className;
        if (text !== undefined && text !== null) node.textContent = text;
        return node;
    }

    function humanizeSlug(s) {
        return String(s)
            .split(/[_-]/)
            .filter(function (w) { return w.length > 0; })
            .map(function (w) { return w[0].toUpperCase() + w.slice(1); })
            .join(' ');
    }

    function spellLevelLabel(level) {
        return level === 0 ? 'Cantrip' : 'Level ' + level;
    }

    /* Human plural for a content table name ("class" → "classes",
     * "armor"/"species" unchanged) for count chips. */
    function categoryPlural(category) {
        const name = humanizeSlug(category).toLowerCase();
        if (name === 'class') return 'classes';
        if (name === 'species' || name === 'armor') return name;
        return name + 's';
    }

    function formatChallengeRating(cr) {
        if (cr === 0.125) return '1/8';
        if (cr === 0.25) return '1/4';
        if (cr === 0.5) return '1/2';
        return Number.isInteger(cr) ? String(cr) : String(cr);
    }

    function authHeaders() {
        return window.lw && window.lw.authHeaders ? window.lw.authHeaders() : {};
    }

    function fetchJson(url) {
        return fetch(url, { headers: authHeaders() }).then(function (r) {
            if (!r.ok) throw new Error('HTTP ' + r.status);
            return r.json();
        });
    }

    /* ── data access (memoized per page load) ───────────────────── */

    const tableCache = new Map();

    function fetchTable(table) {
        if (!tableCache.has(table)) {
            const promise = fetchJson('/api/content/' + encodeURIComponent(table))
                .catch(function (err) {
                    tableCache.delete(table);
                    throw err;
                });
            tableCache.set(table, promise);
        }
        return tableCache.get(table);
    }

    function fetchEntry(table, uuid) {
        return fetchJson(
            '/api/content/' + encodeURIComponent(table) + '/' + encodeURIComponent(uuid)
        );
    }

    /* uuid → name maps for the lookup tables referenced by major
     * content. Schools and creature types carry lowercase wire names
     * ("evocation") and are title-cased once here; class/species names
     * are display-ready and must NOT be re-split ("Half-Elf"). */
    let lookupsPromise = null;

    function loadLookups() {
        if (lookupsPromise) return lookupsPromise;
        lookupsPromise = Promise.all([
            fetchTable('spell_school'),
            fetchTable('size'),
            fetchTable('creature_type'),
            fetchTable('item_category'),
            fetchTable('class'),
            fetchTable('species'),
            fetchTable('weapon_property'),
            fetchTable('document'),
            fetchJson('/api/modules'),
        ]).then(function (results) {
            function nameMap(records, mapName) {
                const out = {};
                for (const r of records) out[r.uuid] = mapName ? humanizeSlug(r.name) : String(r.name);
                return out;
            }
            const sizes = results[1];
            const documents = results[7];
            const modules = results[8];
            const lookups = {
                spellSchools: nameMap(results[0], true),
                sizes: nameMap(sizes),
                creatureTypes: nameMap(results[2], true),
                itemCategories: nameMap(results[3]),
                classes: nameMap(results[4]),
                species: nameMap(results[5]),
                weaponProperties: nameMap(results[6]),
                documents: nameMap(documents),
                contentModules: nameMap(modules),
                sizeRanks: {},
                documentKeys: {},
                moduleSlugs: {},
            };
            for (const s of sizes) {
                if (typeof s.rank === 'number') lookups.sizeRanks[s.uuid] = s.rank;
            }
            for (const d of documents) lookups.documentKeys[d.uuid] = String(d.key);
            for (const m of modules) lookups.moduleSlugs[m.uuid] = String(m.slug);
            lookups.nameOf = function (table, uuid) {
                return typeof uuid === 'string' ? table[uuid] || null : null;
            };
            /* Abbreviated source label: the record's document key, or
             * its module slug for tables with no document reference. */
            lookups.sourceSlugOf = function (record) {
                return (
                    lookups.documentKeys[record.document_uuid] ||
                    lookups.moduleSlugs[record.content_module_uuid] ||
                    null
                );
            };
            lookups.sourceNameOf = function (record) {
                return (
                    lookups.documents[record.document_uuid] ||
                    lookups.contentModules[record.content_module_uuid] ||
                    null
                );
            };
            return lookups;
        }).catch(function (err) {
            lookupsPromise = null;
            throw err;
        });
        return lookupsPromise;
    }

    /* ── category descriptors (port of categories.dart) ─────────── */

    function joinParts(parts) {
        const filtered = parts.filter(function (p) { return p !== null && p !== undefined && p !== ''; });
        return filtered.length ? filtered.join(' · ') : null;
    }

    const categories = [
        {
            table: 'spell',
            label: 'Spells',
            glyph: '✶',
            subtitle: function (r, l) {
                return joinParts([
                    spellLevelLabel(typeof r.level === 'number' ? r.level : 0),
                    l.nameOf(l.spellSchools, r.school),
                ]);
            },
        },
        {
            table: 'creature',
            label: 'Creatures',
            glyph: '🐉',
            subtitle: function (r, l) {
                return joinParts([
                    typeof r.challenge_rating === 'number'
                        ? 'CR ' + formatChallengeRating(r.challenge_rating)
                        : null,
                    l.nameOf(l.sizes, r.size),
                    l.nameOf(l.creatureTypes, r.type),
                ]);
            },
        },
        {
            table: 'class',
            label: 'Classes & subclasses',
            glyph: '🛡',
            subtitle: function (r, l) {
                if (typeof r.subclass_of === 'string') {
                    return 'Subclass of ' + (l.classes[r.subclass_of] || 'unknown');
                }
                return r.hit_dice != null ? 'Hit die d' + r.hit_dice : null;
            },
        },
        {
            table: 'species',
            label: 'Species',
            glyph: '🧝',
            subtitle: function (r, l) {
                if (typeof r.subspecies_of === 'string') {
                    return 'Subspecies of ' + (l.species[r.subspecies_of] || 'unknown');
                }
                return l.nameOf(l.sizes, r.size);
            },
        },
        {
            table: 'background',
            label: 'Backgrounds',
            glyph: '📜',
            subtitle: function () { return null; },
        },
        {
            table: 'feat',
            label: 'Feats',
            glyph: '🎖',
            subtitle: function (r) {
                return r.has_prerequisite && r.prerequisite ? String(r.prerequisite) : null;
            },
        },
        {
            table: 'item',
            label: 'Items & gear',
            glyph: '🎒',
            subtitle: function (r, l) {
                const tail = r.is_magic && typeof r.rarity === 'string'
                    ? humanizeSlug(r.rarity)
                    : typeof r.cost === 'string'
                        ? r.cost + ' gp'
                        : null;
                return joinParts([l.nameOf(l.itemCategories, r.category_uuid), tail]);
            },
        },
        {
            table: 'weapon',
            label: 'Weapons',
            glyph: '⚔',
            subtitle: function (r) {
                return joinParts([
                    r.is_simple ? 'Simple' : 'Martial',
                    r.damage_dice != null ? r.damage_dice + ' ' + r.damage_type : null,
                ]);
            },
        },
        {
            table: 'armor',
            label: 'Armor',
            glyph: '🛡',
            subtitle: function (r) {
                return joinParts([
                    typeof r.category === 'string' ? humanizeSlug(r.category) : null,
                    typeof r.ac_display === 'string' ? 'AC ' + r.ac_display : null,
                ]);
            },
        },
        {
            table: 'condition',
            label: 'Conditions',
            glyph: '🩹',
            subtitle: function () { return null; },
            displayName: function (r) { return humanizeSlug(String(r.name)); },
        },
        {
            table: 'language',
            label: 'Languages',
            glyph: '🗨',
            subtitle: function () { return null; },
        },
    ];

    for (const c of categories) {
        if (!c.displayName) c.displayName = function (r) { return String(r.name); };
    }

    function categoryFor(table) {
        return categories.find(function (c) { return c.table === table; }) || null;
    }

    /* ── filter dimensions (port of filters.dart) ───────────────── */

    function dimSource() {
        return {
            key: 'source',
            label: 'Source',
            valuesOf: function (r) { return [r.document_uuid != null ? r.document_uuid : null]; },
            optionLabel: function (v, l) {
                return typeof v === 'string' ? l.documents[v] || 'Unknown source' : 'Unknown source';
            },
        };
    }

    /* Lookup-style tables (conditions, languages) carry no document
     * reference — their source is the installing content module. */
    function dimModuleSource() {
        return {
            key: 'source',
            label: 'Source',
            valuesOf: function (r) { return [r.content_module_uuid != null ? r.content_module_uuid : null]; },
            optionLabel: function (v, l) {
                return typeof v === 'string'
                    ? l.contentModules[v] || 'Unknown source'
                    : 'Unknown source';
            },
        };
    }

    const RARITY_ORDER = [null, 'common', 'uncommon', 'rare', 'very_rare', 'legendary', 'artifact'];
    const COMPONENT_ORDER = ['verbal', 'somatic', 'material'];

    function filterDimensionsFor(table) {
        const source = dimSource();
        switch (table) {
            case 'spell':
                return [
                    source,
                    {
                        key: 'level',
                        label: 'Level',
                        valuesOf: function (r) { return [typeof r.level === 'number' ? r.level : null]; },
                        optionLabel: function (v) {
                            return typeof v === 'number' ? spellLevelLabel(v) : 'Unknown';
                        },
                        valueSort: function (a, b) {
                            const ra = typeof a === 'number' ? a : 99;
                            const rb = typeof b === 'number' ? b : 99;
                            return ra - rb;
                        },
                    },
                    {
                        key: 'school',
                        label: 'School',
                        valuesOf: function (r) { return [r.school != null ? r.school : null]; },
                        optionLabel: function (v, l) {
                            return typeof v === 'string'
                                ? l.spellSchools[v] || 'Unknown school'
                                : 'Unknown school';
                        },
                    },
                    {
                        key: 'components',
                        label: 'Components',
                        valuesOf: function (r) {
                            const out = [];
                            if (r.verbal) out.push('verbal');
                            if (r.somatic) out.push('somatic');
                            if (r.material) out.push('material');
                            return out;
                        },
                        optionLabel: function (v) {
                            if (v === 'verbal') return 'V (verbal)';
                            if (v === 'somatic') return 'S (somatic)';
                            if (v === 'material') return 'M (material)';
                            return String(v);
                        },
                        valueSort: function (a, b) {
                            return COMPONENT_ORDER.indexOf(String(a)) - COMPONENT_ORDER.indexOf(String(b));
                        },
                    },
                ];
            case 'item':
                return [
                    source,
                    {
                        key: 'type',
                        label: 'Type',
                        valuesOf: function (r) { return [r.category_uuid != null ? r.category_uuid : null]; },
                        optionLabel: function (v, l) {
                            return typeof v === 'string'
                                ? l.itemCategories[v] || 'Uncategorized'
                                : 'Uncategorized';
                        },
                    },
                    {
                        key: 'rarity',
                        label: 'Rarity',
                        valuesOf: function (r) { return [typeof r.rarity === 'string' ? r.rarity : null]; },
                        optionLabel: function (v) {
                            return typeof v === 'string' ? humanizeSlug(v) : 'Mundane';
                        },
                        valueSort: function (a, b) {
                            // Unrecognized rarities sort after the known ladder.
                            function rank(v) {
                                const i = RARITY_ORDER.indexOf(v);
                                return i < 0 ? RARITY_ORDER.length : i;
                            }
                            return rank(a) - rank(b);
                        },
                    },
                ];
            case 'creature':
                return [
                    source,
                    {
                        key: 'type',
                        label: 'Creature type',
                        valuesOf: function (r) { return [r.type != null ? r.type : null]; },
                        optionLabel: function (v, l) {
                            return typeof v === 'string'
                                ? l.creatureTypes[v] || 'Unknown type'
                                : 'Unknown type';
                        },
                    },
                ];
            case 'weapon':
                return [
                    source,
                    {
                        key: 'category',
                        label: 'Category',
                        valuesOf: function (r) { return [!!r.is_simple]; },
                        optionLabel: function (v) { return v === true ? 'Simple' : 'Martial'; },
                    },
                ];
            case 'armor':
                return [
                    source,
                    {
                        key: 'category',
                        label: 'Category',
                        valuesOf: function (r) { return [typeof r.category === 'string' ? r.category : null]; },
                        optionLabel: function (v) {
                            return typeof v === 'string' ? humanizeSlug(v) : 'Uncategorized';
                        },
                    },
                ];
            case 'species':
            case 'class':
            case 'background':
            case 'feat':
                return [source];
            case 'condition':
            case 'language':
                return [dimModuleSource()];
            default:
                return [];
        }
    }

    /* Distinct values present in the loaded records, as labeled
     * options. Sorted by valueSort when given, else by label. */
    function dimensionOptions(dim, records, lookups) {
        const seen = new Set();
        const values = [];
        for (const r of records) {
            for (const v of dim.valuesOf(r)) {
                const key = v === null ? ' null' : typeof v + ':' + v;
                if (!seen.has(key)) {
                    seen.add(key);
                    values.push(v);
                }
            }
        }
        const options = values.map(function (v) {
            return { value: v, label: dim.optionLabel(v, lookups) };
        });
        if (dim.valueSort) {
            options.sort(function (a, b) { return dim.valueSort(a.value, b.value); });
        } else {
            options.sort(function (a, b) { return a.label.localeCompare(b.label); });
        }
        return options;
    }

    /* ── sorts (port of sortOptionsFor) ─────────────────────────── */

    function byName(a, b) {
        return String(a.name).toLowerCase().localeCompare(String(b.name).toLowerCase());
    }

    function sortOptionsFor(table) {
        const sorts = [{ key: 'name', label: 'Name', compare: function (a, b) { return byName(a, b); } }];
        if (table === 'spell') {
            sorts.push({
                key: 'level',
                label: 'Level',
                compare: function (a, b) {
                    const byLevel = (a.level || 0) - (b.level || 0);
                    return byLevel !== 0 ? byLevel : byName(a, b);
                },
            });
        }
        if (table === 'item') {
            sorts.push({
                key: 'cost',
                label: 'Cost',
                compare: function (a, b) {
                    // Costs are decimal strings ("25.00"); priceless or
                    // unpriced items sort last.
                    function parse(v) {
                        if (typeof v !== 'string') return Infinity;
                        const n = parseFloat(v);
                        return Number.isFinite(n) ? n : Infinity;
                    }
                    const byCost = parse(a.cost) - parse(b.cost);
                    return byCost !== 0 ? byCost : byName(a, b);
                },
            });
        }
        if (table === 'creature') {
            sorts.push({
                key: 'cr',
                label: 'Challenge rating',
                compare: function (a, b) {
                    const ca = typeof a.challenge_rating === 'number' ? a.challenge_rating : -1;
                    const cb = typeof b.challenge_rating === 'number' ? b.challenge_rating : -1;
                    return ca !== cb ? ca - cb : byName(a, b);
                },
            });
            sorts.push({
                key: 'size',
                label: 'Size',
                lookupCompare: true,
                compare: function (a, b, lookups) {
                    // Rank from the size table (Tiny 1 … Gargantuan 6);
                    // unknown sizes sort last.
                    function rank(r) {
                        return typeof r.size === 'string' ? lookups.sizeRanks[r.size] || 99 : 99;
                    }
                    const bySize = rank(a) - rank(b);
                    return bySize !== 0 ? bySize : byName(a, b);
                },
            });
        }
        return sorts;
    }

    /* ── filter state + matching ────────────────────────────────── */

    function newFilterState(sorts) {
        return {
            selections: new Map(), // dim key → Set of raw values
            sort: sorts[0],
            activeCount: function () {
                let n = 0;
                this.selections.forEach(function (set) { if (set.size > 0) n += 1; });
                return n;
            },
            reset: function (defaultSort) {
                this.selections.clear();
                this.sort = defaultSort;
            },
        };
    }

    /* AND across dimensions, OR within one; an empty selection means
     * "no restriction". */
    function matchesFilters(record, dimensions, selections) {
        for (const dim of dimensions) {
            const selected = selections.get(dim.key);
            if (!selected || selected.size === 0) continue;
            let hit = false;
            for (const v of dim.valuesOf(record)) {
                if (selected.has(v)) { hit = true; break; }
            }
            if (!hit) return false;
        }
        return true;
    }

    /* Full visible-list computation: text search + filters + sort. */
    function visibleRecords(records, category, query, dimensions, state, lookups) {
        const q = (query || '').trim().toLowerCase();
        const out = records.filter(function (r) {
            if (q && !category.displayName(r).toLowerCase().includes(q)) return false;
            return matchesFilters(r, dimensions, state.selections);
        });
        const sort = state.sort;
        out.sort(function (a, b) { return sort.compare(a, b, lookups); });
        return out;
    }

    /* ── modal scaffolding ──────────────────────────────────────── */

    function openModal(panelClass) {
        const overlay = el('div', 'lw-modal-overlay');
        const panel = el('div', 'lw-modal ' + panelClass);
        overlay.appendChild(panel);
        document.body.appendChild(overlay);
        let onClose = null;
        function close() {
            overlay.remove();
            document.removeEventListener('keydown', onKey);
            if (onClose) onClose();
        }
        function onKey(e) {
            if (e.key === 'Escape') close();
        }
        overlay.addEventListener('click', function (e) {
            if (e.target === overlay) close();
        });
        document.addEventListener('keydown', onKey);
        return {
            overlay: overlay,
            panel: panel,
            close: close,
            setOnClose: function (fn) { onClose = fn; },
        };
    }

    /* ── filter & sort panel (port of filter_sheet.dart) ────────── */

    function openFilterPanel(opts) {
        const dimensions = opts.dimensions;
        const sorts = opts.sorts;
        const lookups = opts.lookups;
        const records = opts.records;
        const state = opts.state;
        const onChanged = opts.onChanged || function () {};

        const modal = openModal('lw-filter-modal');
        const panel = modal.panel;

        function rebuild() {
            panel.replaceChildren();

            const header = el('div', 'lw-filter-header');
            header.appendChild(el('h2', 'lw-modal-title', 'Filter & sort'));
            const resetBtn = el('button', 'lw-btn lw-btn-text', 'Reset');
            resetBtn.type = 'button';
            resetBtn.disabled = state.activeCount() === 0 && state.sort === sorts[0];
            resetBtn.addEventListener('click', function () {
                state.reset(sorts[0]);
                onChanged();
                rebuild();
            });
            header.appendChild(resetBtn);
            panel.appendChild(header);

            const body = el('div', 'lw-filter-body');
            if (sorts.length > 1) {
                body.appendChild(el('h3', 'lw-filter-section-title', 'Sort by'));
                const row = el('div', 'lw-chip-row');
                for (const sort of sorts) {
                    const chip = el(
                        'button',
                        'lw-chip' + (state.sort === sort ? ' lw-chip-selected' : ''),
                        sort.label
                    );
                    chip.type = 'button';
                    chip.addEventListener('click', function () {
                        state.sort = sort;
                        onChanged();
                        rebuild();
                    });
                    row.appendChild(chip);
                }
                body.appendChild(row);
            }

            for (const dim of dimensions) {
                body.appendChild(el('h3', 'lw-filter-section-title', dim.label));
                const row = el('div', 'lw-chip-row');
                const selected = state.selections.get(dim.key) || new Set();
                for (const option of dimensionOptions(dim, records, lookups)) {
                    const isOn = selected.has(option.value);
                    const chip = el(
                        'button',
                        'lw-chip' + (isOn ? ' lw-chip-selected' : ''),
                        option.label
                    );
                    chip.type = 'button';
                    chip.addEventListener('click', function () {
                        const set = state.selections.get(dim.key) || new Set();
                        if (set.has(option.value)) set.delete(option.value);
                        else set.add(option.value);
                        state.selections.set(dim.key, set);
                        onChanged();
                        rebuild();
                    });
                    row.appendChild(chip);
                }
                body.appendChild(row);
            }
            panel.appendChild(body);

            const actions = el('div', 'lw-modal-actions');
            const done = el('button', 'lw-btn lw-btn-filled', 'Done');
            done.type = 'button';
            done.addEventListener('click', modal.close);
            actions.appendChild(done);
            panel.appendChild(actions);
        }

        rebuild();
        return modal;
    }

    /* Funnel button with active-filter count badge. */
    function decorateFilterButton(button, state) {
        let badge = button.querySelector('.lw-badge');
        if (!badge) {
            badge = el('span', 'lw-badge');
            button.appendChild(badge);
        }
        const n = state.activeCount();
        badge.textContent = String(n);
        badge.hidden = n === 0;
    }

    /* ── list rows ──────────────────────────────────────────────── */

    function buildEntryRow(record, category, lookups, action) {
        const li = el('li', 'lw-list-item');
        const inner = typeof action === 'string' ? el('a', 'lw-list-item-link') : el('button', 'lw-list-item-link');
        if (typeof action === 'string') inner.href = action;
        else {
            inner.type = 'button';
            inner.addEventListener('click', function () { action(record); });
        }

        const textCol = el('div', 'lw-list-item-text');
        textCol.appendChild(el('div', 'lw-list-item-title', category.displayName(record)));
        const subtitle = category.subtitle(record, lookups);
        if (subtitle) textCol.appendChild(el('div', 'lw-list-item-subtitle', subtitle));
        inner.appendChild(textCol);

        const sourceSlug = lookups.sourceSlugOf(record);
        if (sourceSlug) inner.appendChild(el('span', 'lw-source-badge', sourceSlug));

        li.appendChild(inner);
        return li;
    }

    /* ── content picker (port of content_picker.dart) ───────────── */

    function openPicker(opts) {
        const table = opts.table;
        const category = categoryFor(table);
        const recordFilter = opts.recordFilter || function () { return true; };
        const dimensions = filterDimensionsFor(table);
        const sorts = sortOptionsFor(table);
        const state = newFilterState(sorts);

        return new Promise(function (resolve) {
            const modal = openModal('lw-picker-modal');
            modal.setOnClose(function () { resolve(null); });
            const panel = modal.panel;

            panel.appendChild(el('h2', 'lw-modal-title', opts.title || 'Select ' + category.label));

            const searchRow = el('div', 'lw-search-row');
            const search = el('input', 'lw-input');
            search.type = 'search';
            search.placeholder = 'Search…';
            searchRow.appendChild(search);
            const filterBtn = el('button', 'lw-filter-btn', '☰');
            filterBtn.type = 'button';
            filterBtn.setAttribute('aria-label', 'Filter & sort');
            searchRow.appendChild(filterBtn);
            panel.appendChild(searchRow);

            const listWrap = el('div', 'lw-picker-list');
            const list = el('ul', 'lw-list');
            listWrap.appendChild(list);
            panel.appendChild(listWrap);
            const status = el('p', 'lw-picker-status', 'Loading…');
            panel.appendChild(status);

            Promise.all([loadLookups(), fetchTable(table)]).then(function (results) {
                const lookups = results[0];
                const records = results[1].filter(recordFilter);

                function render() {
                    const visible = visibleRecords(
                        records, category, search.value, dimensions, state, lookups
                    );
                    list.replaceChildren();
                    for (const record of visible) {
                        list.appendChild(
                            buildEntryRow(record, category, lookups, function (r) {
                                modal.setOnClose(function () {});
                                modal.close();
                                resolve(r);
                            })
                        );
                    }
                    status.textContent = visible.length === 0
                        ? 'No matches.'
                        : visible.length + ' of ' + records.length;
                    decorateFilterButton(filterBtn, state);
                }

                search.addEventListener('input', render);
                filterBtn.addEventListener('click', function () {
                    openFilterPanel({
                        dimensions: dimensions,
                        sorts: sorts,
                        lookups: lookups,
                        records: records,
                        state: state,
                        onChanged: render,
                    });
                });
                render();
                search.focus();
            }).catch(function (err) {
                status.textContent = 'Failed to load: ' + String(err);
            });
        });
    }

    /* ── markdown rendering (DOM-emitting, XSS-safe) ────────────── */

    function renderInline(target, text) {
        // bold / italic / inline code / links, longest-marker first.
        const pattern = /(\*\*([^*]+)\*\*)|(\*([^*]+)\*)|(_([^_]+)_)|(`([^`]+)`)|(\[([^\]]+)\]\((https?:\/\/[^)\s]+)\))/;
        let rest = String(text);
        while (rest.length > 0) {
            const m = rest.match(pattern);
            if (!m) {
                target.appendChild(document.createTextNode(rest));
                return;
            }
            if (m.index > 0) {
                target.appendChild(document.createTextNode(rest.slice(0, m.index)));
            }
            if (m[1]) {
                const strong = el('strong');
                renderInline(strong, m[2]);
                target.appendChild(strong);
            } else if (m[3] || m[5]) {
                const em = el('em');
                renderInline(em, m[4] || m[6]);
                target.appendChild(em);
            } else if (m[7]) {
                target.appendChild(el('code', null, m[8]));
            } else if (m[9]) {
                const a = el('a', null, m[10]);
                a.href = m[11];
                a.rel = 'noopener noreferrer';
                a.target = '_blank';
                target.appendChild(a);
            }
            rest = rest.slice(m.index + m[0].length);
        }
    }

    function isTableRow(line) {
        const t = line.trim();
        return t.startsWith('|') && t.endsWith('|') && t.length > 1;
    }

    function splitTableRow(line) {
        const t = line.trim();
        return t.slice(1, -1).split('|').map(function (c) { return c.trim(); });
    }

    function renderMarkdown(text) {
        const frag = document.createDocumentFragment();
        if (!text) return frag;
        const lines = String(text).replace(/\r\n/g, '\n').split('\n');
        let i = 0;

        while (i < lines.length) {
            const line = lines[i];
            const trimmed = line.trim();

            if (trimmed === '') { i += 1; continue; }

            const heading = trimmed.match(/^(#{1,6})\s+(.*)$/);
            if (heading) {
                const h = el('h' + Math.min(heading[1].length + 2, 6), 'lw-md-heading');
                renderInline(h, heading[2]);
                frag.appendChild(h);
                i += 1;
                continue;
            }

            if (/^(-{3,}|\*{3,})$/.test(trimmed)) {
                frag.appendChild(el('hr'));
                i += 1;
                continue;
            }

            if (trimmed.startsWith('>')) {
                const quote = el('blockquote');
                const parts = [];
                while (i < lines.length && lines[i].trim().startsWith('>')) {
                    parts.push(lines[i].trim().replace(/^>\s?/, ''));
                    i += 1;
                }
                const p = el('p');
                renderInline(p, parts.join(' '));
                quote.appendChild(p);
                frag.appendChild(quote);
                continue;
            }

            if (isTableRow(trimmed)) {
                const rows = [];
                while (i < lines.length && isTableRow(lines[i].trim())) {
                    rows.push(splitTableRow(lines[i]));
                    i += 1;
                }
                const table = el('table');
                let bodyRows = rows;
                // Header + |---| separator row.
                if (rows.length >= 2 && rows[1].every(function (c) { return /^:?-{2,}:?$/.test(c); })) {
                    const thead = el('thead');
                    const tr = el('tr');
                    for (const cell of rows[0]) {
                        const th = el('th');
                        renderInline(th, cell);
                        tr.appendChild(th);
                    }
                    thead.appendChild(tr);
                    table.appendChild(thead);
                    bodyRows = rows.slice(2);
                }
                const tbody = el('tbody');
                for (const cells of bodyRows) {
                    const tr = el('tr');
                    for (const cell of cells) {
                        const td = el('td');
                        renderInline(td, cell);
                        tr.appendChild(td);
                    }
                    tbody.appendChild(tr);
                }
                table.appendChild(tbody);
                frag.appendChild(table);
                continue;
            }

            const unordered = /^[-*+]\s+/.test(trimmed);
            const ordered = /^\d+\.\s+/.test(trimmed);
            if (unordered || ordered) {
                const list = el(ordered ? 'ol' : 'ul');
                const marker = ordered ? /^\d+\.\s+/ : /^[-*+]\s+/;
                while (i < lines.length) {
                    const t = lines[i].trim();
                    if (!(ordered ? /^\d+\.\s+/.test(t) : /^[-*+]\s+/.test(t))) break;
                    const li = el('li');
                    renderInline(li, t.replace(marker, ''));
                    list.appendChild(li);
                    i += 1;
                }
                frag.appendChild(list);
                continue;
            }

            // Paragraph: consume until a blank line or block opener.
            const parts = [trimmed];
            i += 1;
            while (i < lines.length) {
                const t = lines[i].trim();
                if (
                    t === '' || t.startsWith('#') || t.startsWith('>') ||
                    isTableRow(t) || /^[-*+]\s+/.test(t) || /^\d+\.\s+/.test(t)
                ) break;
                parts.push(t);
                i += 1;
            }
            const p = el('p');
            renderInline(p, parts.join(' '));
            frag.appendChild(p);
        }
        return frag;
    }

    /* ── character rules shared by the wizard and the sheet ─────── */

    const abilityList = [
        { key: 'strength', label: 'Strength', abbr: 'STR' },
        { key: 'dexterity', label: 'Dexterity', abbr: 'DEX' },
        { key: 'constitution', label: 'Constitution', abbr: 'CON' },
        { key: 'intelligence', label: 'Intelligence', abbr: 'INT' },
        { key: 'wisdom', label: 'Wisdom', abbr: 'WIS' },
        { key: 'charisma', label: 'Charisma', abbr: 'CHA' },
    ];

    const skillList = [
        { key: 'acrobatics', label: 'Acrobatics', ability: 'dexterity' },
        { key: 'animalHandling', label: 'Animal Handling', ability: 'wisdom' },
        { key: 'arcana', label: 'Arcana', ability: 'intelligence' },
        { key: 'athletics', label: 'Athletics', ability: 'strength' },
        { key: 'deception', label: 'Deception', ability: 'charisma' },
        { key: 'history', label: 'History', ability: 'intelligence' },
        { key: 'insight', label: 'Insight', ability: 'wisdom' },
        { key: 'intimidation', label: 'Intimidation', ability: 'charisma' },
        { key: 'investigation', label: 'Investigation', ability: 'intelligence' },
        { key: 'medicine', label: 'Medicine', ability: 'wisdom' },
        { key: 'nature', label: 'Nature', ability: 'intelligence' },
        { key: 'perception', label: 'Perception', ability: 'wisdom' },
        { key: 'performance', label: 'Performance', ability: 'charisma' },
        { key: 'persuasion', label: 'Persuasion', ability: 'charisma' },
        { key: 'religion', label: 'Religion', ability: 'intelligence' },
        { key: 'sleightOfHand', label: 'Sleight of Hand', ability: 'dexterity' },
        { key: 'stealth', label: 'Stealth', ability: 'dexterity' },
        { key: 'survival', label: 'Survival', ability: 'wisdom' },
    ];

    function abilityMod(score) {
        return Math.floor((score - 10) / 2);
    }

    function proficiencyBonus(level) {
        return 2 + Math.floor((level - 1) / 4);
    }

    function formatBonus(n) {
        return n >= 0 ? '+' + n : String(n);
    }

    /* ── auth gating for login-required pages ───────────────────── */

    /* Reveals #lw-page-root and calls init(me) when a session exists;
     * otherwise shows the #lw-login-required prompt. Login reloads the
     * page, so a fresh gate pass follows. */
    function requireAuth(init) {
        document.addEventListener('lw-auth-ready', function (e) {
            const gate = document.getElementById('lw-login-required');
            const root = document.getElementById('lw-page-root');
            if (!e.detail) {
                if (gate) {
                    gate.hidden = false;
                    const btn = document.getElementById('lw-login-required-btn');
                    const login = document.getElementById('lw-btn-login');
                    if (btn && login) {
                        btn.addEventListener('click', function () { login.click(); });
                    }
                }
                return;
            }
            if (gate) gate.hidden = true;
            if (root) root.hidden = false;
            init(e.detail);
        });
    }

    /* ── exports ────────────────────────────────────────────────── */

    return {
        el: el,
        requireAuth: requireAuth,
        humanizeSlug: humanizeSlug,
        categoryPlural: categoryPlural,
        spellLevelLabel: spellLevelLabel,
        formatChallengeRating: formatChallengeRating,
        fetchJson: fetchJson,
        fetchTable: fetchTable,
        fetchEntry: fetchEntry,
        loadLookups: loadLookups,
        categories: categories,
        categoryFor: categoryFor,
        filterDimensionsFor: filterDimensionsFor,
        sortOptionsFor: sortOptionsFor,
        dimensionOptions: dimensionOptions,
        newFilterState: newFilterState,
        matchesFilters: matchesFilters,
        visibleRecords: visibleRecords,
        openModal: openModal,
        openFilterPanel: openFilterPanel,
        decorateFilterButton: decorateFilterButton,
        buildEntryRow: buildEntryRow,
        openPicker: openPicker,
        renderMarkdown: renderMarkdown,
        abilityList: abilityList,
        skillList: skillList,
        abilityMod: abilityMod,
        proficiencyBonus: proficiencyBonus,
        formatBonus: formatBonus,
    };
})();
