// New-character wizard (name/class required, rest optional); class pick
// pre-fills hit dice + prereq snapshot, species pre-fills speed.

import 'package:flutter/material.dart';

import '../ffi/api/sheet.dart';
import '../services/content_store.dart';
import '../services/local_store.dart';
import '../types/character.dart';
import '../widgets/content_picker.dart';

class CharacterCreateWizardScreen extends StatefulWidget {
  const CharacterCreateWizardScreen({
    super.key,
    required this.store,
    this.initialSpecies,
    this.initialClass,
    this.initialBackground,
    this.initialAlignment,
    this.initialAbilities,
  });

  final LocalStore store;

  /// Prefills from the guided quiz — full content records for the
  /// pickers, a humanized alignment matching the dropdown values, and
  /// suggested ability scores. All stay editable; prefill, never enforce.
  final Map<String, dynamic>? initialSpecies;
  final Map<String, dynamic>? initialClass;
  final Map<String, dynamic>? initialBackground;
  final String? initialAlignment;
  final Map<Ability, int>? initialAbilities;

  @override
  State<CharacterCreateWizardScreen> createState() =>
      _CharacterCreateWizardScreenState();
}

class _CharacterCreateWizardScreenState
    extends State<CharacterCreateWizardScreen> {
  late final ContentStore _content = ContentStore(widget.store);

  final _nameCtl = TextEditingController();
  int _step = 0;
  bool _creating = false;

  Map<String, dynamic>? _species;
  Map<String, dynamic>? _characterClass;
  Map<String, dynamic>? _background;
  String _alignment = '';

  @override
  void initState() {
    super.initState();
    _species = widget.initialSpecies;
    _characterClass = widget.initialClass;
    _background = widget.initialBackground;
    _alignment = widget.initialAlignment ?? '';
  }

  @override
  void dispose() {
    _nameCtl.dispose();
    super.dispose();
  }

  Future<void> _pick(
    String table,
    String title,
    void Function(Map<String, dynamic>) assign, {
    String? where,
  }) async {
    final record = await showContentPicker(
      context: context,
      content: _content,
      table: table,
      title: title,
      where: where,
    );
    if (record != null) setState(() => assign(record));
  }

  Future<void> _create() async {
    final name = _nameCtl.text.trim();
    if (name.isEmpty || _characterClass == null || _creating) return;
    setState(() => _creating = true);
    final created = await widget.store.createCharacter(name);

    // Class grants: saving throw proficiencies, and 1st-level max HP =
    // hit die maximum + Con modifier (5e rule). Con is 10 unless the
    // guided quiz suggested scores — editing scores later won't
    // re-derive HP; prefilled, never enforced.
    final classHitDie = _characterClass?['hit_dice'];
    final classSaves = _characterClass?['prof_saving_throws'];
    final conScore =
        widget.initialAbilities?[Ability.constitution] ??
        created.abilityScore(Ability.constitution);
    final startingHp = classHitDie is num
        ? (classHitDie.truncate() + abilityModifier(score: conScore)).clamp(
            1,
            999,
          )
        : created.maxHp;

    final sheet = await widget.store.saveCharacter(
      created.copyWith(
        abilities: widget.initialAbilities,
        race: _species?['name'] as String? ?? '',
        classes: _characterClass?['name'] is String
            ? [
                ClassEntry(
                  name: _characterClass!['name'] as String,
                  starting: true,
                  primaryAbilities: [
                    for (final group
                        in _characterClass!['primary_abilities']
                                as List<dynamic>? ??
                            [])
                      if (group is List) [for (final a in group) a as String],
                  ],
                ),
              ]
            : null,
        background: _background?['name'] as String? ?? '',
        alignment: _alignment,
        speed: (_species?['speed'] as num?)?.truncate() ?? created.speed,
        hitDice: classHitDie is num ? '1d${classHitDie.truncate()}' : '',
        savingThrowProficiencies: classSaves is List
            ? Ability.parseWireSet(classSaves)
            : null,
        maxHp: startingHp,
        currentHp: startingHp,
      ),
    );
    if (!mounted) return;
    Navigator.of(context).pop<CharacterSheet>(sheet);
  }

  @override
  Widget build(BuildContext context) {
    final nameValid = _nameCtl.text.trim().isNotEmpty;
    final steps = [
      Step(
        title: const Text('Name'),
        isActive: _step >= 0,
        content: TextField(
          controller: _nameCtl,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Character name',
            hintText: 'e.g. Thistle Quickfoot',
            border: OutlineInputBorder(),
          ),
          onChanged: (_) => setState(() {}),
        ),
      ),
      Step(
        title: const Text('Species'),
        subtitle: _species != null ? Text('${_species!['name']}') : null,
        isActive: _step >= 1,
        content: ContentPickerField(
          label: 'Species',
          value: _species?['name'] as String? ?? '',
          onTap: () =>
              _pick('species', 'Choose a species', (r) => _species = r),
          onCleared: () => setState(() => _species = null),
        ),
      ),
      Step(
        title: const Text('Class'),
        subtitle: _characterClass != null
            ? Text('${_characterClass!['name']}')
            : null,
        isActive: _step >= 2,
        content: ContentPickerField(
          label: 'Class',
          value: _characterClass?['name'] as String? ?? '',
          onTap: () => _pick(
            'class',
            'Choose a class',
            (r) => _characterClass = r,
            where: 'subclass_of IS NULL',
          ),
          onCleared: () => setState(() => _characterClass = null),
        ),
      ),
      Step(
        title: const Text('Background & alignment'),
        isActive: _step >= 3,
        content: Column(
          children: [
            ContentPickerField(
              label: 'Background',
              value: _background?['name'] as String? ?? '',
              onTap: () => _pick(
                'background',
                'Choose a background',
                (r) => _background = r,
              ),
              onCleared: () => setState(() => _background = null),
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              initialValue: kCanonicalAlignments.contains(_alignment)
                  ? _alignment
                  : null,
              isExpanded: true,
              decoration: const InputDecoration(
                labelText: 'Alignment',
                border: OutlineInputBorder(),
              ),
              items: [
                for (final a in kCanonicalAlignments)
                  DropdownMenuItem(value: a, child: Text(a)),
              ],
              onChanged: (v) => setState(() => _alignment = v ?? ''),
            ),
          ],
        ),
      ),
    ];

    return Scaffold(
      appBar: AppBar(title: const Text('New character')),
      body: Stepper(
        currentStep: _step,
        onStepTapped: (i) => setState(() => _step = i),
        onStepContinue: () {
          if (_step < steps.length - 1) {
            setState(() => _step++);
          } else {
            _create();
          }
        },
        onStepCancel: _step > 0 ? () => setState(() => _step--) : null,
        // Stepper builds controls for every step (collapsed ones stay
        // in the tree), so the label must come from the step being
        // built, not the active step.
        controlsBuilder: (context, details) {
          final last = details.stepIndex == steps.length - 1;
          // A name and a class are required; everything else can be
          // chosen later on the sheet.
          final canContinue =
              nameValid && !_creating && (!last || _characterClass != null);
          return Padding(
            padding: const EdgeInsets.only(top: 16),
            child: Row(
              children: [
                Tooltip(
                  message: last && _characterClass == null
                      ? 'Choose a class first'
                      : '',
                  child: FilledButton(
                    onPressed: canContinue ? details.onStepContinue : null,
                    child: Text(last ? 'Create' : 'Next'),
                  ),
                ),
                const SizedBox(width: 8),
                if (details.onStepCancel != null)
                  TextButton(
                    onPressed: details.onStepCancel,
                    child: const Text('Back'),
                  ),
              ],
            ),
          );
        },
        steps: steps,
      ),
    );
  }
}
