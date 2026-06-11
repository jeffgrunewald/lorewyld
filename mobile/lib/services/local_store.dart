// On-device SQLite store — the app's primary content home. Characters,
// settings, and lore notes are authored here with no server required;
// `remote_uuid` columns map local records to their server counterparts
// once a setting has been pushed or pulled (see sync_service.dart).

import 'dart:convert';

import 'package:sqflite/sqflite.dart';

import '../types/character.dart';
import '../types/lore_note.dart';
import '../util/uuid.dart';

class LocalSetting {
  final String uuid;
  final String name;
  final String? remoteUuid;
  final DateTime createdAt;
  final DateTime updatedAt;

  const LocalSetting({
    required this.uuid,
    required this.name,
    this.remoteUuid,
    required this.createdAt,
    required this.updatedAt,
  });

  bool get isLinked => remoteUuid != null;
}

class LocalNote {
  final String uuid;
  final String title;
  final String bodyMarkdown;
  final NoteScope scope;
  final NoteVisibility visibility;
  final List<String> tagSlugs;
  final String? remoteUuid;
  final DateTime createdAt;
  final DateTime updatedAt;

  const LocalNote({
    required this.uuid,
    required this.title,
    required this.bodyMarkdown,
    required this.scope,
    required this.visibility,
    required this.tagSlugs,
    this.remoteUuid,
    required this.createdAt,
    required this.updatedAt,
  });
}

class LocalStore {
  final Database _db;

  LocalStore._(this._db);

  static const _schemaVersion = 2;

  /// SRD/content reference tables, mirroring the server's doc-style
  /// layout: identity + a few indexed filter columns, full record JSON
  /// in `data`. Populated by ContentStore.importBundle on first launch.
  static const contentTables = [
    'content_module',
    'license',
    'publisher',
    'document',
    'ability_score',
    'skill',
    'alignment',
    'damage_type',
    'condition',
    'language',
    'size',
    'environment',
    'spell_school',
    'creature_type',
    'item_category',
    'weapon_property',
    'spell',
    'creature',
    'class',
    'species',
    'feat',
    'background',
    'weapon',
    'armor',
    'item',
  ];

  static const _contentExtraColumns = {
    'spell':
        'level INTEGER NOT NULL DEFAULT 0, school_uuid TEXT, concentration INTEGER NOT NULL DEFAULT 0, ritual INTEGER NOT NULL DEFAULT 0,',
    'creature':
        'challenge_rating REAL NOT NULL DEFAULT 0, creature_type_uuid TEXT, size_uuid TEXT,',
    'class': 'subclass_of TEXT,',
    'species': 'is_subspecies INTEGER NOT NULL DEFAULT 0,',
    'weapon': 'is_simple INTEGER NOT NULL DEFAULT 0,',
    'armor': 'category TEXT,',
    'item':
        'category_uuid TEXT, rarity TEXT, is_magic INTEGER NOT NULL DEFAULT 0,',
  };

  static Future<void> _createContentTables(Database db) async {
    for (final table in contentTables) {
      final extras = _contentExtraColumns[table] ?? '';
      await db.execute('''
        CREATE TABLE IF NOT EXISTS "$table" (
          uuid TEXT PRIMARY KEY NOT NULL,
          key  TEXT NOT NULL UNIQUE,
          slug TEXT NOT NULL,
          name TEXT NOT NULL,
          $extras
          data TEXT NOT NULL
        )
      ''');
    }
    await db.execute(
        'CREATE INDEX IF NOT EXISTS idx_spell_level ON spell(level)');
    await db.execute(
        'CREATE INDEX IF NOT EXISTS idx_creature_cr ON creature(challenge_rating)');
    await db.execute(
        'CREATE INDEX IF NOT EXISTS idx_item_magic ON item(is_magic)');
  }

  static Future<LocalStore> open({String? path}) async {
    final dbPath = path ?? '${await getDatabasesPath()}/lorewyld_local.db';
    final db = await openDatabase(
      dbPath,
      version: _schemaVersion,
      onCreate: (db, version) async {
        await db.execute('''
          CREATE TABLE character (
            uuid       TEXT PRIMARY KEY NOT NULL,
            name       TEXT NOT NULL,
            data       TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE setting (
            uuid        TEXT PRIMARY KEY NOT NULL,
            name        TEXT NOT NULL,
            remote_uuid TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE lore_note (
            uuid              TEXT PRIMARY KEY NOT NULL,
            title             TEXT NOT NULL,
            body_markdown     TEXT NOT NULL DEFAULT '',
            scope_kind        TEXT NOT NULL,
            scope_target_uuid TEXT NOT NULL,
            visibility        TEXT NOT NULL DEFAULT 'visible',
            tag_slugs         TEXT NOT NULL DEFAULT '[]',
            remote_uuid       TEXT,
            created_at        TEXT NOT NULL,
            updated_at        TEXT NOT NULL
          )
        ''');
        await db.execute(
            'CREATE INDEX idx_lore_note_scope ON lore_note(scope_kind, scope_target_uuid)');
        await _createContentTables(db);
      },
      onUpgrade: (db, oldVersion, newVersion) async {
        if (oldVersion < 2) {
          await _createContentTables(db);
        }
      },
    );
    return LocalStore._(db);
  }

