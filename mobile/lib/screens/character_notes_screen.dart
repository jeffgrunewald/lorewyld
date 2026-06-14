// Character-scoped lore notes — backstory, journal entries, goals.
// Entirely local, reusing the markdown note editor.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../types/character.dart';
import '../types/lore_note.dart';
import '../widgets/async_list_view.dart';
import 'lore_note_edit_screen.dart';

class CharacterNotesScreen extends StatefulWidget {
  const CharacterNotesScreen({
    super.key,
    required this.store,
    required this.character,
  });

  final LocalStore store;
  final CharacterSheet character;

  @override
  State<CharacterNotesScreen> createState() => _CharacterNotesScreenState();
}

class _CharacterNotesScreenState extends State<CharacterNotesScreen> {
  late Future<List<LocalNote>> _future;

  NoteScope get _scope => NoteScope(
    kind: NoteScopeKind.character,
    targetUuid: widget.character.uuid,
  );

  @override
  void initState() {
    super.initState();
    _future = _load();
  }

  Future<List<LocalNote>> _load() => widget.store.listNotes(
    scopeKind: NoteScopeKind.character,
    scopeTarget: widget.character.uuid,
  );

  Future<void> _refresh() async {
    setState(() {
      _future = _load();
    });
  }

  void _openNote(LocalNote? existing) {
    Navigator.of(context)
        .push(
          MaterialPageRoute(
            builder: (_) => LoreNoteEditScreen(
              store: widget.store,
              scope: _scope,
              existing: existing,
            ),
          ),
        )
        .then((_) => _refresh());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('${widget.character.name} — notes')),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: AsyncListView<LocalNote>(
          future: _future,
          emptyMessage:
              'No notes for this character yet. Tap + to write a backstory or journal entry.',
          itemBuilder: (_, note) {
            final tagStr = note.tagSlugs.join(' · ');
            return ListTile(
              title: Text(note.title),
              subtitle: tagStr.isEmpty ? null : Text(tagStr),
              onTap: () => _openNote(note),
            );
          },
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _openNote(null),
        tooltip: 'New note',
        child: const Icon(Icons.add),
      ),
    );
  }
}
