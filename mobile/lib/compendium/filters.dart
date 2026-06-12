// Filter dimensions and sort orders for the content pickers and the
// Compendium category screens. Each table declares which record
// attributes it can be narrowed by; the options users see are derived
// from the records actually loaded, so new content modules (and their
// source documents, rarities, etc.) surface automatically without code
// changes.

import 'categories.dart';

class FilterOption {
  /// The raw record value matched against (uuid, int level, rarity
  /// string, or null for "absent" values like a mundane item's rarity).
  final Object? value;
  final String label;

  const FilterOption(this.value, this.label);
}

class FilterDimension {
  final String key;
  final String label;

  /// Single-valued attribute accessor (most dimensions).
  final Object? Function(Map<String, dynamic> record)? valueOf;

  /// Set-valued attribute accessor (e.g. spell components) — a record
  /// matches when any of its values is selected.
  final Iterable<Object?> Function(Map<String, dynamic> record)? multiValueOf;

  final String Function(Object? value, ContentLookups lookups) optionLabel;

  /// Orders the derived options; defaults to alphabetical by label.
  final int Function(Object? a, Object? b)? valueSort;

  const FilterDimension({
    required this.key,
    required this.label,
    this.valueOf,
    this.multiValueOf,
    required this.optionLabel,
    this.valueSort,
  }) : assert(valueOf != null || multiValueOf != null,
            'a dimension needs an accessor');

  Iterable<Object?> valuesOf(Map<String, dynamic> record) =>
      multiValueOf?.call(record) ?? [valueOf!(record)];

  /// Distinct values present in [records], as labeled options.
  List<FilterOption> options(
    Iterable<Map<String, dynamic>> records,
    ContentLookups lookups,
  ) {
    final values = {for (final r in records) ...valuesOf(r)};
    final opts = [
      for (final v in values) FilterOption(v, optionLabel(v, lookups)),
    ];
    final sortValues = valueSort;
    sortValues != null
        ? opts.sort((a, b) => sortValues(a.value, b.value))
        : opts.sort((a, b) => a.label.compareTo(b.label));
    return opts;
  }
}

class PickerSort {
  final String key;
  final String label;

  /// Lookups are passed in for orders that rank through a reference
  /// table (creature size).
  final int Function(
    Map<String, dynamic> a,
    Map<String, dynamic> b,
    ContentLookups lookups,
  ) compare;

  const PickerSort({
    required this.key,
    required this.label,
    required this.compare,
  });
}

/// Active filter selections + sort order for one filterable list.
/// Mutated in place by the filter sheet; owners rebuild on change.
class FilterState {
  final Map<String, Set<Object?>> selections = {};
  PickerSort sort;

  FilterState(this.sort);

  int get activeCount => selections.values.where((s) => s.isNotEmpty).length;

  void reset(PickerSort defaultSort) {
    selections.clear();
    sort = defaultSort;
  }
}

int _byName(Map<String, dynamic> a, Map<String, dynamic> b, ContentLookups _) =>
    '${a['name']}'.toLowerCase().compareTo('${b['name']}'.toLowerCase());

final _source = FilterDimension(
  key: 'source',
  label: 'Source',
  valueOf: (r) => r['document_uuid'],
  optionLabel: (v, l) =>
      v is String ? l.documents[v] ?? 'Unknown source' : 'Unknown source',
);

/// Lookup-style tables (conditions, languages) carry no document
/// reference — their source is the installing content module.
final _moduleSource = FilterDimension(
  key: 'source',
  label: 'Source',
  valueOf: (r) => r['content_module_uuid'],
  optionLabel: (v, l) => v is String
      ? l.contentModules[v] ?? 'Unknown source'
      : 'Unknown source',
);

const List<Object?> _rarityOrder = [
  null,
  'common',
  'uncommon',
  'rare',
  'very_rare',
  'legendary',
  'artifact',
];

final _rarity = FilterDimension(
  key: 'rarity',
  label: 'Rarity',
  valueOf: (r) => r['rarity'],
  optionLabel: (v, l) => v is String ? humanizeSlug(v) : 'Mundane',
  valueSort: (a, b) {
    // Unrecognized rarities sort after the known ladder.
    int rank(Object? v) {
      final i = _rarityOrder.indexOf(v);
      return i < 0 ? _rarityOrder.length : i;
    }

    return rank(a).compareTo(rank(b));
  },
);

final _itemType = FilterDimension(
  key: 'type',
  label: 'Type',
  valueOf: (r) => r['category_uuid'],
  optionLabel: (v, l) =>
      v is String ? l.itemCategories[v] ?? 'Uncategorized' : 'Uncategorized',
);

