// Derived-math and serialization tests for the local character sheet.

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
  group('ability modifiers', () {
    test('follow the 5e table including odd scores below 10', () {
      final cases = {1: -5, 3: -4, 8: -1, 9: -1, 10: 0, 11: 0, 15: 2, 20: 5, 30: 10};
      for (final entry in cases.entries) {
        final sheet =
            _sheet(abilities: {Ability.strength: entry.key});
        expect(sheet.abilityModifier(Ability.strength), entry.value,
            reason: 'score ${entry.key}');
      }
    });
  });

  group('proficiency bonus', () {
    test('scales +2 at 1 to +6 at 20', () {
      final cases = {1: 2, 4: 2, 5: 3, 8: 3, 9: 4, 12: 4, 13: 5, 16: 5, 17: 6, 20: 6};
      for (final entry in cases.entries) {
        expect(_sheet(level: entry.key).proficiencyBonus, entry.value,
            reason: 'level ${entry.key}');
      }
    });
  });

  group('saves and skills', () {
    test('proficiency adds the bonus exactly once', () {
      final sheet = _sheet(
        level: 5, // +3 proficiency
        abilities: {Ability.dexterity: 16}, // +3 mod
        saves: {Ability.dexterity},
        skills: {Skill.stealth},
      );
      expect(sheet.savingThrowBonus(Ability.dexterity), 6);
      expect(sheet.skillBonus(Skill.stealth), 6);
      // Unproficient skill on the same ability gets the bare modifier.
      expect(sheet.skillBonus(Skill.acrobatics), 3);
      // Unproficient save on a default-10 ability is flat 0.
      expect(sheet.savingThrowBonus(Ability.wisdom), 0);
    });

    test('initiative and passive perception derive correctly', () {
      final sheet = _sheet(
        level: 1,
        abilities: {Ability.dexterity: 14, Ability.wisdom: 12},
        skills: {Skill.perception},
      );
      expect(sheet.initiativeBonus, 2);
      expect(sheet.passivePerception, 10 + 1 + 2); // 10 + WIS mod + prof
    });
  });

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
    expect(restored.abilityModifier(Ability.charisma),
        original.abilityModifier(Ability.charisma));
    expect(restored.equipment.single.notes, 'silvered');
    expect(restored.spells.first.level, 3);
  });

  test('parseWireSet maps content ability names, skipping unknowns', () {
    // Shape of a class record's prof_saving_throws in the SRD bundle.
    expect(
      Ability.parseWireSet(['dexterity', 'strength']),
      {Ability.dexterity, Ability.strength},
    );
    // Content is external data — unrecognized entries are dropped, not
    // thrown on.
    expect(
      Ability.parseWireSet(['wisdom', 'luck', 42]),
      {Ability.wisdom},
    );
    expect(Ability.parseWireSet([]), isEmpty);
  });
}
