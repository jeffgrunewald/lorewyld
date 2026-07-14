// Full 5e character sheet, stored locally. Documents and computes —
// modifiers, proficiency, save/skill bonuses are derived live — but
// never blocks a value the player wants to write down.
//
// Species, class, background, spells, and equipment are chosen from the
// installed content modules; alignment from the canonical code list.
//
// Leaving with unsaved edits prompts to save or discard; the save icon
// persists immediately. Removing an item or spell asks for confirmation.

import 'dart:convert';

import 'package:flutter/material.dart';

import '../dice/dice_expression_builder.dart';
import '../ffi/api/sheet.dart';
import '../services/content_store.dart';
import '../services/local_store.dart';
import '../types/character.dart';
import '../widgets/content_picker.dart';
import 'character_notes_screen.dart';

class CharacterSheetScreen extends StatefulWidget {
  const CharacterSheetScreen({
    super.key,
    required this.store,
    required this.sheet,
  });

  final LocalStore store;
  final CharacterSheet sheet;

  @override
  State<CharacterSheetScreen> createState() => _CharacterSheetScreenState();
}

class _CharacterSheetScreenState extends State<CharacterSheetScreen> {
  late CharacterSheet _sheet;
  bool _dirty = false;

  // Derived 5e stats computed by the shared Rust core (lorewyld-domain)
  // over FFI, cached per edit so widget builds stay synchronous.
  late DerivedStats _derived;
  Map<String, int> _abilityMods = const {};
  Map<String, int> _saveBonuses = const {};
  Map<String, int> _skillBonuses = const {};

  late final ContentStore _content = ContentStore(widget.store);
  List<Map<String, dynamic>> _languages = const [];

  // Names of items whose content record requires attunement.
  Set<String> _attunableItems = const {};

  // Class table rows (full records) for subclass lookup + prereq checks.
  List<Map<String, dynamic>> _classRows = const [];

  late final TextEditingController _nameCtl;
  late final TextEditingController _hitDiceCtl;
  late final TextEditingController _xpCtl;

  @override
  void initState() {
    super.initState();
    _sheet = widget.sheet;
    _recompute();
    _nameCtl = TextEditingController(text: _sheet.name);
    _hitDiceCtl = TextEditingController(text: _sheet.hitDice);
    _xpCtl = TextEditingController(
      text: _sheet.experiencePoints?.toString() ?? '',
    );
    _content.listNamed('language').then((rows) {
      if (mounted) setState(() => _languages = rows);
    });
    _content.listNamed('item').then((rows) {
      if (!mounted) return;
      setState(() {
        _attunableItems = {
          for (final r in rows)
            if (r['requires_attunement'] == true && r['name'] is String)
              r['name'] as String,
        };
      });
    });
    _content.listNamed('class').then((rows) {
      if (mounted) setState(() => _classRows = rows);
    });
  }

  @override
  void dispose() {
    _nameCtl.dispose();
    _hitDiceCtl.dispose();
    _xpCtl.dispose();
    super.dispose();
  }

  void _mutate(CharacterSheet next) {
    setState(() {
      _sheet = next;
      _recompute();
      _dirty = true;
    });
  }

  // Recompute derived stats from the shared Rust core. The sheet already
  // serializes to the wire shape the Rust `CharacterSheet` deserializes.
  void _recompute() {
    _derived = deriveStats(sheetJson: jsonEncode(_sheet.toJson()));
    _abilityMods = {for (final b in _derived.abilityModifiers) b.name: b.bonus};
    _saveBonuses = {
      for (final b in _derived.savingThrowBonuses) b.name: b.bonus,
    };
    _skillBonuses = {for (final b in _derived.skillBonuses) b.name: b.bonus};
  }

  CharacterSheet _withTextFields() => _sheet.copyWith(
    name: _nameCtl.text.trim().isEmpty ? _sheet.name : _nameCtl.text.trim(),
    hitDice: _hitDiceCtl.text.trim(),
    // Only commit typed XP while tracking is enabled; an unparseable
    // entry falls back to the last good value rather than clearing it.
    experiencePoints: _sheet.experiencePoints == null
        ? null
        : (int.tryParse(_xpCtl.text.trim()) ?? _sheet.experiencePoints),
  );

