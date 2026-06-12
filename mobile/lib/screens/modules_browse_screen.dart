// Read-only browser for content modules installed on the server.
// Mirrors the web /modules page (lists modules; tap a module to see
// its lore notes).

import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../services/server_connection.dart';
import '../types/content_module.dart';
import '../types/lore_note.dart';
import '../widgets/async_list_view.dart';

class ModulesBrowseScreen extends StatefulWidget {
  const ModulesBrowseScreen({super.key, required this.connection});

  final ServerConnection connection;

  @override
  State<ModulesBrowseScreen> createState() => _ModulesBrowseScreenState();
}

class _ModulesBrowseScreenState extends State<ModulesBrowseScreen> {
  late Future<List<ContentModule>> _modulesFuture;

  @override
  void initState() {
    super.initState();
    _modulesFuture = _load();
  }

  Future<List<ContentModule>> _load() {
    return widget.connection.api!.serverInfo().then((info) => info.modules);
  }

  Future<void> _refresh() async {
    setState(() {
      _modulesFuture = _load();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Modules')),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: AsyncListView<ContentModule>(
          future: _modulesFuture,
          emptyMessage: 'No modules published on this server yet.',
          itemBuilder: (context, module) => _ModuleTile(
            module: module,
            onTap: () {
              Navigator.of(context).push(MaterialPageRoute(
                builder: (_) => ModuleDetailScreen(
                  connection: widget.connection,
                  module: module,
                ),
              ));
            },
          ),
        ),
      ),
    );
  }
}

class _ModuleTile extends StatelessWidget {
  const _ModuleTile({required this.module, required this.onTap});

  final ContentModule module;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Row(
        children: [
          Expanded(child: Text(module.name)),
          Text(
            'v${module.versionString}',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ],
      ),
      subtitle: Text(
        [
          licenseDisplayName(module.license),
          'by ${module.authors.isEmpty ? "unknown" : module.authors.join(", ")}',
        ].join(' · '),
      ),
      trailing: const Icon(Icons.chevron_right),
      onTap: onTap,
    );
  }
}

class ModuleDetailScreen extends StatefulWidget {
  const ModuleDetailScreen({
    super.key,
    required this.connection,
    required this.module,
  });

  final ServerConnection connection;
  final ContentModule module;

  @override
  State<ModuleDetailScreen> createState() => _ModuleDetailScreenState();
}

class _ModuleDetailScreenState extends State<ModuleDetailScreen> {
  late Future<List<LoreNoteWithTags>> _notesFuture;

  @override
  void initState() {
    super.initState();
    _notesFuture = widget.connection.api!.listLoreNotes(
      scopeKind: NoteScopeKind.module,
      scopeTarget: widget.module.uuid,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(widget.module.name)),
      body: FutureBuilder<List<LoreNoteWithTags>>(
        future: _notesFuture,
        builder: (context, snap) {
          if (snap.connectionState == ConnectionState.waiting) {
            return const Center(child: CircularProgressIndicator());
          }
          if (snap.hasError) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Text('Failed: ${snap.error}'),
              ),
            );
          }
          final notes = snap.data ?? const [];
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              if (widget.module.description != null) ...[
                Text(widget.module.description!,
                    style: Theme.of(context).textTheme.bodyLarge),
                const SizedBox(height: 16),
              ],
              if (notes.isEmpty)
                const Center(child: Text('No lore notes in this module.'))
              else
                ...notes.map((n) => _NoteCard(entry: n)),
            ],
          );
        },
      ),
    );
  }
}

class _NoteCard extends StatelessWidget {
  const _NoteCard({required this.entry});

  final LoreNoteWithTags entry;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(entry.note.title,
                style: Theme.of(context).textTheme.titleMedium),
            if (entry.tags.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Wrap(
                  spacing: 6,
                  runSpacing: 4,
                  children: entry.tags
                      .map((t) => Chip(
                            label: Text(t.slug),
                            visualDensity: VisualDensity.compact,
                          ))
                      .toList(),
                ),
              ),
            const SizedBox(height: 8),
            MarkdownBody(data: entry.note.bodyMarkdown),
          ],
        ),
      ),
    );
  }
}
