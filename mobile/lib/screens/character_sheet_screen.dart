// Full 5e character sheet, stored locally. Documents and computes —
// modifiers, proficiency, save/skill bonuses are derived live — but
// never blocks a value the player wants to write down.
//
// Edits autosave when the screen is popped; the save icon persists
// immediately.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../types/character.dart';
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

  late final TextEditingController _nameCtl;
  late final TextEditingController _raceCtl;
  late final TextEditingController _classCtl;
  late final TextEditingController _backgroundCtl;
  late final TextEditingController _alignmentCtl;
  late final TextEditingController _hitDiceCtl;

  @override
  void initState() {
    super.initState();
    _sheet = widget.sheet;
    _nameCtl = TextEditingController(text: _sheet.name);
    _raceCtl = TextEditingController(text: _sheet.race);
    _classCtl = TextEditingController(text: _sheet.className);
    _backgroundCtl = TextEditingController(text: _sheet.background);
    _alignmentCtl = TextEditingController(text: _sheet.alignment);
    _hitDiceCtl = TextEditingController(text: _sheet.hitDice);
  }

  @override
  void dispose() {
    _nameCtl.dispose();
    _raceCtl.dispose();
    _classCtl.dispose();
    _backgroundCtl.dispose();
    _alignmentCtl.dispose();
    _hitDiceCtl.dispose();
    super.dispose();
  }

  void _mutate(CharacterSheet next) {
    setState(() {
      _sheet = next;
      _dirty = true;
    });
  }

  CharacterSheet _withTextFields() => _sheet.copyWith(
        name: _nameCtl.text.trim().isEmpty ? _sheet.name : _nameCtl.text.trim(),
        race: _raceCtl.text.trim(),
        className: _classCtl.text.trim(),
        background: _backgroundCtl.text.trim(),
        alignment: _alignmentCtl.text.trim(),
        hitDice: _hitDiceCtl.text.trim(),
      );

  Future<void> _save() async {
    final next = await widget.store.saveCharacter(_withTextFields());
    if (!mounted) return;
    setState(() {
      _sheet = next;
      _dirty = false;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Saved.'), duration: Duration(seconds: 1)),
    );
  }

  Future<void> _delete() async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete character?'),
        content: Text(
            '"${_sheet.name}" and their notes will be permanently removed.'),
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
        builder: (_) => CharacterNotesScreen(
          store: widget.store,
          character: _sheet,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      // Autosave on back: text fields are committed first so nothing
      // typed is lost.
      onPopInvokedWithResult: (didPop, _) {
        if (didPop && (_dirty || _textFieldsChanged())) {
          widget.store.saveCharacter(_withTextFields());
        }
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
            _abilitiesSection(),
            const SizedBox(height: 16),
            _combatSection(),
            const SizedBox(height: 16),
            _savesSection(),
            const SizedBox(height: 16),
            _skillsSection(),
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
      _raceCtl.text.trim() != _sheet.race ||
      _classCtl.text.trim() != _sheet.className ||
      _backgroundCtl.text.trim() != _sheet.background ||
      _alignmentCtl.text.trim() != _sheet.alignment ||
      _hitDiceCtl.text.trim() != _sheet.hitDice;

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
            labelText: 'Name', border: OutlineInputBorder()),
        onChanged: (_) => _dirty = true,
      ),
      const SizedBox(height: 12),
      Row(children: [
        Expanded(
          child: TextField(
            controller: _raceCtl,
            decoration: const InputDecoration(
                labelText: 'Race', border: OutlineInputBorder()),
            onChanged: (_) => _dirty = true,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: TextField(
            controller: _classCtl,
            decoration: const InputDecoration(
                labelText: 'Class', border: OutlineInputBorder()),
            onChanged: (_) => _dirty = true,
          ),
        ),
      ]),
      const SizedBox(height: 12),
      Row(children: [
        Expanded(
          child: _NumberStepper(
            label: 'Level',
            value: _sheet.level,
            min: 1,
            max: 20,
            onChanged: (v) => _mutate(_sheet.copyWith(level: v)),
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: _StatBadge(
            label: 'Proficiency',
            value: CharacterSheet.formatBonus(_sheet.proficiencyBonus),
          ),
        ),
      ]),
      const SizedBox(height: 12),
      Row(children: [
        Expanded(
          child: TextField(
            controller: _backgroundCtl,
            decoration: const InputDecoration(
                labelText: 'Background', border: OutlineInputBorder()),
            onChanged: (_) => _dirty = true,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: TextField(
            controller: _alignmentCtl,
            decoration: const InputDecoration(
                labelText: 'Alignment', border: OutlineInputBorder()),
            onChanged: (_) => _dirty = true,
          ),
        ),
      ]),
    ]);
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
              child: Column(children: [
                Text(a.abbr,
                    style: Theme.of(context).textTheme.labelLarge),
                _NumberStepper(
                  label: '',
                  compact: true,
                  value: _sheet.abilityScore(a),
                  min: 1,
                  max: 30,
                  onChanged: (v) => _mutate(_sheet.copyWith(
                    abilities: {..._sheet.abilities, a: v},
                  )),
                ),
                Text(
                  CharacterSheet.formatBonus(_sheet.abilityModifier(a)),
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ]),
            ),
        ],
      ),
    ]);
  }

  Widget _combatSection() {
    return _sectionCard('Combat', [
      Row(children: [
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
            value: CharacterSheet.formatBonus(_sheet.initiativeBonus),
          ),
        ),
      ]),
      const SizedBox(height: 12),
      Row(children: [
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
            value: '${_sheet.passivePerception}',
          ),
        ),
      ]),
      const SizedBox(height: 12),
      Row(children: [
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
      ]),
      const SizedBox(height: 12),
      TextField(
        controller: _hitDiceCtl,
        decoration: const InputDecoration(
          labelText: 'Hit dice',
          hintText: 'e.g. 3d8',
          border: OutlineInputBorder(),
        ),
        onChanged: (_) => _dirty = true,
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
            CharacterSheet.formatBonus(_sheet.savingThrowBonus(a)),
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
            CharacterSheet.formatBonus(_sheet.skillBonus(s)),
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

  Widget _equipmentSection() {
    return _sectionCard('Equipment', [
      if (_sheet.equipment.isEmpty)
        Text('Nothing yet.',
            style: Theme.of(context).textTheme.bodyMedium),
      for (final (i, item) in _sheet.equipment.indexed)
        ListTile(
          dense: true,
          contentPadding: EdgeInsets.zero,
          title: Text(
              item.quantity > 1 ? '${item.name} ×${item.quantity}' : item.name),
          subtitle: item.notes.isEmpty ? null : Text(item.notes),
          trailing: IconButton(
            icon: const Icon(Icons.remove_circle_outline),
            tooltip: 'Remove',
            onPressed: () {
              final next = [..._sheet.equipment]..removeAt(i);
              _mutate(_sheet.copyWith(equipment: next));
            },
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
      ..sort((a, b) => a.level != b.level
          ? a.level.compareTo(b.level)
          : a.name.compareTo(b.name));
    return _sectionCard('Spells', [
      if (sorted.isEmpty)
        Text('No spells recorded.',
            style: Theme.of(context).textTheme.bodyMedium),
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
            onPressed: () {
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

  Future<void> _addEquipment() async {
    final nameCtl = TextEditingController();
    final qtyCtl = TextEditingController(text: '1');
    final notesCtl = TextEditingController();
    final added = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Add item'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameCtl,
              decoration: const InputDecoration(labelText: 'Name'),
              autofocus: true,
            ),
            TextField(
              controller: qtyCtl,
              decoration: const InputDecoration(labelText: 'Quantity'),
              keyboardType: TextInputType.number,
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
              child: const Text('Cancel')),
          FilledButton(
              onPressed: () => Navigator.pop(ctx, true),
              child: const Text('Add')),
        ],
      ),
    );
    if (added != true || nameCtl.text.trim().isEmpty) return;
    final item = EquipmentItem(
      name: nameCtl.text.trim(),
      quantity: int.tryParse(qtyCtl.text.trim()) ?? 1,
      notes: notesCtl.text.trim(),
    );
    _mutate(_sheet.copyWith(equipment: [..._sheet.equipment, item]));
  }

  Future<void> _addSpell() async {
    final nameCtl = TextEditingController();
    final notesCtl = TextEditingController();
    var level = 0;
    final added = await showDialog<bool>(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: const Text('Add spell'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameCtl,
                decoration: const InputDecoration(labelText: 'Name'),
                autofocus: true,
              ),
              DropdownButtonFormField<int>(
                initialValue: level,
                decoration: const InputDecoration(labelText: 'Level'),
                items: [
                  const DropdownMenuItem(value: 0, child: Text('Cantrip')),
                  for (var l = 1; l <= 9; l++)
                    DropdownMenuItem(value: l, child: Text('Level $l')),
                ],
                onChanged: (v) => setDialogState(() => level = v ?? 0),
              ),
              TextField(
                controller: notesCtl,
                decoration:
                    const InputDecoration(labelText: 'Notes (optional)'),
              ),
            ],
          ),
          actions: [
            TextButton(
                onPressed: () => Navigator.pop(ctx, false),
                child: const Text('Cancel')),
            FilledButton(
                onPressed: () => Navigator.pop(ctx, true),
                child: const Text('Add')),
          ],
        ),
      ),
    );
    if (added != true || nameCtl.text.trim().isEmpty) return;
    final spell = SpellEntry(
      name: nameCtl.text.trim(),
      level: level,
      notes: notesCtl.text.trim(),
    );
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
    return Column(children: [
      Text(label, style: Theme.of(context).textTheme.labelMedium),
      row,
    ]);
  }
}

class _StatBadge extends StatelessWidget {
  const _StatBadge({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Column(children: [
      Text(label, style: Theme.of(context).textTheme.labelMedium),
      const SizedBox(height: 8),
      Text(value, style: Theme.of(context).textTheme.titleLarge),
    ]);
  }
}
