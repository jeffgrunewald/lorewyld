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
  final Map<String, String> documents;
  final Map<String, String> contentModules;

  /// uuid → ordering rank from the size table (Tiny 1 … Gargantuan 6),
  /// for sorting creatures by size.
  final Map<String, int> sizeRanks;

  /// uuid → abbreviated source slug ('srd-2024', 'tob', 'a5e-mm', …)
  /// for the per-record source badge that disambiguates same-name
  /// records from different books.
  final Map<String, String> documentKeys;
  final Map<String, String> moduleSlugs;

  const ContentLookups({
    this.spellSchools = const {},
    this.sizes = const {},
    this.creatureTypes = const {},
    this.itemCategories = const {},
    this.classes = const {},
    this.species = const {},
    this.weaponProperties = const {},
    this.documents = const {},
    this.contentModules = const {},
    this.sizeRanks = const {},
    this.documentKeys = const {},
    this.moduleSlugs = const {},
  });

  /// Abbreviated source label for a record: its document's key, or its
  /// module's slug for tables with no document reference.
  String? sourceSlugOf(Map<String, dynamic> record) =>
      documentKeys[record['document_uuid']] ??
      moduleSlugs[record['content_module_uuid']];

  /// Full source name (document name, falling back to module name).
  String? sourceNameOf(Map<String, dynamic> record) =>
      documents[record['document_uuid']] ??
      contentModules[record['content_module_uuid']];

  static Future<ContentLookups> load(ContentStore content) async {
    final maps = await Future.wait([
      content.lookupNames('spell_school'),
      content.lookupNames('size'),
      content.lookupNames('creature_type'),
      content.lookupNames('item_category'),
      content.lookupNames('class'),
      content.lookupNames('species'),
      content.lookupNames('weapon_property'),
      content.lookupNames('document'),
      content.lookupNames('content_module'),
      content.lookupColumn('document', 'key'),
      content.lookupColumn('content_module', 'slug'),
    ]);
    final sizeRecords = await content.listNamed('size');
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
      documents: maps[7],
      contentModules: maps[8],
      documentKeys: maps[9],
      moduleSlugs: maps[10],
      sizeRanks: {
        for (final s in sizeRecords)
          if (s case {'uuid': final String uuid, 'rank': final int rank})
            uuid: rank,
      },
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
