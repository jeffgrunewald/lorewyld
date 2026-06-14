// Local settings (worldbuilding workspaces). Authoring never needs a
// server; the cloud-download action pulls server settings into the
// local store when logged in.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../services/server_connection.dart';
import '../services/sync_service.dart';
import '../widgets/async_list_view.dart';
import 'setting_detail_screen.dart';

class SettingListScreen extends StatefulWidget {
  const SettingListScreen({
    super.key,
    required this.connection,
    required this.store,
  });

  final ServerConnection connection;
  final LocalStore store;

  @override
  State<SettingListScreen> createState() => _SettingListScreenState();
}

class _SettingListScreenState extends State<SettingListScreen> {
  late Future<List<LocalSetting>> _settingsFuture;

  @override
  void initState() {
    super.initState();
    _settingsFuture = widget.store.listSettings();
  }

  Future<void> _refresh() async {
    setState(() {
      _settingsFuture = widget.store.listSettings();
    });
  }

  Future<void> _createSetting() async {
    final ctl = TextEditingController();
    final result = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('New setting'),
        content: TextField(
          controller: ctl,
          decoration: const InputDecoration(
            labelText: 'Name',
            hintText: 'e.g. Verdant Realms',
          ),
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, ctl.text.trim()),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    if (result == null || result.isEmpty) return;
    final created = await widget.store.createSetting(result);
    if (!mounted) return;
    await _refresh();
    _openSetting(created);
  }

  Future<void> _pullFromServer() async {
    final connection = widget.connection;
    if (!connection.isLoggedIn) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Log in to a server to pull settings from it.'),
        ),
      );
      return;
    }
    final sync = SyncService(store: widget.store, api: connection.api!);
    final List<RemoteSettingCandidate> candidates;
    try {
      candidates = await sync.listRemoteSettings();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Failed to list server settings: $e')),
      );
      return;
    }
    if (!mounted) return;
    if (candidates.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No settings on the server yet.')),
      );
      return;
    }
    final picked = await showDialog<RemoteSettingCandidate>(
      context: context,
      builder: (ctx) => SimpleDialog(
        title: const Text('Pull setting from server'),
        children: [
          for (final c in candidates)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(ctx, c),
              child: ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(c.name),
                subtitle: Text(
                  c.isLinked
                      ? 'Already linked — pull overwrites the local copy'
                      : 'New local copy',
                ),
              ),
            ),
        ],
      ),
    );
    if (picked == null || !mounted) return;
    try {
      final result = await sync.pullSetting(
        remoteSettingUuid: picked.remoteUuid,
        remoteSettingName: picked.name,
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
    }
  }

  void _openSetting(LocalSetting s) {
    Navigator.of(context)
        .push(
          MaterialPageRoute(
            builder: (_) => SettingDetailScreen(
              connection: widget.connection,
              store: widget.store,
              setting: s,
            ),
          ),
        )
        .then((_) => _refresh());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings & lore'),
        actions: [
          IconButton(
            icon: const Icon(Icons.cloud_download_outlined),
            tooltip: 'Pull from server',
            onPressed: _pullFromServer,
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: AsyncListView<LocalSetting>(
          future: _settingsFuture,
          emptyMessage:
              'No settings yet. Tap + to create one and start authoring lore notes — no server needed.',
          itemBuilder: (_, s) => ListTile(
            title: Text(s.name),
            subtitle: Text(s.isLinked ? 'Synced with server' : 'Local only'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => _openSetting(s),
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _createSetting,
        tooltip: 'New setting',
        child: const Icon(Icons.add),
      ),
    );
  }
}
