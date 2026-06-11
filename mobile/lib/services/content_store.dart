// Imports the bundled SRD content (assets/content/srd-bundle.json) into
// the local sqflite database on first launch, and provides read access
// to the seeded reference content.
//
// Records are stored doc-style — identity + a few indexed filter
// columns, full record JSON in `data` — mirroring the server schema, so
// both sides read the same wire shapes.

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show rootBundle;
import 'package:sqflite/sqflite.dart';

import 'local_store.dart';

/// Bundle JSON field -> sqflite table, in import-dependency order, with
/// any extra indexed columns pulled from each record.
class _TableSpec {
  final String bundleField;
  final String table;
  final Map<String, String> extras;

  const _TableSpec(this.bundleField, this.table, [this.extras = const {}]);
}

const _specs = [
  _TableSpec('modules', 'content_module'),
  _TableSpec('licenses', 'license'),
  _TableSpec('publishers', 'publisher'),
  _TableSpec('documents', 'document'),
  _TableSpec('ability_scores', 'ability_score'),
  _TableSpec('skills', 'skill'),
  _TableSpec('alignments', 'alignment'),
  _TableSpec('damage_types', 'damage_type'),
  _TableSpec('conditions', 'condition'),
  _TableSpec('languages', 'language'),
  _TableSpec('sizes', 'size'),
  _TableSpec('environments', 'environment'),
  _TableSpec('spell_schools', 'spell_school'),
  _TableSpec('creature_types', 'creature_type'),
  _TableSpec('item_categories', 'item_category'),
  _TableSpec('weapon_properties', 'weapon_property'),
  _TableSpec('spells', 'spell', {
    'level': 'level',
    'school_uuid': 'school',
    'concentration': 'concentration',
    'ritual': 'ritual',
  }),
  _TableSpec('creatures', 'creature', {
    'challenge_rating': 'challenge_rating',
    'creature_type_uuid': 'type',
    'size_uuid': 'size',
  }),
  _TableSpec('classes', 'class', {'subclass_of': 'subclass_of'}),
  _TableSpec('species', 'species', {'is_subspecies': 'is_subspecies'}),
  _TableSpec('feats', 'feat'),
  _TableSpec('backgrounds', 'background'),
  _TableSpec('weapons', 'weapon', {'is_simple': 'is_simple'}),
  _TableSpec('armors', 'armor', {'category': 'category'}),
  _TableSpec('items', 'item', {
    'category_uuid': 'category_uuid',
    'rarity': 'rarity',
    'is_magic': 'is_magic',
  }),
];

class ContentStore {
  final LocalStore _store;

  ContentStore(this._store);

  Database get _db => _store.database;

  static const _bundleAsset = 'assets/content/srd-bundle.json';
  static const _srdModuleSlug = 'srd';

  Future<bool> get isSeeded async {
    final rows = await _db.query('content_module',
        columns: ['uuid'], where: 'slug = ?', whereArgs: [_srdModuleSlug]);
    return rows.isNotEmpty;
  }

  /// Loads the bundled SRD JSON and imports every record. Decoding the
  /// ~4 MB asset runs in an isolate so the UI thread stays responsive;
  /// inserts run in chunked batches inside one transaction, reporting
  /// progress in [0, 1] after each chunk.
  Future<void> importBundle({void Function(double progress)? onProgress}) async {
    if (await isSeeded) return;
    final raw = await rootBundle.loadString(_bundleAsset);
    final bundle = await compute(_decodeJson, raw);

    final tables = <(_TableSpec, List<dynamic>)>[
      for (final spec in _specs)
        (spec, (bundle[spec.bundleField] as List<dynamic>? ?? const [])),
    ];
    final total = tables.fold<int>(0, (sum, t) => sum + t.$2.length);
    var inserted = 0;

    await _db.transaction((txn) async {
      const chunkSize = 400;
      for (final (spec, records) in tables) {
        // Parents must land before children for self-referential
        // tables; bundle order is key-sorted, not dependency-sorted.
        final ordered = switch (spec.table) {
          'class' => _parentsFirst(records, 'subclass_of'),
          'species' => _parentsFirst(records, 'subspecies_of'),
          _ => records,
        };
        for (var i = 0; i < ordered.length; i += chunkSize) {
          final batch = txn.batch();
          for (final record in ordered.skip(i).take(chunkSize)) {
            final r = record as Map<String, dynamic>;
            batch.insert('"${spec.table}"', {
              'uuid': r['uuid'],
              'key': r['key'] ?? r['slug'],
              'slug': r['slug'],
              'name': r['name'],
              for (final entry in spec.extras.entries)
                entry.key: _bindable(r[entry.value]),
              'data': jsonEncode(r),
            });
          }
          await batch.commit(noResult: true);
          inserted += chunkSize;
          onProgress?.call((inserted / total).clamp(0.0, 1.0));
        }
      }
    });
    onProgress?.call(1.0);
  }

  static List<dynamic> _parentsFirst(List<dynamic> records, String parentField) {
    final parents = <dynamic>[];
    final children = <dynamic>[];
    for (final r in records) {
      ((r as Map<String, dynamic>)[parentField] == null ? parents : children)
          .add(r);
    }
    return [...parents, ...children];
  }

  static Object? _bindable(Object? value) => switch (value) {
        bool b => b ? 1 : 0,
        _ => value,
      };

  // ── reads ───────────────────────────────────────────────────────────

  Future<List<Map<String, dynamic>>> listSpells({int? level}) =>
      _list('spell',
          where: level != null ? 'level = ?' : null,
          whereArgs: level != null ? [level] : null);

  Future<List<Map<String, dynamic>>> listCreatures(
          {double? maxChallengeRating}) =>
      _list('creature',
          where: maxChallengeRating != null ? 'challenge_rating <= ?' : null,
          whereArgs:
              maxChallengeRating != null ? [maxChallengeRating] : null);

  Future<List<Map<String, dynamic>>> listClasses({bool basesOnly = false}) =>
      _list('class', where: basesOnly ? 'subclass_of IS NULL' : null);

  Future<List<Map<String, dynamic>>> listSpecies() => _list('species');

  Future<Map<String, dynamic>?> getByKey(String table, String key) async {
    final rows = await _db.query('"$table"',
        columns: ['data'], where: 'key = ?', whereArgs: [key]);
    if (rows.isEmpty) return null;
    return jsonDecode(rows.first['data'] as String) as Map<String, dynamic>;
  }

  Future<List<Map<String, dynamic>>> _list(
    String table, {
    String? where,
    List<Object?>? whereArgs,
  }) async {
    final rows = await _db.query('"$table"',
        columns: ['data'],
        where: where,
        whereArgs: whereArgs,
        orderBy: 'name COLLATE NOCASE');
    return rows
        .map((r) => jsonDecode(r['data'] as String) as Map<String, dynamic>)
        .toList();
  }
}

Map<String, dynamic> _decodeJson(String raw) =>
    jsonDecode(raw) as Map<String, dynamic>;