  Future<void> _save() async {
    final next = await widget.store.saveCharacter(_withTextFields());
    if (!mounted) return;
    setState(() {
      _sheet = next;
      _recompute();
      _dirty = false;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Saved.'), duration: Duration(seconds: 1)),
    );
  }

  bool get _hasUnsavedChanges => _dirty || _textFieldsChanged();

  Future<void> _confirmLeave() async {
    if (!_hasUnsavedChanges) {
      Navigator.of(context).pop();
      return;
    }
    final action = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Unsaved changes'),
        content: Text('Save changes to "${_sheet.name}" before leaving?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, 'discard'),
            child: const Text('Discard'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, 'save'),
            child: const Text('Save'),
          ),
        ],
      ),
    );
    if (!mounted || action == null) return;
    if (action == 'save') {
      await widget.store.saveCharacter(_withTextFields());
      if (!mounted) return;
    }
    Navigator.of(context).pop();
  }

  Future<bool> _confirmRemove(String kind, String name) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Remove $kind?'),
        content: Text('"$name" will be removed from this character.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton.tonal(
            onPressed: () => Navigator.pop(ctx, true),
            style: FilledButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Remove'),
          ),
        ],
      ),
    );
    return confirm == true;
  }

  Future<void> _delete() async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete character?'),
        content: Text(
          '"${_sheet.name}" and their notes will be permanently removed.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton.tonal(
            onPressed: () => Navigator.pop(ctx, true),
            style: FilledButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirm != true) return;
    await widget.store.deleteCharacter(_sheet.uuid);
    if (!mounted) return;
    Navigator.of(context).pop();
  }

  void _openNotes() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) =>
            CharacterNotesScreen(store: widget.store, character: _sheet),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      // Back is always intercepted so unsaved edits can prompt to save
      // or discard; _confirmLeave pops directly when clean.
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) _confirmLeave();
      },
      child: Scaffold(
        appBar: AppBar(
          title: Text(_sheet.name),
          actions: [
            IconButton(
              icon: const Icon(Icons.menu_book_outlined),
              tooltip: 'Backstory & notes',
              onPressed: _openNotes,
            ),
            IconButton(
              icon: const Icon(Icons.save_outlined),
              tooltip: 'Save',
              onPressed: _save,
            ),
            PopupMenuButton<String>(
              onSelected: (v) {
                if (v == 'delete') _delete();
              },
              itemBuilder: (_) => const [
                PopupMenuItem(value: 'delete', child: Text('Delete character')),
              ],
            ),
          ],
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _identitySection(),
            const SizedBox(height: 16),
            _classesSection(),
            const SizedBox(height: 16),
            _abilitiesSection(),
            const SizedBox(height: 16),
            _combatSection(),
            const SizedBox(height: 16),
            _savesSection(),
            const SizedBox(height: 16),
            _skillsSection(),
            const SizedBox(height: 16),
            _languagesSection(),
            const SizedBox(height: 16),
            _equipmentSection(),
            const SizedBox(height: 16),
            _spellsSection(),
            const SizedBox(height: 32),
          ],
        ),
      ),
    );
  }

  bool _textFieldsChanged() =>
      _nameCtl.text.trim() != _sheet.name ||
      _hitDiceCtl.text.trim() != _sheet.hitDice ||
      (_sheet.experiencePoints != null &&
          _xpCtl.text.trim() != _sheet.experiencePoints.toString());

  // ── sections ────────────────────────────────────────────────────────

  Widget _sectionCard(String title, List<Widget> children) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 12),
            ...children,
          ],
        ),
      ),
    );
  }

  Widget _identitySection() {
    return _sectionCard('Identity', [
      TextField(
        controller: _nameCtl,
        decoration: const InputDecoration(
          labelText: 'Name',
          border: OutlineInputBorder(),
        ),
        onChanged: (_) => _dirty = true,
      ),
      const SizedBox(height: 12),
      Row(
        children: [
          Expanded(
            child: ContentPickerField(
              label: 'Species',
              value: _sheet.race,
              onTap: _pickSpecies,
              onCleared: () => _mutate(_sheet.copyWith(race: '')),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: ContentPickerField(
              label: 'Background',
              value: _sheet.background,
              onTap: _pickBackground,
              onCleared: () => _mutate(_sheet.copyWith(background: '')),
            ),
          ),
        ],
      ),
      const SizedBox(height: 12),
      _experienceField(),
      const SizedBox(height: 12),
      _alignmentDropdown(),
    ]);
  }

  // Optional XP tracking. The toggle enables the field (seeding 0) and
  // clears it back to null when turned off, mirroring the nullable model.
  Widget _experienceField() {
    final tracking = _sheet.experiencePoints != null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SwitchListTile(
          title: const Text('Track experience points'),
          subtitle: const Text('For campaigns that advance by XP'),
          contentPadding: EdgeInsets.zero,
          value: tracking,
          onChanged: (on) {
            if (on) {
              _xpCtl.text = '0';
              _mutate(_sheet.copyWith(experiencePoints: 0));
            } else {
              _xpCtl.clear();
              _mutate(_sheet.copyWith(clearExperiencePoints: true));
            }
          },
        ),
        if (tracking)
          TextField(
            controller: _xpCtl,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Experience Points',
              border: OutlineInputBorder(),
            ),
            onChanged: (_) => _dirty = true,
          ),
      ],
    );
  }

  Widget _alignmentDropdown() {
    // Nine canonical alignments only ("Unaligned" is creature-only);
    // off-list stored values show as unset — the server rejects them.
    final current = _sheet.alignment;
    return DropdownButtonFormField<String>(
      initialValue: kCanonicalAlignments.contains(current) ? current : null,
      // The field gets half a Row; without isExpanded the selected
      // value sizes to its natural width and overflows.
      isExpanded: true,
      decoration: const InputDecoration(
        labelText: 'Alignment',
        border: OutlineInputBorder(),
      ),
      items: [
        for (final o in kCanonicalAlignments)
          DropdownMenuItem(
            value: o,
            child: Text(o, overflow: TextOverflow.ellipsis),
          ),
      ],
      onChanged: (v) => _mutate(_sheet.copyWith(alignment: v ?? '')),
    );
  }

  Future<Map<String, dynamic>?> _pickContent(
    String table,
    String title, {
    String? where,
    List<Object?>? whereArgs,
  }) {
    return showContentPicker(
      context: context,
      content: _content,
      table: table,
      title: title,
      where: where,
      whereArgs: whereArgs,
    );
  }

  Future<void> _pickSpecies() async {
    final record = await _pickContent('species', 'Choose a species');
    if (record == null) return;
    _mutate(_sheet.copyWith(race: record['name'] as String? ?? ''));
  }

  // ── classes & subclasses ────────────────────────────────────────────

  Map<String, dynamic>? _baseClassRecord(String name) {
    for (final r in _classRows) {
      if (r['subclass_of'] == null && r['name'] == name) return r;
    }
    return null;
  }

  int _subclassCount(Object? parentUuid) => parentUuid == null
      ? 0
      : _classRows.where((r) => r['subclass_of'] == parentUuid).length;

  List<ClassEntry> _replaceEntry(int index, ClassEntry entry) {
    final next = [..._sheet.classes];
    next[index] = entry;
    return next;
  }

  // Base classes not already on the sheet (excludeIndex exempts the row
  // being re-picked).
  ({String where, List<Object?> args}) _basePickerFilter(int excludeIndex) {
    final taken = [
      for (var i = 0; i < _sheet.classes.length; i++)
        if (i != excludeIndex) _sheet.classes[i].name,
    ];
    if (taken.isEmpty) {
      return (where: 'subclass_of IS NULL', args: const []);
    }
    final holes = List.filled(taken.length, '?').join(', ');
    return (
      where: 'subclass_of IS NULL AND name NOT IN ($holes)',
      args: taken,
    );
  }

  // Starting-class grants (saves, first-level HP while untouched):
  // documents, never enforces.
  CharacterSheet _applyClassGrants(
    CharacterSheet sheet,
    Map<String, dynamic> record,
  ) {
    final saves = record['prof_saving_throws'];
    final hitDie = record['hit_dice'];
    var next = sheet.copyWith(
      savingThrowProficiencies: saves is List
          ? Ability.parseWireSet(saves)
          : null,
    );
    if (next.maxHp <= 1 && hitDie is num) {
      final hp =
          (hitDie.truncate() +
                  abilityModifier(score: next.abilityScore(Ability.constitution)))
              .clamp(1, 999);
      next = next.copyWith(maxHp: hp, currentHp: hp);
    }
    return next;
  }

  void _seedHitDice(Map<String, dynamic> record) {
    final hitDie = record['hit_dice'];
    if (_hitDiceCtl.text.trim().isEmpty && hitDie is num) {
      _hitDiceCtl.text = '1d${hitDie.truncate()}';
      _dirty = true;
    }
  }

  // Held-class prereq records: the entry's pick-time snapshot wins (it
  // travels with the sheet), else live content, else a name stub.
  Map<String, dynamic> _prereqRecord(ClassEntry entry) {
    if (entry.primaryAbilities.isNotEmpty) {
      return {'name': entry.name, 'primary_abilities': entry.primaryAbilities};
    }
    return _baseClassRecord(entry.name) ?? {'name': entry.name};
  }

  // Extracts the snapshot groups from a picked class record.
  List<List<String>> _snapshotPrereqs(Map<String, dynamic> record) => [
    for (final group in record['primary_abilities'] as List<dynamic>? ?? [])
      if (group is List) [for (final a in group) a as String],
  ];

  // Hard block (shared core check): every class in the set must meet its
  // primary-ability prerequisite (13+) before another class is added.
  Future<bool> _passesPrereqs(
    Map<String, dynamic> newRecord,
    int excludeIndex,
  ) async {
    final others = [
      for (var i = 0; i < _sheet.classes.length; i++)
        if (i != excludeIndex) _prereqRecord(_sheet.classes[i]),
    ];
    if (others.isEmpty) return true;
    final abilities = {
      for (final e in _sheet.abilities.entries) e.key.name: e.value,
    };
    final raw = checkMulticlass(
      abilitiesJson: jsonEncode(abilities),
      classRecordsJson: jsonEncode([newRecord, ...others]),
    );
    final failed = [
      for (final r in (jsonDecode(raw) as List<dynamic>))
        if ((r as Map<String, dynamic>)['ok'] != true) r,
    ];
    if (failed.isEmpty) return true;
    if (mounted) {
      await showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Multiclass requirements not met'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final r in failed)
                Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: Text(
                    '${r['name']} requires '
                    '${(r['requirement'] as String?)?.isNotEmpty == true ? r['requirement'] : 'an ability score of 13'}.',
                  ),
                ),
            ],
          ),
          actions: [
            FilledButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('OK'),
            ),
          ],
        ),
      );
    }
    return false;
  }

  Future<void> _addClass() async {
    final filter = _basePickerFilter(-1);
    final record = await _pickContent(
      'class',
      _sheet.classes.isEmpty ? 'Choose a class' : 'Add a class',
      where: filter.where,
      whereArgs: filter.args,
    );
    if (record == null || !mounted) return;
    if (!await _passesPrereqs(record, -1)) return;
    final first = _sheet.classes.isEmpty;
    var next = _sheet.copyWith(
      classes: [
        ..._sheet.classes,
        ClassEntry(
          name: record['name'] as String? ?? '',
          starting: first,
          primaryAbilities: _snapshotPrereqs(record),
        ),
      ],
    );
    if (first) next = _applyClassGrants(next, record);
    _mutate(next);
    if (first) _seedHitDice(record);
  }

  Future<void> _repickClass(int index) async {
    final filter = _basePickerFilter(index);
    final record = await _pickContent(
      'class',
      'Choose a class',
      where: filter.where,
      whereArgs: filter.args,
    );
    if (record == null || !mounted) return;
    if (!await _passesPrereqs(record, index)) return;
    final entry = _sheet.classes[index];
    var next = _sheet.copyWith(
      classes: _replaceEntry(
        index,
        entry.copyWith(
          name: record['name'] as String? ?? '',
          subclass: '',
          primaryAbilities: _snapshotPrereqs(record),
        ),
      ),
    );
    if (entry.starting) next = _applyClassGrants(next, record);
    _mutate(next);
    if (entry.starting) _seedHitDice(record);
  }

  Future<void> _removeClass(int index) async {
    if (_sheet.classes.length <= 1) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('A character needs at least one class.'),
          duration: Duration(seconds: 2),
        ),
      );
      return;
    }
    final entry = _sheet.classes[index];
    if (!await _confirmRemove('class', entry.name) || !mounted) return;
    final next = [..._sheet.classes]..removeAt(index);
    if (!next.any((c) => c.starting)) {
      next[0] = next[0].copyWith(starting: true);
    }
    _mutate(_sheet.copyWith(classes: next));
  }

  void _setStartingClass(int index) {
    _mutate(
      _sheet.copyWith(
        classes: [
          for (var i = 0; i < _sheet.classes.length; i++)
            _sheet.classes[i].copyWith(starting: i == index),
        ],
      ),
    );
  }

  Future<void> _pickSubclass(int index) async {
    final entry = _sheet.classes[index];
    final parent = _baseClassRecord(entry.name);
    if (parent == null) return;
    final record = await _pickContent(
      'class',
      'Choose a ${entry.name} subclass',
      where: 'subclass_of = ?',
      whereArgs: [parent['uuid']],
    );
    if (record == null) return;
    _setSubclass(index, record['name'] as String? ?? '');
  }

  void _setSubclass(int index, String name) {
    _mutate(
      _sheet.copyWith(
        classes: _replaceEntry(
          index,
          _sheet.classes[index].copyWith(subclass: name),
        ),
      ),
    );
  }

  Widget _classesSection() {
    final startingIndex = _sheet.classes.indexWhere((c) => c.starting);
    return _sectionCard('Classes', [
      Row(
        children: [
          Expanded(
            child: _StatBadge(label: 'Level', value: '${_derived.level}'),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: _StatBadge(
              label: 'Proficiency',
              value: CharacterSheet.formatBonus(_derived.proficiencyBonus),
            ),
          ),
        ],
      ),
      const SizedBox(height: 12),
      RadioGroup<int>(
        groupValue: startingIndex,
        onChanged: (v) {
          if (v != null) _setStartingClass(v);
        },
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (var i = 0; i < _sheet.classes.length; i++) ...[
              _classRow(i, startingIndex),
              const SizedBox(height: 12),
            ],
          ],
        ),
      ),
      Align(
        alignment: Alignment.centerLeft,
        child: TextButton.icon(
          onPressed: _addClass,
          icon: const Icon(Icons.add),
          label: const Text('Add class'),
        ),
      ),
    ]);
  }

  Widget _classRow(int index, int startingIndex) {
    final entry = _sheet.classes[index];
    final parent = _baseClassRecord(entry.name);
    final subclasses = _subclassCount(parent?['uuid']);
    final unlocked = entry.level >= 3;
    final subclassEnabled = unlocked && subclasses > 0;
    final String? hint;
    if (!unlocked) {
      hint = 'Subclass unlocks at level 3';
    } else if (parent == null) {
      hint = 'Class content not found';
    } else if (subclasses == 0) {
      hint = 'No subclasses available';
    } else {
      hint = null;
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Tooltip(
              message: 'Starting class — determines first-level stats',
              child: Radio<int>(value: index),
            ),
            Expanded(
              child: ContentPickerField(
                label: index == startingIndex ? 'Class (starting)' : 'Class',
                value: entry.name,
                onTap: () => _repickClass(index),
                onCleared: () => _removeClass(index),
              ),
            ),
            const SizedBox(width: 12),
            _NumberStepper(
              label: 'Level',
              compact: true,
              value: entry.level,
              min: 1,
              max: 20,
              onChanged: (v) => _mutate(
                _sheet.copyWith(
                  classes: _replaceEntry(index, entry.copyWith(level: v)),
                ),
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.only(left: 48),
          child: ContentPickerField(
            label: 'Subclass',
            value: entry.subclass,
            enabled: subclassEnabled,
            helperText: hint,
            onTap: () => _pickSubclass(index),
            onCleared: entry.subclass.isEmpty
                ? null
                : () => _setSubclass(index, ''),
          ),
        ),
      ],
    );
  }

  Future<void> _pickBackground() async {
    final record = await _pickContent('background', 'Choose a background');
    if (record == null) return;
    _mutate(_sheet.copyWith(background: record['name'] as String? ?? ''));
  }

  Widget _abilitiesSection() {
    return _sectionCard('Abilities', [
      Wrap(
        spacing: 12,
        runSpacing: 12,
        children: [
          for (final a in Ability.values)
            SizedBox(
              width: 100,
              child: Column(
                children: [
                  Text(a.abbr, style: Theme.of(context).textTheme.labelLarge),
                  _NumberStepper(
                    label: '',
                    compact: true,
                    value: _sheet.abilityScore(a),
                    min: 1,
                    max: 30,
                    onChanged: (v) => _mutate(
                      _sheet.copyWith(abilities: {..._sheet.abilities, a: v}),
                    ),
                  ),
                  Text(
                    CharacterSheet.formatBonus(_abilityMods[a.name]!),
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ],
              ),
            ),
        ],
      ),
    ]);
  }

  Widget _combatSection() {
    return _sectionCard('Combat', [
      Row(
        children: [
          Expanded(
            child: _NumberStepper(
              label: 'Armor class',
              value: _sheet.armorClass,
              min: 0,
              max: 40,
              onChanged: (v) => _mutate(_sheet.copyWith(armorClass: v)),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: _StatBadge(
              label: 'Initiative',
              value: CharacterSheet.formatBonus(_derived.initiative),
            ),
          ),
        ],
      ),
      const SizedBox(height: 12),
      Row(
        children: [
          Expanded(
            child: _NumberStepper(
              label: 'Speed',
              value: _sheet.speed,
              min: 0,
              max: 200,
              step: 5,
              onChanged: (v) => _mutate(_sheet.copyWith(speed: v)),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: _StatBadge(
              label: 'Passive Perception',
              value: '${_derived.passivePerception}',
            ),
          ),
        ],
      ),
      const SizedBox(height: 12),
      Row(
        children: [
          Expanded(
            child: _NumberStepper(
              label: 'Current HP',
              value: _sheet.currentHp,
              min: 0,
              max: 999,
              onChanged: (v) => _mutate(_sheet.copyWith(currentHp: v)),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: _NumberStepper(
              label: 'Max HP',
              value: _sheet.maxHp,
              min: 1,
              max: 999,
              onChanged: (v) => _mutate(_sheet.copyWith(maxHp: v)),
            ),
          ),
        ],
      ),
      const SizedBox(height: 12),
      TextField(
        controller: _hitDiceCtl,
        readOnly: true,
        decoration: const InputDecoration(
          labelText: 'Hit dice',
          hintText: 'e.g. 3d8',
          border: OutlineInputBorder(),
        ),
        onTap: () async {
          final expr = await showDiceExpressionBuilder(
            context,
            initial: _hitDiceCtl.text,
          );
          if (expr == null) return;
          setState(() {
            _hitDiceCtl.text = expr;
            _dirty = true;
          });
        },
      ),
    ]);
  }

  Widget _savesSection() {
    return _sectionCard('Saving throws', [
      for (final a in Ability.values)
        CheckboxListTile(
          dense: true,
          controlAffinity: ListTileControlAffinity.leading,
          contentPadding: EdgeInsets.zero,
          value: _sheet.savingThrowProficiencies.contains(a),
          title: Text(a.label),
          secondary: Text(
            CharacterSheet.formatBonus(_saveBonuses[a.name]!),
            style: Theme.of(context).textTheme.titleMedium,
          ),
          onChanged: (checked) {
            final next = {..._sheet.savingThrowProficiencies};
            checked == true ? next.add(a) : next.remove(a);
            _mutate(_sheet.copyWith(savingThrowProficiencies: next));
          },
        ),
    ]);
  }

  Widget _skillsSection() {
    return _sectionCard('Skills', [
      for (final s in Skill.values)
        CheckboxListTile(
          dense: true,
          controlAffinity: ListTileControlAffinity.leading,
          contentPadding: EdgeInsets.zero,
          value: _sheet.skillProficiencies.contains(s),
          title: Text('${s.label} (${s.ability.abbr})'),
          secondary: Text(
            CharacterSheet.formatBonus(_skillBonuses[s.name]!),
            style: Theme.of(context).textTheme.titleMedium,
          ),
          onChanged: (checked) {
            final next = {..._sheet.skillProficiencies};
            checked == true ? next.add(s) : next.remove(s);
            _mutate(_sheet.copyWith(skillProficiencies: next));
          },
        ),
    ]);
  }

  Widget _languagesSection() {
    // Installed language names, unioned with any already on the sheet that
    // are no longer installed — so a stored language is never silently lost.
    final installed = [for (final r in _languages) r['name'] as String];
    final names = {...installed, ..._sheet.languages}.toList()..sort();
    return _sectionCard('Languages', [
      if (names.isEmpty)
        Text('No languages installed.',
            style: Theme.of(context).textTheme.bodyMedium)
      else
        Wrap(
          spacing: 8,
          runSpacing: 4,
          children: [
            for (final name in names)
              FilterChip(
                label: Text(name),
                selected: _sheet.languages.contains(name),
                onSelected: (on) {
                  final next = {..._sheet.languages};
                  on ? next.add(name) : next.remove(name);
                  _mutate(_sheet.copyWith(languages: next));
                },
              ),
          ],
        ),
    ]);
  }

  Widget _equipmentSection() {
    return _sectionCard('Equipment', [
      if (_sheet.equipment.isEmpty)
        Text('Nothing yet.', style: Theme.of(context).textTheme.bodyMedium),
      for (final (i, item) in _sheet.equipment.indexed)
        ListTile(
          dense: true,
          contentPadding: EdgeInsets.zero,
          title: Text(
            item.quantity > 1 ? '${item.name} ×${item.quantity}' : item.name,
          ),
          subtitle: item.notes.isEmpty ? null : Text(item.notes),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (_attunableItems.contains(item.name))
                Tooltip(
                  message: 'Attuned (max 3)',
                  child: Checkbox(
                    value: item.attuned,
                    onChanged: (v) => _setAttuned(i, v == true),
                  ),
                ),
              IconButton(
                icon: const Icon(Icons.remove_circle_outline),
                tooltip: 'Remove',
                onPressed: () async {
                  if (!await _confirmRemove('item', item.name) || !mounted) {
                    return;
                  }
                  final next = [..._sheet.equipment]..removeAt(i);
                  _mutate(_sheet.copyWith(equipment: next));
                },
              ),
            ],
          ),
        ),
      Align(
        alignment: Alignment.centerLeft,
        child: TextButton.icon(
          onPressed: _addEquipment,
          icon: const Icon(Icons.add),
          label: const Text('Add item'),
        ),
      ),
    ]);
  }

  Widget _spellsSection() {
    final sorted = [..._sheet.spells]
      ..sort(
        (a, b) => a.level != b.level
            ? a.level.compareTo(b.level)
            : a.name.compareTo(b.name),
      );
    return _sectionCard('Spells', [
      if (sorted.isEmpty)
        Text(
          'No spells recorded.',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
      for (final spell in sorted)
        ListTile(
          dense: true,
          contentPadding: EdgeInsets.zero,
          leading: CircleAvatar(
            radius: 14,
            child: Text(spell.level == 0 ? 'C' : '${spell.level}'),
          ),
          title: Text(spell.name),
          subtitle: spell.notes.isEmpty ? null : Text(spell.notes),
          trailing: IconButton(
            icon: const Icon(Icons.remove_circle_outline),
            tooltip: 'Remove',
            onPressed: () async {
              if (!await _confirmRemove('spell', spell.name) || !mounted) {
                return;
              }
              final next = [..._sheet.spells]..remove(spell);
              _mutate(_sheet.copyWith(spells: next));
            },
          ),
        ),
      Align(
        alignment: Alignment.centerLeft,
        child: TextButton.icon(
          onPressed: _addSpell,
          icon: const Icon(Icons.add),
          label: const Text('Add spell'),
        ),
      ),
    ]);
  }

  void _setAttuned(int index, bool attuned) {
    if (attuned &&
        _sheet.equipment.where((e) => e.attuned).length >= 3) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Too many items selected for attunement — limit is 3.'),
        ),
      );
      return;
    }
    final next = [..._sheet.equipment];
    next[index] = next[index].copyWith(attuned: attuned);
    _mutate(_sheet.copyWith(equipment: next));
  }

  Future<void> _addEquipment() async {
    final record = await _pickContent('item', 'Add an item');
    if (record == null || !mounted) return;
    final name = record['name'] as String? ?? '';

    final qtyCtl = TextEditingController(text: '1');
    final notesCtl = TextEditingController();
    final added = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(name),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: qtyCtl,
              decoration: const InputDecoration(labelText: 'Quantity'),
              keyboardType: TextInputType.number,
              autofocus: true,
            ),
            TextField(
              controller: notesCtl,
              decoration: const InputDecoration(labelText: 'Notes (optional)'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Add'),
          ),
        ],
      ),
    );
    if (added != true || name.isEmpty) return;
    final item = EquipmentItem(
      name: name,
      quantity: int.tryParse(qtyCtl.text.trim()) ?? 1,
      notes: notesCtl.text.trim(),
    );
    _mutate(_sheet.copyWith(equipment: [..._sheet.equipment, item]));
  }

  Future<void> _addSpell() async {
    final record = await _pickContent('spell', 'Add a spell');
    if (record == null) return;
    final name = record['name'] as String? ?? '';
    if (name.isEmpty) return;
    final spell = SpellEntry(name: name, level: record['level'] as int? ?? 0);
    _mutate(_sheet.copyWith(spells: [..._sheet.spells, spell]));
  }
}

