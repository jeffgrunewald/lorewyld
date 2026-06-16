// Serialization tests for the local character sheet.
//
// The derived 5e math (modifiers, proficiency, saves, skills, initiative,
// passive perception) now lives in the shared Rust core and is tested
// there (`shared/domain/src/sheet.rs`), including a test that parses this
// sheet's exact `toJson()` wire shape. Driving the FFI bindings from a
// Flutter test requires the native library built by cargokit, so the
// numeric equivalence is asserted Rust-side rather than duplicated here.

import 'package:flutter_test/flutter_test.dart';

import 'package:lorewyld/types/character.dart';

CharacterSheet _sheet({
  int level = 1,
  Map<Ability, int>? abilities,
  Set<Ability> saves = const {},
  Set<Skill> skills = const {},
}) {
  final now = DateTime.utc(2026, 1, 1);
  return CharacterSheet(
    uuid: 'test-uuid',
    name: 'Thistle',
    level: level,
    abilities: abilities ?? CharacterSheet.defaultAbilities(),
    savingThrowProficiencies: saves,
    skillProficiencies: skills,
    createdAt: now,
    updatedAt: now,
  );
}

void main() {
  test('json round-trip preserves the full sheet', () {
    final original = _sheet(
      level: 7,
      abilities: {
        for (final a in Ability.values) a: 8 + Ability.values.indexOf(a),
      },
      saves: {Ability.constitution, Ability.intelligence},
      skills: {Skill.arcana, Skill.sleightOfHand},
    ).copyWith(
      race: 'Gnome',
      className: 'Wizard',
      background: 'Sage',
      alignment: 'NG',
      armorClass: 15,
      speed: 25,
      maxHp: 38,
      currentHp: 22,
      hitDice: '7d6',
      equipment: const [
        EquipmentItem(name: 'Dagger', quantity: 2, notes: 'silvered'),
      ],
      spells: const [
        SpellEntry(name: 'Fireball', level: 3, notes: '8d6'),
        SpellEntry(name: 'Prestidigitation'),
      ],
    );

    final restored = CharacterSheet.fromJson(original.toJson());

    expect(restored.toJson(), original.toJson());
    expect(restored.equipment.single.notes, 'silvered');
    expect(restored.spells.first.level, 3);
  });

  test('parseWireSet maps content ability names, skipping unknowns', () {
    // Shape of a class record's prof_saving_throws in the SRD bundle.
    expect(Ability.parseWireSet(['dexterity', 'strength']), {
      Ability.dexterity,
      Ability.strength,
    });
    // Content is external data — unrecognized entries are dropped, not
    // thrown on.
    expect(Ability.parseWireSet(['wisdom', 'luck', 42]), {Ability.wisdom});
    expect(Ability.parseWireSet([]), isEmpty);
  });
}