final _spellLevel = FilterDimension(
  key: 'level',
  label: 'Level',
  valueOf: (r) => r['level'],
  optionLabel: (v, l) => v is int ? spellLevelLabel(v) : 'Unknown',
  valueSort: (a, b) => (a is int ? a : 99).compareTo(b is int ? b : 99),
);

final _spellSchool = FilterDimension(
  key: 'school',
  label: 'School',
  valueOf: (r) => r['school'],
  optionLabel: (v, l) =>
      v is String ? l.spellSchools[v] ?? 'Unknown school' : 'Unknown school',
);

const _componentOrder = ['verbal', 'somatic', 'material'];

final _spellComponents = FilterDimension(
  key: 'components',
  label: 'Components',
  multiValueOf: (r) => [
    if (r['verbal'] == true) 'verbal',
    if (r['somatic'] == true) 'somatic',
    if (r['material'] == true) 'material',
  ],
  optionLabel: (v, l) => switch (v) {
    'verbal' => 'V (verbal)',
    'somatic' => 'S (somatic)',
    'material' => 'M (material)',
    _ => '$v',
  },
  valueSort: (a, b) =>
      _componentOrder.indexOf('$a').compareTo(_componentOrder.indexOf('$b')),
);

final _creatureType = FilterDimension(
  key: 'type',
  label: 'Creature type',
  valueOf: (r) => r['type'],
  optionLabel: (v, l) =>
      v is String ? l.creatureTypes[v] ?? 'Unknown type' : 'Unknown type',
);

final _weaponCategory = FilterDimension(
  key: 'category',
  label: 'Category',
  valueOf: (r) => r['is_simple'],
  optionLabel: (v, l) => v == true ? 'Simple' : 'Martial',
);

final _armorCategory = FilterDimension(
  key: 'category',
  label: 'Category',
  valueOf: (r) => r['category'],
  optionLabel: (v, l) => v is String ? humanizeSlug(v) : 'Uncategorized',
);

List<FilterDimension> filterDimensionsFor(String table) => switch (table) {
      'spell' => [_source, _spellLevel, _spellSchool, _spellComponents],
      'item' => [_source, _itemType, _rarity],
      'creature' => [_source, _creatureType],
      'weapon' => [_source, _weaponCategory],
      'armor' => [_source, _armorCategory],
      'species' || 'class' || 'background' || 'feat' => [_source],
      'condition' || 'language' => [_moduleSource],
      _ => const [],
    };

List<PickerSort> sortOptionsFor(String table) => [
      const PickerSort(key: 'name', label: 'Name', compare: _byName),
      if (table == 'spell')
        PickerSort(
          key: 'level',
          label: 'Level',
          compare: (a, b, l) {
            final byLevel =
                (a['level'] as int? ?? 0).compareTo(b['level'] as int? ?? 0);
            return byLevel != 0 ? byLevel : _byName(a, b, l);
          },
        ),
      if (table == 'item')
        PickerSort(
          key: 'cost',
          label: 'Cost',
          compare: (a, b, l) {
            // Costs are decimal strings ("25.00"); priceless/unpriced
            // items sort last.
            double parse(Object? v) => v is String
                ? double.tryParse(v) ?? double.infinity
                : double.infinity;
            final byCost = parse(a['cost']).compareTo(parse(b['cost']));
            return byCost != 0 ? byCost : _byName(a, b, l);
          },
        ),
      if (table == 'creature') ...[
        PickerSort(
          key: 'cr',
          label: 'Challenge rating',
          compare: (a, b, l) {
            num cr(Map<String, dynamic> r) =>
                r['challenge_rating'] as num? ?? -1;
            final byCr = cr(a).compareTo(cr(b));
            return byCr != 0 ? byCr : _byName(a, b, l);
          },
        ),
        PickerSort(
          key: 'size',
          label: 'Size',
          compare: (a, b, l) {
            // Rank comes from the size table (Tiny 1 … Gargantuan 6);
            // unknown sizes sort last.
            int rank(Map<String, dynamic> r) => switch (r['size']) {
                  final String uuid => l.sizeRanks[uuid] ?? 99,
                  _ => 99,
                };
            final bySize = rank(a).compareTo(rank(b));
            return bySize != 0 ? bySize : _byName(a, b, l);
          },
        ),
      ],
    ];

/// True when [record] passes every dimension that has an active
/// selection; empty selections mean "no restriction".
bool matchesFilters(
  Map<String, dynamic> record,
  List<FilterDimension> dimensions,
  Map<String, Set<Object?>> selections,
) {
  for (final dim in dimensions) {
    final selected = selections[dim.key];
    if (selected != null &&
        selected.isNotEmpty &&
        !dim.valuesOf(record).any(selected.contains)) {
      return false;
    }
  }
  return true;
}
