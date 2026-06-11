// Search across local lore notes — free-text + tag-chip filters,
// scope-faceted. Fully offline: this searches the on-device store.
// Tapping a result opens the note for editing.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../types/lore_note.dart';
import '../widgets/markdown_editor.dart';
import 'lore_note_edit_screen.dart';

class SearchScreen extends StatefulWidget {
  const SearchScreen({super.key, required this.store});

  final LocalStore store;

  @override
  State<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends State<SearchScreen> {
  final _qCtl = TextEditingController();
  List<String> _tagSlugs = [];
  NoteScopeKind? _scopeKind;
  Future<List<LocalNote>>? _resultsFuture;

  @override
  void dispose() {
    _qCtl.dispose();
    super.dispose();
  }

  void _runSearch() {
    final q = _qCtl.text.trim();
    setState(() {
      _resultsFuture = widget.store.searchNotes(
        q: q.isEmpty ? null : q,
        scopeKind: _scopeKind,
        tagSlugs: _tagSlugs,
      );
    });
  }

  void _clearFilters() {
    setState(() {
      _qCtl.clear();
      _tagSlugs = [];
      _scopeKind = null;
      _resultsFuture = null;
    });
  }

  Widget _scopeChip(NoteScopeKind? kind, String label) {
    final selected = _scopeKind == kind;
    return ChoiceChip(
      label: Text(label),
      selected: selected,
      onSelected: (_) {
        setState(() => _scopeKind = selected ? null : kind);
        _runSearch();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Search')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
            child: TextField(
              controller: _qCtl,
              decoration: InputDecoration(
                labelText: 'Search your lore',
                hintText: 'Free-text across titles + bodies',
                border: const OutlineInputBorder(),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.search),
                  onPressed: _runSearch,
                ),
              ),
              textInputAction: TextInputAction.search,
              onSubmitted: (_) => _runSearch(),
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: TagChipInput(
              suggestions: (pattern) =>
                  widget.store.suggestTagSlugs(prefix: pattern),
              slugs: _tagSlugs,
              onChanged: (slugs) {
                setState(() => _tagSlugs = slugs);
                _runSearch();
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  _scopeChip(null, 'All scopes'),
                  const SizedBox(width: 6),
                  _scopeChip(NoteScopeKind.setting, 'Settings'),
                  const SizedBox(width: 6),
                  _scopeChip(NoteScopeKind.character, 'Characters'),
                  const SizedBox(width: 6),
                  TextButton(
                    onPressed: _clearFilters,
                    child: const Text('Clear'),
                  ),
                ],
              ),
            ),
          ),
          const Divider(height: 1),
          Expanded(child: _buildResults()),
        ],
      ),
    );
  }

  Widget _buildResults() {
    final future = _resultsFuture;
    if (future == null) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Text(
            'Enter a query or pick a tag to start searching.',
            textAlign: TextAlign.center,
          ),
        ),
      );
    }
    return FutureBuilder<List<LocalNote>>(
      future: future,
      builder: (context, snap) {
        if (snap.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }
        if (snap.hasError) {
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Text('Search failed: ${snap.error}'),
            ),
          );
        }
        final results = snap.data ?? const [];
        if (results.isEmpty) {
          return const Center(
            child: Text('No results.'),
          );
        }
        return ListView.separated(
          itemCount: results.length,
          separatorBuilder: (_, _) => const Divider(height: 1),
          itemBuilder: (_, i) {
            final note = results[i];
            final tagStr = note.tagSlugs.join(' · ');
            return ListTile(
              title: Text(note.title),
              subtitle: Text(
                [
                  'in ${note.scope.kind.wire}',
                  if (tagStr.isNotEmpty) tagStr,
                ].join(' · '),
              ),
              onTap: () {
                Navigator.of(context)
                    .push(MaterialPageRoute(
                      builder: (_) => LoreNoteEditScreen(
                        store: widget.store,
                        scope: note.scope,
                        existing: note,
                      ),
                    ))
                    // Re-run the search so an edit (or delete) made in
                    // the editor is reflected in the results list.
                    .then((_) {
                      if (mounted) _runSearch();
                    });
              },
            );
          },
        );
      },
    );
  }
}
