// End-to-end sync verification against a REAL running server.
//
// Skipped unless LW_E2E is set in the environment — run with:
//   LW_E2E=1 flutter test test/sync_e2e_test.dart
// against a freshly seeded server on localhost:8080.
//
// Deliberately avoids TestWidgetsFlutterBinding: initializing it would
// install flutter_test's HttpOverrides, which stub out real network IO.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:lorewyld/services/api_client.dart';
import 'package:lorewyld/services/local_store.dart';
import 'package:lorewyld/services/sync_service.dart';
import 'package:lorewyld/types/lore_note.dart';

void main() {
  final enabled = Platform.environment['LW_E2E'] != null;
  final serverUrl =
      Platform.environment['LW_SERVER_URL'] ?? 'http://localhost:8080';
  final joinCode = Platform.environment['LW_JOIN_CODE'];

  test('register → author offline → push → pull → publish round-trip',
      () async {
    sqfliteFfiInit();
    databaseFactory = databaseFactoryFfi;

    final api = ApiClient(baseUri: Uri.parse(serverUrl));
    final suffix = DateTime.now().millisecondsSinceEpoch;

    // ── register with the new username/email/password shape ─────────
    final auth = await api.register(
      joinCode: joinCode!,
      username: 'mobile_e2e_$suffix',
      email: 'mobile_e2e_$suffix@example.com',
      password: 'mobilepass$suffix',
    );
    api.setSessionToken(auth.sessionToken);
    expect(auth.user.username, 'mobile_e2e_$suffix');

    // me() resolves the session
    final me = await api.me();
    expect(me.uuid, auth.user.uuid);

    // ── author entirely locally ─────────────────────────────────────
    final store = await LocalStore.open(path: inMemoryDatabasePath);
    final setting = await store.createSetting('E2E Realm $suffix');
    final scope =
        NoteScope(kind: NoteScopeKind.setting, targetUuid: setting.uuid);
    await store.createNote(
      title: 'Fey Court',
      bodyMarkdown: 'Ancient rulers.',
      scope: scope,
      tagSlugs: const ['npc'],
    );
    await store.createNote(
      title: 'Secret Door',
      bodyMarkdown: 'Behind the falls.',
      scope: scope,
      visibility: NoteVisibility.gamemasterOnly,
    );

    // ── push ────────────────────────────────────────────────────────
    final sync = SyncService(store: store, api: api);
    final pushed = await sync.pushSetting(setting);
    expect(pushed.created, 2);
    expect(pushed.updated, 0);

    final linked = (await store.getSetting(setting.uuid))!;
    expect(linked.remoteUuid, isNotNull);
    final localNotes = await store.listNotes(
        scopeKind: NoteScopeKind.setting, scopeTarget: setting.uuid);
    expect(localNotes.every((n) => n.remoteUuid != null), isTrue);

    // ── second push updates instead of duplicating ──────────────────
    final feyCourt =
        localNotes.firstWhere((n) => n.title == 'Fey Court');
    await store.updateNote(
        uuid: feyCourt.uuid, bodyMarkdown: 'Ancient AND current rulers.');
    final repush = await sync.pushSetting(linked);
    expect(repush.created, 0);
    expect(repush.updated, 2);
    final serverCopy = await api.getLoreNote(feyCourt.remoteUuid!);
    expect(serverCopy.note.bodyMarkdown, 'Ancient AND current rulers.');

    // ── server-side edit pulls back down (LWW overwrite) ────────────
    await api.updateLoreNote(
        uuid: feyCourt.remoteUuid!, title: 'The Fey Court (revised)');
    final pulled = await sync.pullSetting(
      remoteSettingUuid: linked.remoteUuid!,
      remoteSettingName: linked.name,
    );
    expect(pulled.updated, greaterThanOrEqualTo(1));
    final localAfterPull = (await store.getNote(feyCourt.uuid))!;
    expect(localAfterPull.title, 'The Fey Court (revised)');

    // ── publish via remote uuids; server stamps the author email ────
    final publishResult = await api.publishModule(
      sourceSettingUuid: linked.remoteUuid!,
      name: 'E2E Module $suffix',
      slug: 'e2e-module-$suffix',
      license: 'CC-BY 4.0',
      versionString: '1.0.0',
      selectedNoteUuids: [feyCourt.remoteUuid!],
    );
    final authors =
        (publishResult['module']['authors'] as List<dynamic>).cast<String>();
    expect(authors, contains('mobile_e2e_$suffix@example.com'));

    // ── logout revokes the session ──────────────────────────────────
    await api.logout();
    await expectLater(api.me(), throwsA(isA<ApiException>()));

    await store.close();
  }, skip: enabled ? false : 'set LW_E2E=1 with a running server to enable');
}
