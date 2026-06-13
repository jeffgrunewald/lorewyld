// Edit (or create) a lore note in a fixed scope, entirely against the
// local store — no server required. The MarkdownEditor widget owns the
// inputs; this screen wires it to LocalStore.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../types/lore_note.dart';
import '../widgets/markdown_editor.dart';

class LoreNoteEditScreen extends StatefulWidget {
  const LoreNoteEditScreen({
    super.key,
    required this.store,
    required this.scope,
    this.existing,
  });

  final LocalStore store;
  final NoteScope scope;
  final LocalNote? existing;

  @override
  State<LoreNoteEditScreen> createState() => _LoreNoteEditScreenState();
}

class _LoreNoteEditScreenState extends State<LoreNoteEditScreen> {
  bool _saving = false;
  bool _deleting = false;
  NoteVisibility _visibility = NoteVisibility.visible;

  @override
  void initState() {
    super.initState();
    _visibility = widget.existing?.visibility ?? NoteVisibility.visible;
  }

  Future<void> _save({
    required String title,
    required String body,
    required List<String> tagSlugs,
  }) async {
    if (title.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Title is required.')),
      );
      return;
    }
    setState(() => _saving = true);
    try {
      final existing = widget.existing;
      if (existing == null) {
        await widget.store.createNote(
          title: title,
          bodyMarkdown: body,
          scope: widget.scope,
          visibility: _visibility,
          tagSlugs: tagSlugs,
        );
      } else {
        await widget.store.updateNote(
          uuid: existing.uuid,
          title: title,
          bodyMarkdown: body,
          visibility: _visibility,
          tagSlugs: tagSlugs,
        );
      }
      if (!mounted) return;
      Navigator.of(context).pop();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Save failed: $e')),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _delete() async {
    final existing = widget.existing;
    if (existing == null) return;
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete note?'),
        content: Text('"${existing.title}" will be permanently removed.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton.tonal(
            onPressed: () => Navigator.pop(ctx, true),
            style: FilledButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirm != true) return;
    setState(() => _deleting = true);
    try {
      await widget.store.deleteNote(existing.uuid);
      if (!mounted) return;
      Navigator.of(context).pop();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Delete failed: $e')),
      );
    } finally {
      if (mounted) setState(() => _deleting = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final existing = widget.existing;
    return Scaffold(
      appBar: AppBar(
        title: Text(existing == null ? 'New note' : 'Edit note'),
        actions: [
          PopupMenuButton<NoteVisibility>(
            tooltip: 'Visibility',
            icon: Icon(_visibilityIcon(_visibility)),
            onSelected: (v) => setState(() => _visibility = v),
            itemBuilder: (_) => [
              for (final v in NoteVisibility.values)
                CheckedPopupMenuItem(
                  value: v,
                  checked: _visibility == v,
                  child: Text(_visibilityLabel(v)),
                ),
            ],
          ),
        ],
      ),
      body: MarkdownEditor(
        tagSuggestions: (pattern) =>
            widget.store.suggestTagSlugs(prefix: pattern),
        initialTitle: existing?.title ?? '',
        initialBody: existing?.bodyMarkdown ?? '',
        initialTagSlugs: existing?.tagSlugs ?? const [],
        saving: _saving,
        deleting: _deleting,
        onSave: ({required title, required body, required tagSlugs}) =>
            _save(title: title, body: body, tagSlugs: tagSlugs),
        onDelete: existing == null ? null : _delete,
      ),
    );
  }

  String _visibilityLabel(NoteVisibility v) => switch (v) {
        NoteVisibility.visible => 'Visible to everyone',
        NoteVisibility.authorOnly => 'Only me',
        NoteVisibility.gamemasterOnly => 'GMs only',
      };

  IconData _visibilityIcon(NoteVisibility v) => switch (v) {
        NoteVisibility.visible => Icons.visibility_outlined,
        NoteVisibility.authorOnly => Icons.lock_outline,
        NoteVisibility.gamemasterOnly => Icons.shield_outlined,
      };
}
