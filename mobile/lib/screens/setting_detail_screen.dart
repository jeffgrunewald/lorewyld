// Setting detail — shows the setting's lore notes with edit/create
// affordances. Tap a note to edit it; tap + to create a new one in
// this setting's scope.

import 'package:flutter/material.dart';

import '../services/server_connection.dart';
import '../types/lore_note.dart';
import '../types/setting.dart';
import 'lore_note_edit_screen.dart';
import 'promote_module_wizard_screen.dart';

class SettingDetailScreen extends StatefulWidget {
  const SettingDetailScreen({
    super.key,
    required this.connection,
    required this.setting,
  });

  final ServerConnection connection;
  final Setting setting;

  @override
  State<SettingDetailScreen> createState() => _SettingDetailScreenState();
}

class _SettingDetailScreenState extends State<SettingDetailScreen> {
  late Future<List<LoreNoteWithTags>> _notesFuture;

  @override
  void initState() {
    super.initState();
    _notesFuture = _load();
  }

  Future<List<LoreNoteWithTags>> _load() {
    return widget.connection.api!.listLoreNotes(
      scopeKind: NoteScopeKind.setting,
      scopeTarget: widget.setting.uuid,
    );
  }

  Future<void> _refresh() async {
    setState(() => _notesFuture = _load());
  }

  void _openNote(LoreNoteWithTags? existing) {
    Navigator.of(context)
        .push(MaterialPageRoute(
          builder: (_) => LoreNoteEditScreen(
            connection: widget.connection,
            scope: NoteScope(
              kind: NoteScopeKind.setting,
              targetUuid: widget.setting.uuid,
            ),
            existing: existing,
          ),
        ))
        .then((_) => _refresh());
  }

  void _openPromoteWizard() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => PromoteModuleWizardScreen(
          connection: widget.connection,
          setting: widget.setting,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.setting.name),
        actions: [
          IconButton(
            icon: const Icon(Icons.publish),
            tooltip: 'Promote to module',
            onPressed: _openPromoteWizard,
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: FutureBuilder<List<LoreNoteWithTags>>(
          future: _notesFuture,
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
            final notes = snap.data ?? const [];
            if (notes.isEmpty) {
              return ListView(
                children: const [
                  SizedBox(height: 80),
                  Center(
                    child: Padding(
                      padding: EdgeInsets.symmetric(horizontal: 24),
                      child: Text(
                        'No lore notes in this setting yet. Tap + to add one.',
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ),
                ],
              );
            }
            return ListView.separated(
              itemCount: notes.length,
              separatorBuilder: (_, __) => const Divider(height: 1),
              itemBuilder: (_, i) {
                final entry = notes[i];
                final tagStr = entry.tags.map((t) => t.slug).join(' · ');
                return ListTile(
                  title: Text(entry.note.title),
                  subtitle: tagStr.isEmpty ? null : Text(tagStr),
                  onTap: () => _openNote(entry),
                );
              },
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
