// Local-first 5e character sheet. Lives only in the on-device database
// for now — the server has no character entity yet.
//
// Platform pillar: the sheet documents and computes (modifiers, save
// and skill bonuses, proficiency from level) but never enforces — any
// score or value the player types is accepted.

enum Ability {
  strength('Strength', 'STR'),
  dexterity('Dexterity', 'DEX'),
  constitution('Constitution', 'CON'),
  intelligence('Intelligence', 'INT'),
  wisdom('Wisdom', 'WIS'),
  charisma('Charisma', 'CHA');

  final String label;
  final String abbr;
  const Ability(this.label, this.abbr);

  static Ability fromWire(String s) =>
      Ability.values.firstWhere((a) => a.name == s);

  /// Parses a content record's ability-name list (e.g. a class's
  /// `prof_saving_throws`), silently skipping unrecognized entries —
  /// content is data we read, not data we control.
  static Set<Ability> parseWireSet(Iterable<dynamic> names) => {
        for (final n in names) ...Ability.values.where((a) => a.name == n),
      };
}

enum Skill {
  acrobatics('Acrobatics', Ability.dexterity),
  animalHandling('Animal Handling', Ability.wisdom),
  arcana('Arcana', Ability.intelligence),
  athletics('Athletics', Ability.strength),
  deception('Deception', Ability.charisma),
  history('History', Ability.intelligence),
  insight('Insight', Ability.wisdom),
  intimidation('Intimidation', Ability.charisma),
  investigation('Investigation', Ability.intelligence),
  medicine('Medicine', Ability.wisdom),
  nature('Nature', Ability.intelligence),
  perception('Perception', Ability.wisdom),
  performance('Performance', Ability.charisma),
  persuasion('Persuasion', Ability.charisma),
  religion('Religion', Ability.intelligence),
  sleightOfHand('Sleight of Hand', Ability.dexterity),
  stealth('Stealth', Ability.dexterity),
  survival('Survival', Ability.wisdom);

  final String label;
  final Ability ability;
  const Skill(this.label, this.ability);

  static Skill fromWire(String s) =>
      Skill.values.firstWhere((k) => k.name == s);
}

class EquipmentItem {
  final String name;
  final int quantity;
  final String notes;

  const EquipmentItem({
    required this.name,
    this.quantity = 1,
    this.notes = '',
  });

  factory EquipmentItem.fromJson(Map<String, dynamic> json) => EquipmentItem(
        name: json['name'] as String,
        quantity: json['quantity'] as int? ?? 1,
        notes: json['notes'] as String? ?? '',
      );

  Map<String, dynamic> toJson() =>
      {'name': name, 'quantity': quantity, 'notes': notes};
}

class SpellEntry {
  final String name;

  /// 0 = cantrip.
  final int level;
  final String notes;

  const SpellEntry({required this.name, this.level = 0, this.notes = ''});

  factory SpellEntry.fromJson(Map<String, dynamic> json) => SpellEntry(
        name: json['name'] as String,
        level: json['level'] as int? ?? 0,
        notes: json['notes'] as String? ?? '',
      );

  Map<String, dynamic> toJson() =>
      {'name': name, 'level': level, 'notes': notes};
}

class CharacterSheet {
  final String uuid;
  final String name;
  final String race;
  final String className;
  final int level;
  final String background;
  final String alignment;
  final Map<Ability, int> abilities;
  final Set<Ability> savingThrowProficiencies;
  final Set<Skill> skillProficiencies;
  final int armorClass;
  final int speed;
  final int maxHp;
  final int currentHp;
  final String hitDice;
  final List<EquipmentItem> equipment;
  final List<SpellEntry> spells;
  final DateTime createdAt;
  final DateTime updatedAt;

  const CharacterSheet({
    required this.uuid,
    required this.name,
    this.race = '',
    this.className = '',
    this.level = 1,
    this.background = '',
    this.alignment = '',
    required this.abilities,
    this.savingThrowProficiencies = const {},
    this.skillProficiencies = const {},
    this.armorClass = 10,
    this.speed = 30,
    this.maxHp = 1,
    this.currentHp = 1,
    this.hitDice = '',
    this.equipment = const [],
    this.spells = const [],
    required this.createdAt,
    required this.updatedAt,
  });

  static Map<Ability, int> defaultAbilities() =>
      {for (final a in Ability.values) a: 10};

  // ── derived 5e math ─────────────────────────────────────────────────

  int abilityScore(Ability a) => abilities[a] ?? 10;

  /// Floor((score - 10) / 2) — Dart's ~/ truncates toward zero, which is
  /// wrong for odd scores below 10, so use floor division explicitly.
  int abilityModifier(Ability a) => ((abilityScore(a) - 10) / 2).floor();