  Database get database => _db;

  Future<void> close() => _db.close();

  static String _now() => DateTime.now().toUtc().toIso8601String();

  // ── characters ──────────────────────────────────────────────────────

  Future<List<CharacterSheet>> listCharacters() async {
    final rows = await _db.query('character', orderBy: 'name COLLATE NOCASE');
    return rows
        .map((r) => CharacterSheet.fromJson(
            jsonDecode(r['data'] as String) as Map<String, dynamic>))
        .toList();
  }

  Future<CharacterSheet> createCharacter(String name) async {
    final now = DateTime.now().toUtc();
    final sheet = CharacterSheet(
      uuid: generateUuidV4(),
      name: name,
      abilities: CharacterSheet.defaultAbilities(),
      createdAt: now,
      updatedAt: now,
    );
    await _db.insert('character', {
      'uuid': sheet.uuid,
      'name': sheet.name,
      'data': jsonEncode(sheet.toJson()),
      'created_at': sheet.createdAt.toIso8601String(),
      'updated_at': sheet.updatedAt.toIso8601String(),
    });
    return sheet;
  }

  Future<CharacterSheet> saveCharacter(CharacterSheet sheet) async {
    final updated = sheet.copyWith(updatedAt: DateTime.now().toUtc());
    await _db.update(
      'character',
      {
        'name': updated.name,
        'data': jsonEncode(updated.toJson()),
        'updated_at': updated.updatedAt.toIso8601String(),
      },
      where: 'uuid = ?',
      whereArgs: [updated.uuid],
    );
    return updated;
  }

  Future<void> deleteCharacter(String uuid) async {
    await _db.delete('character', where: 'uuid = ?', whereArgs: [uuid]);
    await _db.delete(
      'lore_note',
      where: 'scope_kind = ? AND scope_target_uuid = ?',
      whereArgs: [NoteScopeKind.character.wire, uuid],
    );
  }

  // ── settings ────────────────────────────────────────────────────────

  Future<List<LocalSetting>> listSettings() async {
    final rows =
        await _db.query('setting', orderBy: 'updated_at DESC');
    return rows.map(_settingFromRow).toList();
  }

  Future<LocalSetting?> getSetting(String uuid) async {
    final rows =
        await _db.query('setting', where: 'uuid = ?', whereArgs: [uuid]);
    return rows.isEmpty ? null : _settingFromRow(rows.first);
  }

  Future<LocalSetting?> getSettingByRemoteUuid(String remoteUuid) async {
    final rows = await _db.query('setting',
        where: 'remote_uuid = ?', whereArgs: [remoteUuid]);
    return rows.isEmpty ? null : _settingFromRow(rows.first);
  }

  Future<LocalSetting> createSetting(String name,
      {String? remoteUuid}) async {
    final now = _now();
    final uuid = generateUuidV4();
    await _db.insert('setting', {
      'uuid': uuid,
      'name': name,
      'remote_uuid': remoteUuid,
      'created_at': now,
      'updated_at': now,
    });
    return (await getSetting(uuid))!;
  }

  Future<void> renameSetting(String uuid, String name) async {
    await _db.update(
      'setting',
      {'name': name, 'updated_at': _now()},
      where: 'uuid = ?',
      whereArgs: [uuid],
    );
  }

  Future<void> linkSettingRemote(String uuid, String remoteUuid) async {
    await _db.update(
      'setting',
      {'remote_uuid': remoteUuid, 'updated_at': _now()},
      where: 'uuid = ?',
      whereArgs: [uuid],
    );
  }

  Future<void> deleteSetting(String uuid) async {
    await _db.delete('setting', where: 'uuid = ?', whereArgs: [uuid]);
    await _db.delete(
      'lore_note',
      where: 'scope_kind = ? AND scope_target_uuid = ?',
      whereArgs: [NoteScopeKind.setting.wire, uuid],
    );
  }

  LocalSetting _settingFromRow(Map<String, Object?> r) => LocalSetting(
        uuid: r['uuid'] as String,
        name: r['name'] as String,
        remoteUuid: r['remote_uuid'] as String?,
        createdAt: DateTime.parse(r['created_at'] as String),
        updatedAt: DateTime.parse(r['updated_at'] as String),
      );

  // ── lore notes ──────────────────────────────────────────────────────

  Future<List<LocalNote>> listNotes({
    NoteScopeKind? scopeKind,
    String? scopeTarget,
  }) async {
    final where = <String>[];
    final args = <Object?>[];
    if (scopeKind != null) {
      where.add('scope_kind = ?');
      args.add(scopeKind.wire);
    }
    if (scopeTarget != null) {
      where.add('scope_target_uuid = ?');
      args.add(scopeTarget);
    }
    final rows = await _db.query(
      'lore_note',
      where: where.isEmpty ? null : where.join(' AND '),
      whereArgs: args.isEmpty ? null : args,
      orderBy: 'updated_at DESC',
    );
    return rows.map(_noteFromRow).toList();
  }

