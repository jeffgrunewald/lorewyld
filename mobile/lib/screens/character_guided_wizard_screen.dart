// New-player guidance quiz: nine questions about interests and play
// style, answered one per page, then ranked class/species/background
// suggestions from the installed content with "why this fits" notes.
//
// The questionnaire and the recommendation engine live in the shared
// Rust core (lorewyld-domain via FFI) — the same logic the web quiz
// runs through WASM. Chosen picks hand off to the regular create
// wizard as prefills; everything stays editable there and on the sheet.

import 'dart:convert';

import 'package:flutter/material.dart';

import '../compendium/categories.dart';
import '../ffi/api/guidance.dart';
import '../services/content_store.dart';
import '../services/local_store.dart';
import '../types/character.dart';
import 'character_create_wizard_screen.dart';

class CharacterGuidedWizardScreen extends StatefulWidget {
  const CharacterGuidedWizardScreen({super.key, required this.store});

  final LocalStore store;

  @override
  State<CharacterGuidedWizardScreen> createState() =>
      _CharacterGuidedWizardScreenState();
}

class _CharacterGuidedWizardScreenState
    extends State<CharacterGuidedWizardScreen> {
  late final ContentStore _content = ContentStore(widget.store);
  final _pageCtl = PageController();

  late final List<Map<String, dynamic>> _questions;
  late final String _intro;
  final Map<String, String> _answers = {};

  // Results phase: null until the last question is answered.
  Map<String, dynamic>? _results;
  Map<String, List<Map<String, dynamic>>> _candidates = const {};
  final Map<String, Map<String, dynamic>?> _picks = {};
  bool _scoring = false;
  String _error = '';

  @override
  void initState() {
    super.initState();
    final quiz = jsonDecode(guidanceQuestionnaire()) as Map<String, dynamic>;
    _intro = quiz['intro'] as String? ?? '';
    // Option display order is shuffled once per quiz so no answer
    // benefits from position bias; answers are keyed by stable values,
    // so the deterministic engine is unaffected, and back-swiping
    // doesn't reshuffle.
    _questions = [
      for (final q in quiz['questions'] as List)
        {
          ...q as Map<String, dynamic>,
          'options': [...q['options'] as List]..shuffle(),
        },
    ];
  }

  @override
  void dispose() {
    _pageCtl.dispose();
    super.dispose();
  }

  Future<void> _answer(String questionKey, String value, int index) async {
    setState(() => _answers[questionKey] = value);
    if (index + 1 < _questions.length) {
      await _pageCtl.nextPage(
        duration: const Duration(milliseconds: 250),
        curve: Curves.easeOut,
      );
    } else {
      await _score();
    }
  }

  /// Ranks the installed content against the collected answers. Full
  /// records (not summaries) go to the engine — mobile has them locally,
  /// and they give the heuristic the most to work with for homebrew.
  Future<void> _score() async {
    setState(() {
      _scoring = true;
      _error = '';
    });
    try {
      final classes = await _content.listClasses(basesOnly: true);
      final species = await _content.listSpecies();
      final backgrounds = await _content.listBackgrounds();
      final answers = [
        for (final e in _answers.entries)
          {'question': e.key, 'value': e.value},
      ];
      final raw = guidanceRecommend(
        answersJson: jsonEncode(answers),
        candidatesJson: jsonEncode({
          'classes': classes,
          'species': species,
          'backgrounds': backgrounds,
        }),
      );
      final results = jsonDecode(raw) as Map<String, dynamic>?;
      if (results == null) throw Exception('recommendation failed');
      if (!mounted) return;
      setState(() {
        _candidates = {
          'classes': classes,
          'species': species,
          'backgrounds': backgrounds,
        };
        _results = results;
        _picks['classes'] = _firstRecommendation(results, 'classes');
        _picks['species'] = _firstRecommendation(results, 'species');
        _picks['backgrounds'] = _firstRecommendation(results, 'backgrounds');
        _scoring = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _scoring = false;
        _error = 'Failed to build suggestions: $e';
      });
    }
  }

  static Map<String, dynamic>? _firstRecommendation(
    Map<String, dynamic> results,
    String category,
  ) {
    final recs = results[category] as List? ?? const [];
    return recs.isEmpty ? null : recs.first as Map<String, dynamic>;
  }

  /// The full local record behind a recommendation, for wizard prefill.
  Map<String, dynamic>? _recordFor(String category, Map<String, dynamic>? rec) {
    if (rec == null) return null;
    for (final record in _candidates[category] ?? const []) {
      if (record['uuid'] == rec['uuid']) return record;
    }
    return null;
  }

  Map<Ability, int> _suggestedAbilities() {
    final scores =
        _results!['standard_array_suggestion'] as Map<String, dynamic>? ?? {};
    return {
      for (final a in Ability.values) a: (scores[a.name] as num?)?.toInt() ?? 10,
    };
  }

  Future<void> _useThesePicks() async {
    final sheet = await Navigator.of(context).push<CharacterSheet>(
      MaterialPageRoute(
        builder: (_) => CharacterCreateWizardScreen(
          store: widget.store,
          initialSpecies: _recordFor('species', _picks['species']),
          initialClass: _recordFor('classes', _picks['classes']),
          initialBackground: _recordFor('backgrounds', _picks['backgrounds']),
          initialAlignment: humanizeSlug(
            '${_results!['alignment_suggestion']}',
          ),
          initialAbilities: _suggestedAbilities(),
        ),
      ),
    );
    if (sheet == null || !mounted) return;
    // Chain the created sheet back to the character list.
    Navigator.of(context).pop<CharacterSheet>(sheet);
  }

  void _retake() {
    setState(() {
      _results = null;
      _error = '';
      _answers.clear();
      _picks.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Find your character')),
      body: _results != null
          ? _buildResults(context)
          : _scoring
          ? const Center(child: CircularProgressIndicator())
          : _buildQuiz(context),
    );
  }

  /* ── quiz pages ─────────────────────────────────────────────── */

  Widget _buildQuiz(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
          child: ListenableBuilder(
            listenable: _pageCtl,
            builder: (context, _) {
              final page = _pageCtl.hasClients
                  ? (_pageCtl.page ?? 0)
                  : 0.0;
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Question ${page.round() + 1} of ${_questions.length}',
                    style: Theme.of(context).textTheme.labelMedium,
                  ),
                  const SizedBox(height: 6),
                  LinearProgressIndicator(
                    value: page / _questions.length,
                    minHeight: 6,
                    borderRadius: BorderRadius.circular(3),
                  ),
                ],
              );
            },
          ),
        ),
        Expanded(
          child: PageView.builder(
            controller: _pageCtl,
            itemCount: _questions.length,
            itemBuilder: (context, index) =>
                _buildQuestion(context, _questions[index], index),
          ),
        ),
      ],
    );
  }

  Widget _buildQuestion(
    BuildContext context,
    Map<String, dynamic> question,
    int index,
  ) {
    final key = question['key'] as String;
    final selected = _answers[key];
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // Framing from the engine: answer as the character you want to
        // inhabit, not as yourself.
        if (index == 0 && _intro.isNotEmpty) ...[
          Text(_intro, style: Theme.of(context).textTheme.bodyMedium),
          const SizedBox(height: 16),
        ],
        Text(
          question['prompt'] as String,
          style: Theme.of(context).textTheme.titleLarge,
        ),
        const SizedBox(height: 16),
        for (final option in question['options'] as List)
          Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: _OptionCard(
              label: (option as Map<String, dynamic>)['label'] as String,
              selected: selected == option['value'],
              onTap: () => _answer(key, option['value'] as String, index),
            ),
          ),
        if (_error.isNotEmpty)
          Text(
            _error,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
      ],
    );
  }

  /* ── results ────────────────────────────────────────────────── */

  Widget _buildResults(BuildContext context) {
    final results = _results!;
    final abilities = _suggestedAbilities();
    final statLine = Ability.values
        .map((a) => '${a.abbr} ${abilities[a]}')
        .join(' · ');
    final alignment = humanizeSlug('${results['alignment_suggestion']}');

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          'Here’s what fits the character you described. Pick one from '
          'each group — you can change everything later.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 8),
        ..._resultSection(context, 'Class', 'classes'),
        ..._resultSection(context, 'Species', 'species'),
        ..._resultSection(context, 'Background', 'backgrounds'),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Suggested starting stats',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                Text(statLine),
                const SizedBox(height: 8),
                Text(
                  'Alignment: $alignment — ${results['alignment_reason']}',
                ),
                const SizedBox(height: 8),
                Text(
                  'These are prefills, never rules — every value stays '
                  'editable on the sheet.',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ),
        if (_error.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              _error,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
        const SizedBox(height: 16),
        Row(
          children: [
            TextButton(onPressed: _retake, child: const Text('Retake quiz')),
            const Spacer(),
            FilledButton(
              onPressed: _useThesePicks,
              child: const Text('Use these picks'),
            ),
          ],
        ),
      ],
    );
  }

  List<Widget> _resultSection(
    BuildContext context,
    String title,
    String category,
  ) {
    final recs = _results![category] as List? ?? const [];
    return [
      Padding(
        padding: const EdgeInsets.only(top: 16, bottom: 8),
        child: Text(title, style: Theme.of(context).textTheme.titleMedium),
      ),
      if (recs.isEmpty)
        const Text('No installed content to suggest.')
      else
        for (final rec in recs)
          _RecommendationCard(
            recommendation: rec as Map<String, dynamic>,
            selected: _picks[category]?['uuid'] == rec['uuid'],
            onTap: () => setState(() => _picks[category] = rec),
          ),
    ];
  }
}

