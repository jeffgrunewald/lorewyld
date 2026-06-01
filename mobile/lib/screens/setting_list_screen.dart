// List of the current user's settings (worldbuilding workspaces).
// Tap any to drill into its lore notes.

import 'package:flutter/material.dart';

import '../services/server_connection.dart';
import '../types/setting.dart';
import 'setting_detail_screen.dart';

class SettingListScreen extends StatefulWidget {
  const SettingListScreen({super.key, required this.connection});

  final ServerConnection connection;

  @override
  State<SettingListScreen> createState() => _SettingListScreenState();
}

class _SettingListScreenState extends State<SettingListScreen> {
  late Future<List<Setting>> _settingsFuture;

  @override
  void initState() {
    super.initState();
    _settingsFuture = _load();
  }

  Future<List<Setting>> _load() {
    return widget.connection.api!.listSettings();
  }

  Future<void> _refresh() async {
    setState(() => _settingsFuture = _load());
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
    try {
      final created = await widget.connection.api!.createSetting(name: result);
      if (!mounted) return;
      await _refresh();
      _openSetting(created);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Failed: $e')),
      );
    }
  }

  void _openSetting(Setting s) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => SettingDetailScreen(
          connection: widget.connection,
          setting: s,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: FutureBuilder<List<Setting>>(
          future: _settingsFuture,
          builder: (context, snap) {
            if (snap.connectionState == ConnectionState.waiting) {
              return const Center(child: CircularProgressIndicator());
            }
            if (snap.hasError) {
              return ListView(
                children: [
                  Padding(
                    padding: const EdgeInsets.all(24),
                    child: Text('Failed to load: ${snap.error}'),
                  ),
                ],
              );
            }
            final settings = snap.data ?? const [];
            if (settings.isEmpty) {
              return ListView(
                children: const [
                  SizedBox(height: 80),
                  Center(
                    child: Padding(
                      padding: EdgeInsets.symmetric(horizontal: 24),
                      child: Text(
                        'No settings yet. Tap + to create one and start authoring lore notes.',
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ),
                ],
              );
            }
            return ListView.separated(
              itemCount: settings.length,
              separatorBuilder: (_, __) => const Divider(height: 1),
              itemBuilder: (_, i) {
                final s = settings[i];
                return ListTile(
                  title: Text(s.name),
                  subtitle: s.publishedAsModuleUuid != null
                      ? const Text('Published')
                      : const Text('Draft'),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () => _openSetting(s),
                );
              },
            );
          },
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