  int get proficiencyBonus => 2 + ((level.clamp(1, 20) - 1) ~/ 4);

  int savingThrowBonus(Ability a) =>
      abilityModifier(a) +
      (savingThrowProficiencies.contains(a) ? proficiencyBonus : 0);

  int skillBonus(Skill s) =>
      abilityModifier(s.ability) +
      (skillProficiencies.contains(s) ? proficiencyBonus : 0);

  int get initiativeBonus => abilityModifier(Ability.dexterity);

  int get passivePerception => 10 + skillBonus(Skill.perception);

  static String formatBonus(int bonus) => bonus >= 0 ? '+$bonus' : '$bonus';

  // ── serialization (stored as one JSON document per character) ──────

  factory CharacterSheet.fromJson(Map<String, dynamic> json) =>
      CharacterSheet(
        uuid: json['uuid'] as String,
        name: json['name'] as String,
        race: json['race'] as String? ?? '',
        className: json['class_name'] as String? ?? '',
        level: json['level'] as int? ?? 1,
        background: json['background'] as String? ?? '',
        alignment: json['alignment'] as String? ?? '',
        abilities: {
          for (final e
              in (json['abilities'] as Map<String, dynamic>? ?? {}).entries)
            Ability.fromWire(e.key): e.value as int,
        },
        savingThrowProficiencies: {
          for (final s
              in json['saving_throw_proficiencies'] as List<dynamic>? ?? [])
            Ability.fromWire(s as String),
        },
        skillProficiencies: {
          for (final s in json['skill_proficiencies'] as List<dynamic>? ?? [])
            Skill.fromWire(s as String),
        },
        armorClass: json['armor_class'] as int? ?? 10,
        speed: json['speed'] as int? ?? 30,
        maxHp: json['max_hp'] as int? ?? 1,
        currentHp: json['current_hp'] as int? ?? 1,
        hitDice: json['hit_dice'] as String? ?? '',
        equipment: (json['equipment'] as List<dynamic>? ?? [])
            .map((e) => EquipmentItem.fromJson(e as Map<String, dynamic>))
            .toList(),
        spells: (json['spells'] as List<dynamic>? ?? [])
            .map((e) => SpellEntry.fromJson(e as Map<String, dynamic>))
            .toList(),
        createdAt: DateTime.parse(json['created_at'] as String),
        updatedAt: DateTime.parse(json['updated_at'] as String),
      );

  Map<String, dynamic> toJson() => {
        'uuid': uuid,
        'name': name,
        'race': race,
        'class_name': className,
        'level': level,
        'background': background,
        'alignment': alignment,
        'abilities': {
          for (final e in abilities.entries) e.key.name: e.value,
        },
        'saving_throw_proficiencies':
            savingThrowProficiencies.map((a) => a.name).toList(),
        'skill_proficiencies':
            skillProficiencies.map((s) => s.name).toList(),
        'armor_class': armorClass,
        'speed': speed,
        'max_hp': maxHp,
        'current_hp': currentHp,
        'hit_dice': hitDice,
        'equipment': equipment.map((e) => e.toJson()).toList(),
        'spells': spells.map((s) => s.toJson()).toList(),
        'created_at': createdAt.toIso8601String(),
        'updated_at': updatedAt.toIso8601String(),
      };

  CharacterSheet copyWith({
    String? name,
    String? race,
    String? className,
    int? level,
    String? background,
    String? alignment,
    Map<Ability, int>? abilities,
    Set<Ability>? savingThrowProficiencies,
    Set<Skill>? skillProficiencies,
    int? armorClass,
    int? speed,
    int? maxHp,
    int? currentHp,
    String? hitDice,
    List<EquipmentItem>? equipment,
    List<SpellEntry>? spells,
    DateTime? updatedAt,
  }) =>
      CharacterSheet(
        uuid: uuid,
        name: name ?? this.name,
        race: race ?? this.race,
        className: className ?? this.className,
        level: level ?? this.level,
        background: background ?? this.background,
        alignment: alignment ?? this.alignment,
        abilities: abilities ?? this.abilities,
        savingThrowProficiencies:
            savingThrowProficiencies ?? this.savingThrowProficiencies,
        skillProficiencies: skillProficiencies ?? this.skillProficiencies,
        armorClass: armorClass ?? this.armorClass,
        speed: speed ?? this.speed,
        maxHp: maxHp ?? this.maxHp,
        currentHp: currentHp ?? this.currentHp,
        hitDice: hitDice ?? this.hitDice,
        equipment: equipment ?? this.equipment,
        spells: spells ?? this.spells,
        createdAt: createdAt,
        updatedAt: updatedAt ?? this.updatedAt,
      );
}
