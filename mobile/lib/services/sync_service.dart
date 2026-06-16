// Per-setting push/pull between the local store and a connected server.
//
// Model: last-write-wins, no merge. Local records carry a `remote_uuid`
// linking them to their server counterpart. Push overwrites the server
// copy from local; pull overwrites the local copy from the server.
// Deletions do not propagate in either direction — a record deleted on
// one side simply stops being updated on the other.

import '../types/lore_note.dart';
import 'api_client.dart';
import 'local_store.dart';

class SyncResult {
  final int created;
  final int updated;

  const SyncResult({required this.created, required this.updated});

  String describe(String verb) =>
      '$verb ${created + updated} notes ($created new, $updated updated)';
}

class SyncService {
  final LocalStore store;
  final ApiClient api;

  SyncService({required this.store, required this.api});

  /// Uploads a local setting and all its notes to the server. First push
  /// creates the server records and stores the uuid mapping; later
  /// pushes update them in place.
  Future<SyncResult> pushSetting(LocalSetting setting) async {
    var remoteUuid = setting.remoteUuid;
    if (remoteUuid == null) {
      final created = await api.createSetting(name: setting.name);
      remoteUuid = created.uuid;
      await store.linkSettingRemote(setting.uuid, remoteUuid);
    } else {
      await api.updateSetting(uuid: remoteUuid, name: setting.name);
    }

    var created = 0;
    var updated = 0;
    final notes = await store.listNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: setting.uuid,
    );
    for (final note in notes) {
      if (note.remoteUuid == null) {
        final remote = await api.createLoreNote(
          title: note.title,
          bodyMarkdown: note.bodyMarkdown,
          scope: NoteScope(kind: NoteScopeKind.setting, targetUuid: remoteUuid),
          visibility: note.visibility,
          tagSlugs: note.tagSlugs,
        );
        await store.updateNote(uuid: note.uuid, remoteUuid: remote.note.uuid);
        created++;
      } else {
        await api.updateLoreNote(
          uuid: note.remoteUuid!,
          title: note.title,
          bodyMarkdown: note.bodyMarkdown,
          visibility: note.visibility,
          tagSlugs: note.tagSlugs,
        );
        updated++;
      }
    }
    return SyncResult(created: created, updated: updated);
  }

  /// Server settings owned by (or shared with) the logged-in user,
  /// offered as pull candidates.
  Future<List<RemoteSettingCandidate>> listRemoteSettings() async {
    final remote = await api.listSettings();
    final candidates = <RemoteSettingCandidate>[];
    for (final s in remote) {
      final local = await store.getSettingByRemoteUuid(s.uuid);
      candidates.add(
        RemoteSettingCandidate(
          remoteUuid: s.uuid,
          name: s.name,
          alreadyLinkedLocalUuid: local?.uuid,
        ),
      );
    }
    return candidates;
  }

  /// Downloads a server setting and its notes into the local store,
  /// overwriting any previously pulled local copies (matched by
  /// remote_uuid). Local notes never pushed/pulled are left alone.
  Future<SyncResult> pullSetting({
    required String remoteSettingUuid,
    required String remoteSettingName,
  }) async {
    var local = await store.getSettingByRemoteUuid(remoteSettingUuid);
    if (local == null) {
      local = await store.createSetting(
        remoteSettingName,
        remoteUuid: remoteSettingUuid,
      );
    } else {
      await store.renameSetting(local.uuid, remoteSettingName);
    }

    final remoteNotes = await api.listLoreNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: remoteSettingUuid,
    );

    final localNotes = await store.listNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: local.uuid,
    );
    final byRemoteUuid = {
      for (final n in localNotes)
        if (n.remoteUuid != null) n.remoteUuid!: n,
    };

    var created = 0;
    var updated = 0;
    for (final remote in remoteNotes) {
      final slugs = remote.tags.map((t) => t.slug).toList();
      final existing = byRemoteUuid[remote.note.uuid];
      if (existing == null) {
        await store.createNote(
          title: remote.note.title,
          bodyMarkdown: remote.note.bodyMarkdown,
          scope: NoteScope(kind: NoteScopeKind.setting, targetUuid: local.uuid),
          visibility: remote.note.visibility,
          tagSlugs: slugs,
          remoteUuid: remote.note.uuid,
        );
        created++;
      } else {
        await store.updateNote(
          uuid: existing.uuid,
          title: remote.note.title,
          bodyMarkdown: remote.note.bodyMarkdown,
          visibility: remote.note.visibility,
          tagSlugs: slugs,
        );
        updated++;
      }
    }
    return SyncResult(created: created, updated: updated);
  }
}

class RemoteSettingCandidate {
  final String remoteUuid;
  final String name;
  final String? alreadyLinkedLocalUuid;

  const RemoteSettingCandidate({
    required this.remoteUuid,
    required this.name,
    this.alreadyLinkedLocalUuid,
  });

  bool get isLinked => alreadyLinkedLocalUuid != null;
}
