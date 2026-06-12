// Picker/Compendium filter dimensions and sort orders — option
// derivation from loaded records, selection matching, and per-table
// sort comparators.

import 'package:flutter_test/flutter_test.dart';

import 'package:lorewyld/compendium/categories.dart';
import 'package:lorewyld/compendium/filters.dart';

const _lookups = ContentLookups(
  documents: {'doc-51': 'SRD 5.1', 'doc-52': 'SRD 5.2'},
  contentModules: {'mod-srd': 'System Reference Document'},
  itemCategories: {'cat-gear': 'Adventuring Gear', 'cat-weapon': 'Weapon'},
  spellSchools: {'sch-evo': 'Evocation', 'sch-nec': 'Necromancy'},
  creatureTypes: {'ct-dragon': 'Dragon', 'ct-beast': 'Beast'},
  sizeRanks: {'sz-tiny': 1, 'sz-large': 4, 'sz-huge': 5},
);

final _items = [
  {
    'name': 'Rope',
    'document_uuid': 'doc-52',
    'category_uuid': 'cat-gear',
    'rarity': null,
    'cost': '1.00',
  },
  {
    'name': 'Vorpal Sword',
    'document_uuid': 'doc-51',
    'category_uuid': 'cat-weapon',
    'rarity': 'legendary',
    'cost': null,
  },
  {
    'name': 'Bag of Holding',
    'document_uuid': 'doc-52',
    'category_uuid': 'cat-gear',
    'rarity': 'uncommon',
    'cost': '4000.00',
  },
];

final _spells = [
  {
    'name': 'Fire Bolt',
    'document_uuid': 'doc-52',
    'level': 0,
    'school': 'sch-evo',
    'verbal': true,
    'somatic': true,
    'material': false,
  },
  {
    'name': 'Fireball',
    'document_uuid': 'doc-52',
    'level': 3,
    'school': 'sch-evo',
    'verbal': true,
    'somatic': true,
    'material': true,
  },
  {
    'name': 'Animate Dead',
    'document_uuid': 'doc-51',
    'level': 3,
    'school': 'sch-nec',
    'verbal': true,
    'somatic': false,
    'material': true,
  },
];

final _creatures = [
  {
    'name': 'Adult Red Dragon',
    'document_uuid': 'doc-52',
    'type': 'ct-dragon',
    'size': 'sz-huge',
    'challenge_rating': 17.0,
  },
  {
    'name': 'Wolf',
    'document_uuid': 'doc-52',
    'type': 'ct-beast',
    'size': 'sz-large',
    'challenge_rating': 0.25,
  },
  {
    'name': 'Rat',
    'document_uuid': 'doc-51',
    'type': 'ct-beast',
    'size': 'sz-tiny',
    'challenge_rating': 0.0,
  },
];

