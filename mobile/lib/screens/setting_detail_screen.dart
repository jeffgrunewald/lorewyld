// Setting detail — the setting's local lore notes with edit/create
// affordances, plus the server actions: push (upload local content),
// pull (overwrite local from server), and promote-to-module (requires
// the setting to have been pushed first).

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../services/server_connection.dart';
import '../services/sync_service.dart';
import '../types/lore_note.dart';
import '../widgets/async_list_view.dart';
import 'lore_note_edit_screen.dart';
import 'promote_module_wizard_screen.dart';

class SettingDetailScreen extends StatefulWidget {
  const SettingDetailScreen({
    super.key,
    required this.connection,
    required this.store,
    required this.setting,
  });

  final ServerConnection connection;
  final LocalStore store;
  final LocalSetting setting;

  @override
  State<SettingDetailScreen> createState() => _SettingDetailScreenState();
}

class _SettingDetailScreenState extends State<SettingDetailScreen> {
  late LocalSetting _setting;
  late Future<List<LocalNote>> _notesFuture;
  bool _syncing = false;

  @override
  void initState() {
    super.initState();
    _setting = widget.setting;
    _notesFuture = _load();
  }

  Future<List<LocalNote>> _load() {
    return widget.store.listNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: _setting.uuid,
    );
  }

  Future<void> _refresh() async {
    final reloaded = await widget.store.getSetting(_setting.uuid);
    if (!mounted) return;
    setState(() {
      if (reloaded != null) _setting = reloaded;
      _notesFuture = _load();
    });
  }

  void _openNote(LocalNote? existing) {
    Navigator.of(context)
        .push(
          MaterialPageRoute(
            builder: (_) => LoreNoteEditScreen(
              store: widget.store,
              scope: NoteScope(
                kind: NoteScopeKind.setting,
                targetUuid: _setting.uuid,
              ),
              existing: existing,
            ),
          ),
        )
        .then((_) => _refresh());
  }

  bool _requireLogin() {
    if (widget.connection.isLoggedIn) return false;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Log in to a server first (cloud icon, top right).'),
      ),
    );
    return true;
  }

  Future<void> _push() async {
    if (_requireLogin()) return;
    setState(() => _syncing = true);
    try {
      final sync = SyncService(
        store: widget.store,
        api: widget.connection.api!,
      );
      final result = await sync.pushSetting(_setting);
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(result.describe('Pushed'))));
      await _refresh();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Push failed: $e')));
    } finally {
      if (mounted) setState(() => _syncing = false);
    }
  }

  Future<void> _pull() async {
    if (_requireLogin()) return;
    final remoteUuid = _setting.remoteUuid;
    if (remoteUuid == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'This setting has never been pushed — there is nothing to pull.',
          ),
        ),
      );
      return;
    }
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Pull from server?'),
        content: const Text(
          'Server versions overwrite the local copies of previously '
          'synced notes. Local-only notes are kept.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Pull'),
          ),
        ],
      ),
    );
    if (confirm != true) return;
    setState(() => _syncing = true);
    try {
      final sync = SyncService(
        store: widget.store,
        api: widget.connection.api!,
      );
      final result = await sync.pullSetting(
        remoteSettingUuid: remoteUuid,
        remoteSettingName: _setting.name,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(result.describe('Pulled'))));
      await _refresh();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Pull failed: $e')));
    } finally {
      if (mounted) setState(() => _syncing = false);
    }
  }

  Future<void> _openPromoteWizard() async {
    if (_requireLogin()) return;
    if (_setting.remoteUuid == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'Push this setting to the server before publishing it.',
          ),
        ),
      );
      return;
    }
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => PromoteModuleWizardScreen(
          connection: widget.connection,
          store: widget.store,
          setting: _setting,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_setting.name),
        actions: [
          if (_syncing)
            const Padding(
              padding: EdgeInsets.all(14),
              child: SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            )
          else ...[
            IconButton(
              icon: const Icon(Icons.cloud_upload_outlined),
              tooltip: 'Push to server',
              onPressed: _push,
            ),
            IconButton(
              icon: const Icon(Icons.cloud_download_outlined),
              tooltip: 'Pull from server',
              onPressed: _pull,
            ),
            IconButton(
              icon: const Icon(Icons.publish),
              tooltip: 'Promote to module',
              onPressed: _openPromoteWizard,
            ),
          ],
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: AsyncListView<LocalNote>(
          future: _notesFuture,
          emptyMessage: 'No lore notes in this setting yet. Tap + to add one.',
          itemBuilder: (_, note) {
            final tagStr = note.tagSlugs.join(' · ');
            return ListTile(
              title: Text(note.title),
              subtitle: tagStr.isEmpty ? null : Text(tagStr),
              trailing: note.remoteUuid == null
                  ? const Tooltip(
                      message: 'Local only — not yet pushed',
                      child: Icon(Icons.cloud_off_outlined, size: 18),
                    )
                  : null,
              onTap: () => _openNote(note),
            );
          },
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _openNote(null),
        tooltip: 'New lore note',
        child: const Icon(Icons.add),
      ),
    );
  }
}
