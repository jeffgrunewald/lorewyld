/* New-player guidance quiz for the Lorewyld web app.
 *
 * Renders the fixed questionnaire from the shared Rust core (via
 * window.lwContent.guidanceQuestionnaire), collects answers, ranks the
 * server's installed content with guidanceRecommend, and hands the
 * chosen picks to the /characters/new wizard through sessionStorage.
 * Everything is prefill advice — the create wizard and sheet stay fully
 * editable.
 *
 * Depends on window.lwContent (loaded first, blocking, in order). All
 * DOM construction goes through C.el — never innerHTML with data.
 */
window.lwGuidance = (function () {
    'use strict';

    const C = window.lwContent;
    const PICKS_KEY = 'lw-guided-picks';

    /* Fisher–Yates copy. Option display order is randomized once per
     * quiz load so no answer benefits from position bias; answers are
     * keyed by stable values, so the deterministic engine is unaffected,
     * and shuffling only at load keeps Back from reshuffling. */
    function shuffled(items) {
        const out = items.slice();
        for (let i = out.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            const tmp = out[i];
            out[i] = out[j];
            out[j] = tmp;
        }
        return out;
    }

    function mount(root) {
        const quiz = C.guidanceQuestionnaire();
        if (!quiz) {
            root.replaceChildren(C.el('p', 'lw-picker-status', 'The guidance quiz is unavailable.'));
            return;
        }
        const state = {
            intro: quiz.intro || '',
            questions: quiz.questions.map(function (q) {
                return Object.assign({}, q, { options: shuffled(q.options) });
            }),
            index: 0,
            answers: {},   // question key -> option value
            results: null, // RecommendationSet
            tables: null,  // {classes, species, backgrounds} summary rows
            picks: {},     // category -> selected recommendation
            error: '',
        };
        renderQuestion(root, state);
    }

    /* ── quiz steps ─────────────────────────────────────────────── */

    function renderQuestion(root, state) {
        root.replaceChildren();
        const q = state.questions[state.index];
        const total = state.questions.length;

        // Framing from the engine: answer as the character you want to
        // inhabit, not as yourself.
        if (state.index === 0 && state.intro) {
            root.appendChild(C.el('p', 'lw-page-subtitle', state.intro));
        }
        root.appendChild(C.el('p', 'lw-quiz-count', 'Question ' + (state.index + 1) + ' of ' + total));
        const track = C.el('div', 'lw-quiz-progress');
        const fill = C.el('span', 'lw-quiz-progress-fill');
        fill.style.width = String(Math.round((state.index / total) * 100)) + '%';
        track.appendChild(fill);
        root.appendChild(track);

        root.appendChild(C.el('h2', 'lw-quiz-prompt', q.prompt));

        const list = C.el('div', 'lw-quiz-options');
        for (const option of q.options) {
            const btn = C.el(
                'button',
                'lw-quiz-option' + (state.answers[q.key] === option.value ? ' lw-quiz-option-selected' : ''),
                option.label
            );
            btn.type = 'button';
            btn.addEventListener('click', function () {
                state.answers[q.key] = option.value;
                if (state.index + 1 < total) {
                    state.index += 1;
                    renderQuestion(root, state);
                } else {
                    renderResults(root, state);
                }
            });
            list.appendChild(btn);
        }
        root.appendChild(list);

        const actions = C.el('div', 'lw-step-actions');
        if (state.index > 0) {
            const back = C.el('button', 'lw-btn lw-btn-text', 'Back');
            back.type = 'button';
            back.addEventListener('click', function () {
                state.index -= 1;
                renderQuestion(root, state);
            });
            actions.appendChild(back);
        }
        root.appendChild(actions);
    }

    /* ── results ────────────────────────────────────────────────── */

    function renderResults(root, state) {
        root.replaceChildren(C.el('p', 'lw-picker-status', 'Reading the omens…'));

        Promise.all([
            C.fetchTable('class'),
            C.fetchTable('species'),
            C.fetchTable('background'),
        ]).then(function (tables) {
            const answers = Object.keys(state.answers).map(function (key) {
                return { question: key, value: state.answers[key] };
            });
            state.tables = { classes: tables[0], species: tables[1], backgrounds: tables[2] };
            state.results = C.guidanceRecommend(answers, state.tables);
            if (!state.results) throw new Error('recommendation failed');
            state.picks = {
                class: state.results.classes[0] || null,
                species: state.results.species[0] || null,
                background: state.results.backgrounds[0] || null,
            };
            drawResults(root, state);
        }).catch(function (err) {
            root.replaceChildren(C.el('p', 'lw-picker-status', 'Failed to build suggestions: ' + err));
        });
    }

    function recommendationCard(rec, selected, onSelect) {
        const card = C.el('button', 'lw-rec-card' + (selected ? ' lw-rec-card-selected' : ''));
        card.type = 'button';
        const header = C.el('div', 'lw-rec-card-header');
        header.appendChild(C.el('span', 'lw-rec-card-name', rec.name));
        header.appendChild(C.el('span', 'lw-rec-card-score', Math.round(rec.score * 100) + '% fit'));
        card.appendChild(header);
        const bar = C.el('div', 'lw-rec-score');
        const fill = C.el('span', 'lw-rec-score-fill');
        fill.style.width = String(Math.round(rec.score * 100)) + '%';
        bar.appendChild(fill);
        card.appendChild(bar);
        const reasons = C.el('ul', 'lw-rec-reasons');
        for (const reason of rec.reasons) {
            reasons.appendChild(C.el('li', null, reason));
        }
        card.appendChild(reasons);
        if (!rec.mapped) {
            card.appendChild(C.el('span', 'lw-rec-estimated', 'Estimated from its mechanics'));
        }
        card.addEventListener('click', onSelect);
        return card;
    }

    function resultSection(root, state, title, category, recs) {
        root.appendChild(C.el('h2', 'lw-group-header', title));
        if (recs.length === 0) {
            root.appendChild(C.el('p', 'lw-picker-status', 'No installed content to suggest.'));
            return;
        }
        const row = C.el('div', 'lw-rec-row');
        for (const rec of recs) {
            row.appendChild(recommendationCard(rec, state.picks[category] === rec, function () {
                state.picks[category] = rec;
                drawResults(root, state);
            }));
        }
        root.appendChild(row);
    }

    function abilityLine(results) {
        const scores = results.standard_array_suggestion;
        return C.abilityList.map(function (a) {
            return a.abbr + ' ' + scores[a.key];
        }).join(' · ');
    }

    function drawResults(root, state) {
        root.replaceChildren();
        const results = state.results;

        root.appendChild(C.el('p', 'lw-page-subtitle',
            'Here’s what fits the character you described. Pick one from each row — you can change everything later.'));

        resultSection(root, state, 'Class', 'class', results.classes);
        resultSection(root, state, 'Species', 'species', results.species);
        resultSection(root, state, 'Background', 'background', results.backgrounds);

        const advice = C.el('div', 'lw-card');
        advice.appendChild(C.el('div', 'lw-card-title', 'Suggested starting stats'));
        advice.appendChild(C.el('p', null, abilityLine(results)));
        const alignment = C.humanizeSlug(results.alignment_suggestion);
        advice.appendChild(C.el('p', null, 'Alignment: ' + alignment + ' — ' + results.alignment_reason));
        advice.appendChild(C.el('p', 'lw-field-help',
            'These are prefills, never rules — every value stays editable on the sheet.'));
        root.appendChild(advice);

        if (state.error) {
            root.appendChild(C.el('p', 'lw-form-error', state.error));
        }

        const actions = C.el('div', 'lw-step-actions');
        const retake = C.el('button', 'lw-btn lw-btn-text', 'Retake quiz');
        retake.type = 'button';
        retake.addEventListener('click', function () {
            state.index = 0;
            renderQuestion(root, state);
        });
        actions.appendChild(retake);
        const use = C.el('button', 'lw-btn lw-btn-filled', 'Use these picks');
        use.type = 'button';
        use.addEventListener('click', function () { usePicks(state); });
        actions.appendChild(use);
        root.appendChild(actions);
    }

    /* ── handoff to /characters/new ─────────────────────────────── */

    /* The create wizard wants the records it would have picked itself:
     * the species/background summary rows and the class uuid (it fetches
     * the full class record for saving-throw grants). */
    function tableRecord(rows, rec) {
        return rows.find(function (r) { return r.uuid === rec.uuid; }) || null;
    }

    function usePicks(state) {
        const picks = state.picks;
        const payload = {
            version: state.results.version,
            species: picks.species ? tableRecord(state.tables.species, picks.species) : null,
            class_uuid: picks.class ? picks.class.uuid : null,
            background: picks.background ? tableRecord(state.tables.backgrounds, picks.background) : null,
            alignment: C.humanizeSlug(state.results.alignment_suggestion),
            abilities: state.results.standard_array_suggestion,
        };
        try {
            sessionStorage.setItem(PICKS_KEY, JSON.stringify(payload));
        } catch (err) {
            state.error = 'Could not stash your picks: ' + err;
            return drawResults(document.getElementById('lw-guidance'), state);
        }
        location.href = '/characters/new';
    }

    /* Read-and-clear the guided picks; the create wizard calls this once
     * on init so a plain reload of /characters/new starts blank. */
    function takePicks() {
        try {
            const raw = sessionStorage.getItem(PICKS_KEY);
            if (!raw) return null;
            sessionStorage.removeItem(PICKS_KEY);
            return JSON.parse(raw);
        } catch (err) {
            return null;
        }
    }

    return {
        mount: mount,
        takePicks: takePicks,
    };
})();