void main() {
  group('dimension declarations', () {
    test('every table filters by source; some add more dimensions', () {
      for (final table in ['species', 'class', 'background', 'feat']) {
        expect(filterDimensionsFor(table).map((d) => d.key), ['source'],
            reason: table);
      }
      // Conditions and languages carry no document reference — their
      // source is the installing content module.
      for (final table in ['condition', 'language']) {
        final dims = filterDimensionsFor(table);
        expect(dims.map((d) => d.key), ['source'], reason: table);
        expect(dims.first.valuesOf({'content_module_uuid': 'mod-srd'}),
            ['mod-srd']);
        expect(dims.first.optionLabel('mod-srd', _lookups),
            'System Reference Document');
      }
      expect(filterDimensionsFor('item').map((d) => d.key),
          ['source', 'type', 'rarity']);
      expect(filterDimensionsFor('spell').map((d) => d.key),
          ['source', 'level', 'school', 'components']);
      expect(filterDimensionsFor('creature').map((d) => d.key),
          ['source', 'type']);
      expect(filterDimensionsFor('weapon').map((d) => d.key),
          ['source', 'category']);
      expect(filterDimensionsFor('armor').map((d) => d.key),
          ['source', 'category']);
    });
  });

  group('option derivation', () {
    test('distinct values present in the records, labeled via lookups', () {
      final source = filterDimensionsFor('item')[0];
      expect(
        source.options(_items, _lookups).map((o) => o.label),
        ['SRD 5.1', 'SRD 5.2'],
      );

      final rarity = filterDimensionsFor('item')[2];
      // Rarity ladder order, mundane (null) first.
      expect(
        rarity.options(_items, _lookups).map((o) => o.label),
        ['Mundane', 'Uncommon', 'Legendary'],
      );

      final level = filterDimensionsFor('spell')[1];
      expect(
        level.options(_spells, _lookups).map((o) => o.label),
        ['Cantrip', 'Level 3'],
      );

      // Set-valued dimension: options flatten every record's set, in
      // V/S/M order.
      final components = filterDimensionsFor('spell')[3];
      expect(
        components.options(_spells, _lookups).map((o) => o.label),
        ['V (verbal)', 'S (somatic)', 'M (material)'],
      );

      final creatureType = filterDimensionsFor('creature')[1];
      expect(
        creatureType.options(_creatures, _lookups).map((o) => o.label),
        ['Beast', 'Dragon'],
      );

      final weaponCategory = filterDimensionsFor('weapon')[1];
      expect(
        weaponCategory
            .options([
              {'is_simple': true},
              {'is_simple': false},
            ], _lookups)
            .map((o) => o.label),
        ['Martial', 'Simple'],
      );
    });
  });

  group('matching', () {
    final dims = filterDimensionsFor('item');

    test('empty selections match everything', () {
      expect(
        _items.where((r) => matchesFilters(r, dims, {})).length,
        _items.length,
      );
    });

    test('selections AND across dimensions, OR within one', () {
      final selections = {
        'source': <Object?>{'doc-52'},
        'rarity': <Object?>{null, 'uncommon'},
      };
      expect(
        _items
            .where((r) => matchesFilters(r, dims, selections))
            .map((r) => r['name']),
        ['Rope', 'Bag of Holding'],
      );
    });

    test('spells filter by component (any selected component matches)', () {
      final dims = filterDimensionsFor('spell');
      expect(
        _spells
            .where((r) => matchesFilters(r, dims, {
                  'components': {'material'},
                }))
            .map((r) => r['name']),
        ['Fireball', 'Animate Dead'],
      );
      expect(
        _spells
            .where((r) => matchesFilters(r, dims, {
                  'components': {'somatic'},
                  'source': {'doc-51'},
                }))
            .map((r) => r['name']),
        isEmpty,
      );
    });

    test('creatures filter by type', () {
      final selections = {
        'type': <Object?>{'ct-beast'},
      };
      expect(
        _creatures
            .where((r) =>
                matchesFilters(r, filterDimensionsFor('creature'), selections))
            .map((r) => r['name']),
        ['Wolf', 'Rat'],
      );
    });
  });

  group('sorts', () {
    test('spells sort by level then name', () {
      final byLevel =
          sortOptionsFor('spell').firstWhere((s) => s.key == 'level');
      final sorted = [..._spells]
        ..sort((a, b) => byLevel.compare(a, b, _lookups));
      expect(sorted.map((s) => s['name']),
          ['Fire Bolt', 'Animate Dead', 'Fireball']);
    });

    test('items sort by cost with unpriced last', () {
      final byCost =
          sortOptionsFor('item').firstWhere((s) => s.key == 'cost');
      final sorted = [..._items]
        ..sort((a, b) => byCost.compare(a, b, _lookups));
      expect(sorted.map((s) => s['name']),
          ['Rope', 'Bag of Holding', 'Vorpal Sword']);
    });

    test('creatures sort by challenge rating and by size rank', () {
      final byCr = sortOptionsFor('creature').firstWhere((s) => s.key == 'cr');
      final crSorted = [..._creatures]
        ..sort((a, b) => byCr.compare(a, b, _lookups));
      expect(crSorted.map((c) => c['name']),
          ['Rat', 'Wolf', 'Adult Red Dragon']);

      final bySize =
          sortOptionsFor('creature').firstWhere((s) => s.key == 'size');
      final sizeSorted = [..._creatures]
        ..sort((a, b) => bySize.compare(a, b, _lookups));
      expect(sizeSorted.map((c) => c['name']),
          ['Rat', 'Wolf', 'Adult Red Dragon']);
    });
  });
}
