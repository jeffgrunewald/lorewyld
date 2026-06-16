// LocalStore CRUD + search tests against an in-memory SQLite database.

import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:lorewyld/services/local_store.dart';
import 'package:lorewyld/types/lore_note.dart';

void main() {
  sqfliteFfiInit();
  databaseFactory = databaseFactoryFfi;

  late LocalStore store;

  setUp(() async {
    store = await LocalStore.open(path: inMemoryDatabasePath);
  });

  tearDown(() => store.close());

  test('characters: create, list, save, delete', () async {
    final created = await store.createCharacter('Thistle');
    expect(created.level, 1);

    final updated = await store.saveCharacter(
      created.copyWith(level: 3, race: 'Halfling'),
    );
    expect(updated.level, 3);

    final listed = await store.listCharacters();
    expect(listed.single.race, 'Halfling');

    await store.deleteCharacter(created.uuid);
    expect(await store.listCharacters(), isEmpty);
  });

  test('settings and notes: offline authoring round-trip', () async {
    final setting = await store.createSetting('Verdant Realms');
    expect(setting.isLinked, isFalse);

    final note = await store.createNote(
      title: 'The Fey Court',
      bodyMarkdown: 'Ancient rulers of the realm.',
      scope: NoteScope(kind: NoteScopeKind.setting, targetUuid: setting.uuid),
      tagSlugs: const ['npc', 'fey-realm'],
    );
    expect(note.remoteUuid, isNull);

    final notes = await store.listNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: setting.uuid,
    );
    expect(notes.single.tagSlugs, ['npc', 'fey-realm']);

    // Deleting the setting removes its notes too.
    await store.deleteSetting(setting.uuid);
    expect(
      await store.listNotes(
        scopeKind: NoteScopeKind.setting,
        scopeTarget: setting.uuid,
      ),
      isEmpty,
    );
  });

  test('search: free text, tags (quoted match), and scope filters', () async {
    final setting = await store.createSetting('World');
    final scope = NoteScope(
      kind: NoteScopeKind.setting,
      targetUuid: setting.uuid,
    );
    await store.createNote(
      title: 'Fey Court',
      bodyMarkdown: 'rulers',
      scope: scope,
      tagSlugs: const ['fey'],
    );
    await store.createNote(
      title: 'Iron Keep',
      bodyMarkdown: 'fey-realm border fort',
      scope: scope,
      tagSlugs: const ['fey-realm'],
    );

    final byText = await store.searchNotes(q: 'court');
    expect(byText.single.title, 'Fey Court');

    // Tag filter must not treat "fey" as a prefix of "fey-realm".
    final byTag = await store.searchNotes(tagSlugs: const ['fey']);
    expect(byTag.single.title, 'Fey Court');

    final byScope = await store.searchNotes(scopeKind: NoteScopeKind.character);
    expect(byScope, isEmpty);
  });

  test('remote linking: settings and notes track their server uuids', () async {
    final setting = await store.createSetting('Pushed');
    await store.linkSettingRemote(setting.uuid, 'remote-123');
    final linked = await store.getSettingByRemoteUuid('remote-123');
    expect(linked?.uuid, setting.uuid);
    expect(linked?.isLinked, isTrue);
  });

  test('v2→v3 migration adds content_module_uuid (backfilled from data) '
      'and the uninstall tombstone table', () async {
    // A v2-shaped database needs a real file: in-memory databases
    // vanish on close and can't be reopened by LocalStore.open.
    final path =
        '${await databaseFactory.getDatabasesPath()}/migration_test_v2.db';
    await databaseFactory.deleteDatabase(path);
    final v2 = await openDatabase(
      path,
      version: 2,
      onCreate: (db, _) async {
        // Minimal v2 content-table shape: no content_module_uuid column.
        await db.execute('''
          CREATE TABLE spell (
            uuid TEXT PRIMARY KEY NOT NULL,
            key  TEXT NOT NULL UNIQUE,
            slug TEXT NOT NULL,
            name TEXT NOT NULL,
            level INTEGER NOT NULL DEFAULT 0, school_uuid TEXT,
            concentration INTEGER NOT NULL DEFAULT 0,
            ritual INTEGER NOT NULL DEFAULT 0,
            data TEXT NOT NULL
          )
        ''');
        for (final table in LocalStore.contentTables) {
          if (table == 'spell' || table == 'content_module') continue;
          await db.execute('''
            CREATE TABLE IF NOT EXISTS "$table" (
              uuid TEXT PRIMARY KEY NOT NULL,
              key  TEXT NOT NULL UNIQUE,
              slug TEXT NOT NULL,
              name TEXT NOT NULL,
              data TEXT NOT NULL
            )
          ''');
        }
        await db.execute('''
          CREATE TABLE content_module (
            uuid TEXT PRIMARY KEY NOT NULL,
            key  TEXT NOT NULL UNIQUE,
            slug TEXT NOT NULL,
            name TEXT NOT NULL,
            data TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE character (
            uuid TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL,
            data TEXT NOT NULL, created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE setting (
            uuid TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL,
            remote_uuid TEXT, created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
          )
        ''');
        await db.execute('''
          CREATE TABLE lore_note (
            uuid TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL,
            body_markdown TEXT NOT NULL DEFAULT '',
            scope_kind TEXT NOT NULL, scope_target_uuid TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'visible',
            tag_slugs TEXT NOT NULL DEFAULT '[]', remote_uuid TEXT,
            created_at TEXT NOT NULL, updated_at TEXT NOT NULL
          )
        ''');
        await db.insert('spell', {
          'uuid': 'spell-1',
          'key': 'srd_fireball',
          'slug': 'fireball',
          'name': 'Fireball',
          'data':
              '{"uuid":"spell-1","content_module_uuid":"module-1","name":"Fireball"}',
        });
      },
    );
    await v2.close();

    final migrated = await LocalStore.open(path: path);
    final rows = await migrated.database.query(
      'spell',
      columns: ['uuid', 'content_module_uuid'],
    );
    expect(rows.single['content_module_uuid'], 'module-1');
    // Tombstone table exists and is usable.
    await migrated.database.insert('removed_content_module', {
      'slug': 'tob',
      'removed_at': '2026-06-12T00:00:00Z',
    });
    expect((await migrated.database.query('removed_content_module')).length, 1);
    await migrated.close();
    await databaseFactory.deleteDatabase(path);
  });
}
