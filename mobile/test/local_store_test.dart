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

    final updated =
        await store.saveCharacter(created.copyWith(level: 3, race: 'Halfling'));
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
          scopeKind: NoteScopeKind.setting, scopeTarget: setting.uuid),
      isEmpty,
    );
  });

  test('search: free text, tags (quoted match), and scope filters',
      () async {
    final setting = await store.createSetting('World');
    final scope =
        NoteScope(kind: NoteScopeKind.setting, targetUuid: setting.uuid);
    await store.createNote(
        title: 'Fey Court',
        bodyMarkdown: 'rulers',
        scope: scope,
        tagSlugs: const ['fey']);
    await store.createNote(
        title: 'Iron Keep',
        bodyMarkdown: 'fey-realm border fort',
        scope: scope,
        tagSlugs: const ['fey-realm']);

    final byText = await store.searchNotes(q: 'court');
    expect(byText.single.title, 'Fey Court');

    // Tag filter must not treat "fey" as a prefix of "fey-realm".
    final byTag = await store.searchNotes(tagSlugs: const ['fey']);
    expect(byTag.single.title, 'Fey Court');

    final byScope =
        await store.searchNotes(scopeKind: NoteScopeKind.character);
    expect(byScope, isEmpty);
  });

  test('remote linking: settings and notes track their server uuids',
      () async {
    final setting = await store.createSetting('Pushed');
    await store.linkSettingRemote(setting.uuid, 'remote-123');
    final linked = await store.getSettingByRemoteUuid('remote-123');
    expect(linked?.uuid, setting.uuid);
    expect(linked?.isLinked, isTrue);
  });
}