// ── small field widgets ───────────────────────────────────────────────

class _NumberStepper extends StatelessWidget {
  const _NumberStepper({
    required this.label,
    required this.value,
    required this.onChanged,
    this.min = 0,
    this.max = 99,
    this.step = 1,
    this.compact = false,
  });

  final String label;
  final int value;
  final int min;
  final int max;
  final int step;
  final bool compact;
  final ValueChanged<int> onChanged;

  Widget _stepButton(IconData icon, VoidCallback? onPressed) {
    // Default IconButtons have a 48px minimum touch target; two of
    // those plus the value column overflow the 100px ability tiles.
    // Compact mode trims them to fit while keeping a tappable area.
    if (!compact) {
      return IconButton(
        icon: Icon(icon),
        visualDensity: VisualDensity.compact,
        onPressed: onPressed,
      );
    }
    return IconButton(
      icon: Icon(icon, size: 18),
      onPressed: onPressed,
      // M3 IconButton inflates layout to the 48px tap target unless the
      // style explicitly shrink-wraps; plain `constraints` is ignored.
      style: IconButton.styleFrom(
        padding: EdgeInsets.zero,
        minimumSize: const Size(30, 30),
        fixedSize: const Size(30, 30),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final row = Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _stepButton(
          Icons.remove,
          value - step >= min ? () => onChanged(value - step) : null,
        ),
        SizedBox(
          width: compact ? 28 : 40,
          child: Text(
            '$value',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        _stepButton(
          Icons.add,
          value + step <= max ? () => onChanged(value + step) : null,
        ),
      ],
    );
    if (label.isEmpty) return row;
    return Column(
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        row,
      ],
    );
  }
}

class _StatBadge extends StatelessWidget {
  const _StatBadge({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        const SizedBox(height: 8),
        Text(value, style: Theme.of(context).textTheme.titleLarge),
      ],
    );
  }
}
