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

import '../types/bundled_module.dart';
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
  static const _bundleMetaAsset = 'assets/content/srd-bundle.meta.json';

  /// The SRD module hosts the shared vocabulary (licenses, publishers,
  /// schools, sizes, …) every other module references — it can never
  /// be uninstalled.
  static const pinnedModuleSlug = 'srd';

  Future<Set<String>> installedModuleSlugs() async {
    final rows = await _db.query('content_module', columns: ['slug']);
    return {for (final r in rows) r['slug'] as String};
  }

  /// slug → removed_at for modules the user explicitly uninstalled.
  Future<Map<String, String>> removedModules() async {
    final rows = await _db.query('removed_content_module');
    return {
      for (final r in rows) r['slug'] as String: r['removed_at'] as String,
    };
  }

  /// Modules the shipped bundle contains, from the small manifest —
  /// launches don't decode the ~20 MB bundle just to learn nothing is
  /// missing, and the management UI can describe uninstalled modules.
  Future<List<BundledModule>> bundledModules() async {
    final raw = await rootBundle.loadString(_bundleMetaAsset);
    final meta = jsonDecode(raw) as Map<String, dynamic>;
    return [
      for (final m in meta['modules'] as List<dynamic>? ?? const [])
        BundledModule.fromJson(m as Map<String, dynamic>),
    ];
  }

  /// Bundled modules that should be present but aren't: not installed
  /// and not deliberately removed by the user.
  Future<Set<String>> _missingModuleSlugs() async {
    final bundled = {for (final m in await bundledModules()) m.slug};
    final existing = await installedModuleSlugs();
    final removed = (await removedModules()).keys.toSet();
    return bundled.difference(existing).difference(removed);
  }

  Future<bool> get isSeeded async => (await _missingModuleSlugs()).isEmpty;

  /// Imports every bundled record whose content module isn't installed
  /// yet — a fresh install seeds everything; an app upgraded to a
  /// bundle with additional source modules seeds only what's new.
  /// Modules the user uninstalled stay uninstalled (tombstoned in
  /// `removed_content_module`). Decoding the asset runs in an isolate
  /// so the UI thread stays responsive; inserts run in chunked batches
  /// inside one transaction, reporting progress in [0, 1] after each
  /// chunk.
  Future<void> importBundle({void Function(double progress)? onProgress}) async {
    final missing = await _missingModuleSlugs();
    if (missing.isEmpty) return;

    final raw = await rootBundle.loadString(_bundleAsset);
    final bundle = await compute(_decodeJson, raw);

    final allowedModuleUuids = {
      for (final m in bundle['modules'] as List<dynamic>? ?? const [])
        if (missing.contains((m as Map<String, dynamic>)['slug']))
          m['uuid'] as String,
    };
    // Module rows are matched by slug; every other record rides along
    // only when its owning module is being seeded.
    bool allowed(_TableSpec spec, Map<String, dynamic> r) =>
        spec.table == 'content_module'
            ? missing.contains(r['slug'])
            : allowedModuleUuids.contains(r['content_module_uuid']);

    final tables = <(_TableSpec, List<Map<String, dynamic>>)>[
      for (final spec in _specs)
        (
          spec,
          [
            for (final r
                in bundle[spec.bundleField] as List<dynamic>? ?? const [])
              if (allowed(spec, r as Map<String, dynamic>)) r,
          ],
        ),
    ];
    final total = tables.fold<int>(0, (sum, t) => sum + t.$2.length);
    if (total == 0) return;
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
              if (spec.table != 'content_module')
                'content_module_uuid': r['content_module_uuid'],
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

  /// Deletes one module and every record it owns, and tombstones it so
  /// the seeder won't bring it back. The SRD module is pinned.
  Future<void> uninstallModule(String slug) async {
    if (slug == pinnedModuleSlug) {
      throw ArgumentError('the $pinnedModuleSlug module cannot be removed');
    }
    await _db.transaction((txn) async {
      final rows = await txn.query('content_module',
          columns: ['uuid'], where: 'slug = ?', whereArgs: [slug]);
      if (rows.isNotEmpty) {
        final moduleUuid = rows.first['uuid'] as String;
        for (final table in LocalStore.contentTables) {
          if (table == 'content_module') continue;
          await txn.delete('"$table"',
              where: 'content_module_uuid = ?', whereArgs: [moduleUuid]);
        }
        await txn.delete('content_module',
            where: 'slug = ?', whereArgs: [slug]);
      }
      await txn.insert(
        'removed_content_module',
        {
          'slug': slug,
          'removed_at': DateTime.now().toUtc().toIso8601String(),
        },
        conflictAlgorithm: ConflictAlgorithm.replace,
      );
    });
  }

  /// Clears a module's tombstone and re-seeds it from the bundle.
  Future<void> reinstallModule(
    String slug, {
    void Function(double progress)? onProgress,
  }) async {
    await _db.delete('removed_content_module',
        where: 'slug = ?', whereArgs: [slug]);
    await importBundle(onProgress: onProgress);
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

  Future<List<Map<String, dynamic>>> listBackgrounds() => _list('background');

  Future<List<Map<String, dynamic>>> listAlignments() => _list('alignment');

  Future<Map<String, dynamic>?> getByKey(String table, String key) async {
    final rows = await _db.query('"$table"',
        columns: ['data'], where: 'key = ?', whereArgs: [key]);
    if (rows.isEmpty) return null;
    return jsonDecode(rows.first['data'] as String) as Map<String, dynamic>;
  }

  /// Name-search within one content table. Combines an optional LIKE
  /// match on [query] with an optional extra [where] clause.
  Future<List<Map<String, dynamic>>> listNamed(
    String table, {
    String? query,
    String? where,
    List<Object?>? whereArgs,
    int? limit,
  }) {
    final hasQuery = query != null && query.trim().isNotEmpty;
    final clauses = [
      if (where != null) '($where)',
      if (hasQuery) 'name LIKE ?',
    ];
    return _list(
      table,
      where: clauses.isEmpty ? null : clauses.join(' AND '),
      whereArgs: [
        ...?whereArgs,
        if (hasQuery) '%${query.trim()}%',
      ],
      limit: limit,
    );
  }

  Future<int> count(String table) async {
    final rows = await _db.rawQuery('SELECT COUNT(*) AS n FROM "$table"');
    return rows.first['n'] as int? ?? 0;
  }

  /// uuid → display name for a lookup table (spell schools, sizes,
  /// creature types, item categories). Small tables; load whole.
  Future<Map<String, String>> lookupNames(String table) =>
      lookupColumn(table, 'name');

  /// uuid → an arbitrary indexed column ('key', 'slug', …).
  Future<Map<String, String>> lookupColumn(String table, String column) async {
    final rows = await _db.query('"$table"', columns: ['uuid', column]);
    return {
      for (final r in rows)
        if (r[column] != null) r['uuid'] as String: r[column] as String,
    };
  }

  Future<List<Map<String, dynamic>>> _list(
    String table, {
    String? where,
    List<Object?>? whereArgs,
    int? limit,
  }) async {
    final rows = await _db.query('"$table"',
        columns: ['data'],
        where: where,
        whereArgs: whereArgs,
        orderBy: 'name COLLATE NOCASE',
        limit: limit);
    return rows
        .map((r) => jsonDecode(r['data'] as String) as Map<String, dynamic>)
        .toList();
  }
}

Map<String, dynamic> _decodeJson(String raw) =>
    jsonDecode(raw) as Map<String, dynamic>;