class _OptionCard extends StatelessWidget {
  const _OptionCard({
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Card(
      margin: EdgeInsets.zero,
      color: selected ? scheme.primaryContainer : null,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: selected ? scheme.primary : scheme.outlineVariant,
        ),
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Text(label, style: Theme.of(context).textTheme.bodyLarge),
        ),
      ),
    );
  }
}

class _RecommendationCard extends StatelessWidget {
  const _RecommendationCard({
    required this.recommendation,
    required this.selected,
    required this.onTap,
  });

  final Map<String, dynamic> recommendation;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final score = ((recommendation['score'] as num?) ?? 0).toDouble();
    final reasons = recommendation['reasons'] as List? ?? const [];
    final mapped = recommendation['mapped'] == true;

    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: selected ? scheme.primary : scheme.outlineVariant,
          width: selected ? 2 : 1,
        ),
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      '${recommendation['name']}',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                  Text(
                    '${(score * 100).round()}% fit',
                    style: Theme.of(context).textTheme.labelMedium,
                  ),
                ],
              ),
              const SizedBox(height: 6),
              LinearProgressIndicator(
                value: score,
                minHeight: 4,
                borderRadius: BorderRadius.circular(2),
              ),
              const SizedBox(height: 8),
              for (final reason in reasons)
                Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: Text(
                    '• $reason',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              if (!mapped)
                Text(
                  'ESTIMATED FROM ITS MECHANICS',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: scheme.outline,
                    letterSpacing: 0.5,
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
