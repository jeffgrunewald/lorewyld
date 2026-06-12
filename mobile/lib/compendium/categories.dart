// Compendium category descriptors — one per browsable content table —
// shared by the Compendium screens and the content pickers so list
// subtitles render identically everywhere.

import 'package:flutter/material.dart';

import '../services/content_store.dart';

/// Title-cases a snake_case / kebab-case wire value ("chaotic_evil" →
/// "Chaotic Evil").
String humanizeSlug(String s) => s
    .split(RegExp(r'[_-]'))
    .where((w) => w.isNotEmpty)
    .map((w) => w[0].toUpperCase() + w.substring(1))
    .join(' ');

String spellLevelLabel(int level) => level == 0 ? 'Cantrip' : 'Level $level';

String formatChallengeRating(num cr) {
  if (cr == 0.125) return '1/8';
  if (cr == 0.25) return '1/4';
  if (cr == 0.5) return '1/2';
  return cr == cr.truncate() ? '${cr.truncate()}' : '$cr';
}

/// uuid → name maps for the lookup tables referenced by major content,
/// loaded whole once per screen (each is at most a few dozen rows).
class ContentLookups {
  final Map<String, String> spellSchools;
  final Map<String, String> sizes;
  final Map<String, String> creatureTypes;
  final Map<String, String> itemCategories;
  final Map<String, String> classes;
  final Map<String, String> species;
  final Map<String, String> weaponProperties;

  const ContentLookups({
    this.spellSchools = const {},
    this.sizes = const {},
    this.creatureTypes = const {},
    this.itemCategories = const {},
    this.classes = const {},
    this.species = const {},
    this.weaponProperties = const {},
  });

  static Future<ContentLookups> load(ContentStore content) async {
    final maps = await Future.wait([
      content.lookupNames('spell_school'),
      content.lookupNames('size'),
      content.lookupNames('creature_type'),
      content.lookupNames('item_category'),
      content.lookupNames('class'),
      content.lookupNames('species'),
      content.lookupNames('weapon_property'),
    ]);
    // Schools and creature types carry lowercase wire names
    // ("evocation"); title-case them once here. Class/species names are
    // already display-ready — and must not be re-split ("Half-Elf").
    Map<String, String> humanized(Map<String, String> m) =>
        m.map((k, v) => MapEntry(k, humanizeSlug(v)));
    return ContentLookups(
      spellSchools: humanized(maps[0]),
      sizes: maps[1],
      creatureTypes: humanized(maps[2]),
      itemCategories: maps[3],
      classes: maps[4],
      species: maps[5],
      weaponProperties: maps[6],
    );
  }

  String? nameOf(Map<String, String> table, Object? uuid) =>
      uuid is String ? table[uuid] : null;
}

class CompendiumCategory {
  final String table;
  final String label;
  final IconData icon;
  final String? Function(Map<String, dynamic> record, ContentLookups lookups)
      subtitle;

  /// Most names are already display-ready; conditions carry lowercase
  /// wire values ("blinded") that need title-casing.
  final String Function(Map<String, dynamic> record) displayName;

  const CompendiumCategory({
    required this.table,
    required this.label,
    required this.icon,
    required this.subtitle,
    this.displayName = _rawName,
  });

  static String _rawName(Map<String, dynamic> r) => '${r['name']}';
}

String _humanizedName(Map<String, dynamic> r) => humanizeSlug('${r['name']}');

final compendiumCategories = <CompendiumCategory>[
  CompendiumCategory(
    table: 'spell',
    label: 'Spells',
    icon: Icons.auto_fix_high_outlined,
    subtitle: (r, l) => [
      spellLevelLabel(r['level'] as int? ?? 0),
      ?l.nameOf(l.spellSchools, r['school']),
    ].join(' · '),
  ),
  CompendiumCategory(
    table: 'creature',
    label: 'Creatures',
    icon: Icons.cruelty_free_outlined,
    subtitle: (r, l) => [
      if (r['challenge_rating'] case final num cr)
        'CR ${formatChallengeRating(cr)}',
      ?l.nameOf(l.sizes, r['size']),
      ?l.nameOf(l.creatureTypes, r['type']),
    ].join(' · '),
  ),
  CompendiumCategory(
    table: 'class',
    label: 'Classes & subclasses',
    icon: Icons.shield_outlined,
    subtitle: (r, l) => switch (r['subclass_of']) {
      final String parent => 'Subclass of ${l.classes[parent] ?? 'unknown'}',
      _ => r['hit_dice'] != null ? 'Hit die d${r['hit_dice']}' : null,
    },
  ),
  CompendiumCategory(
    table: 'species',
    label: 'Species',
    icon: Icons.emoji_people_outlined,
    subtitle: (r, l) => switch (r['subspecies_of']) {
      final String parent => 'Subspecies of ${l.species[parent] ?? 'unknown'}',
      _ => l.nameOf(l.sizes, r['size']),
    },
  ),
  CompendiumCategory(
    table: 'background',
    label: 'Backgrounds',
    icon: Icons.history_edu_outlined,
    subtitle: (r, l) => null,
  ),
  CompendiumCategory(
    table: 'feat',
    label: 'Feats',
    icon: Icons.military_tech_outlined,
    subtitle: (r, l) =>
        r['has_prerequisite'] == true ? '${r['prerequisite']}' : null,
  ),
  CompendiumCategory(
    table: 'item',
    label: 'Items & gear',
    icon: Icons.backpack_outlined,
    subtitle: (r, l) => [
      ?l.nameOf(l.itemCategories, r['category_uuid']),
      if (r['is_magic'] == true && r['rarity'] is String)
        humanizeSlug(r['rarity'] as String)
      else if (r['cost'] case final String cost)
        '$cost gp',
    ].join(' · '),
  ),
  CompendiumCategory(
    table: 'weapon',
    label: 'Weapons',
    icon: Icons.colorize_outlined,
    subtitle: (r, l) => [
      r['is_simple'] == true ? 'Simple' : 'Martial',
      if (r['damage_dice'] != null) '${r['damage_dice']} ${r['damage_type']}',
    ].join(' · '),
  ),
  CompendiumCategory(
    table: 'armor',
    label: 'Armor',
    icon: Icons.security_outlined,
    subtitle: (r, l) => [
      if (r['category'] case final String c) humanizeSlug(c),
      if (r['ac_display'] case final String ac) 'AC $ac',
    ].join(' · '),
  ),
  CompendiumCategory(
    table: 'condition',
    label: 'Conditions',
    icon: Icons.healing_outlined,
    subtitle: (r, l) => null,
    displayName: _humanizedName,
  ),
  CompendiumCategory(
    table: 'language',
    label: 'Languages',
    icon: Icons.translate_outlined,
    subtitle: (r, l) => null,
  ),
];

CompendiumCategory categoryFor(String table) =>
    compendiumCategories.firstWhere((c) => c.table == table);