  Future<LocalNote?> getNote(String uuid) async {
    final rows =
        await _db.query('lore_note', where: 'uuid = ?', whereArgs: [uuid]);
    return rows.isEmpty ? null : _noteFromRow(rows.first);
  }

  Future<LocalNote> createNote({
    required String title,
    required String bodyMarkdown,
    required NoteScope scope,
    NoteVisibility visibility = NoteVisibility.visible,
    List<String> tagSlugs = const [],
    String? remoteUuid,
  }) async {
    final now = _now();
    final uuid = generateUuidV4();
    await _db.insert('lore_note', {
      'uuid': uuid,
      'title': title,
      'body_markdown': bodyMarkdown,
      'scope_kind': scope.kind.wire,
      'scope_target_uuid': scope.targetUuid,
      'visibility': visibility.wire,
      'tag_slugs': jsonEncode(tagSlugs),
      'remote_uuid': remoteUuid,
      'created_at': now,
      'updated_at': now,
    });
    return (await getNote(uuid))!;
  }

  Future<LocalNote> updateNote({
    required String uuid,
    String? title,
    String? bodyMarkdown,
    NoteVisibility? visibility,
    List<String>? tagSlugs,
    String? remoteUuid,
  }) async {
    final values = <String, Object?>{'updated_at': _now()};
    if (title != null) values['title'] = title;
    if (bodyMarkdown != null) values['body_markdown'] = bodyMarkdown;
    if (visibility != null) values['visibility'] = visibility.wire;
    if (tagSlugs != null) values['tag_slugs'] = jsonEncode(tagSlugs);
    if (remoteUuid != null) values['remote_uuid'] = remoteUuid;
    await _db.update('lore_note', values,
        where: 'uuid = ?', whereArgs: [uuid]);
    return (await getNote(uuid))!;
  }

  Future<void> deleteNote(String uuid) async {
    await _db.delete('lore_note', where: 'uuid = ?', whereArgs: [uuid]);
  }

  /// Local free-text search over title + body, with optional scope and
  /// tag filters (AND semantics across tags).
  Future<List<LocalNote>> searchNotes({
    String? q,
    NoteScopeKind? scopeKind,
    List<String> tagSlugs = const [],
    int limit = 50,
  }) async {
    final where = <String>[];
    final args = <Object?>[];
    if (q != null && q.trim().isNotEmpty) {
      where.add('(title LIKE ? OR body_markdown LIKE ?)');
      final pattern = '%${q.trim()}%';
      args
        ..add(pattern)
        ..add(pattern);
    }
    if (scopeKind != null) {
      where.add('scope_kind = ?');
      args.add(scopeKind.wire);
    }
    for (final slug in tagSlugs) {
      // tag_slugs is a JSON array of quoted strings — match the quoted
      // form so "fey" doesn't match "fey-realm".
      where.add('tag_slugs LIKE ?');
      args.add('%"$slug"%');
    }
    final rows = await _db.query(
      'lore_note',
      where: where.isEmpty ? null : where.join(' AND '),
      whereArgs: args.isEmpty ? null : args,
      orderBy: 'updated_at DESC',
      limit: limit,
    );
    return rows.map(_noteFromRow).toList();
  }

  /// Distinct tag slugs across all local notes, optionally filtered by
  /// prefix — feeds the tag autocomplete offline.
  Future<List<String>> suggestTagSlugs({String? prefix, int limit = 8}) async {
    final rows = await _db.query('lore_note', columns: ['tag_slugs']);
    final all = <String>{};
    for (final row in rows) {
      final decoded = jsonDecode(row['tag_slugs'] as String) as List<dynamic>;
      all.addAll(decoded.cast<String>());
    }
    final filtered = all
        .where((s) =>
            prefix == null || prefix.isEmpty || s.contains(prefix.toLowerCase()))
        .toList()
      ..sort();
    return filtered.take(limit).toList();
  }

  LocalNote _noteFromRow(Map<String, Object?> r) => LocalNote(
        uuid: r['uuid'] as String,
        title: r['title'] as String,
        bodyMarkdown: r['body_markdown'] as String,
        scope: NoteScope(
          kind: NoteScopeKind.fromWire(r['scope_kind'] as String),
          targetUuid: r['scope_target_uuid'] as String,
        ),
        visibility: NoteVisibility.fromWire(r['visibility'] as String),
        tagSlugs: (jsonDecode(r['tag_slugs'] as String) as List<dynamic>)
            .cast<String>(),
        remoteUuid: r['remote_uuid'] as String?,
        createdAt: DateTime.parse(r['created_at'] as String),
        updatedAt: DateTime.parse(r['updated_at'] as String),
      );
}
