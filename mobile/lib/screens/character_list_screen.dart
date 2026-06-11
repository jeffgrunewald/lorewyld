// Local character roster — fully offline. Tap a character to open the
// sheet; + creates a new one.

import 'package:flutter/material.dart';

import '../services/local_store.dart';
import '../types/character.dart';
import '../widgets/async_list_view.dart';
import 'character_sheet_screen.dart';

class CharacterListScreen extends StatefulWidget {
  const CharacterListScreen({super.key, required this.store});

  final LocalStore store;

  @override
  State<CharacterListScreen> createState() => _CharacterListScreenState();
}

class _CharacterListScreenState extends State<CharacterListScreen> {
  late Future<List<CharacterSheet>> _future;

  @override
  void initState() {
    super.initState();
    _future = widget.store.listCharacters();
  }

  Future<void> _refresh() async {
    // Block body, not `() => _future = ...`: an arrow closure returns the
    // assigned Future, which setState() rejects.
    setState(() {
      _future = widget.store.listCharacters();
    });
  }

  Future<void> _createCharacter() async {
    final ctl = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('New character'),
        content: TextField(
          controller: ctl,
          decoration: const InputDecoration(
            labelText: 'Name',
            hintText: 'e.g. Thistle Quickfoot',
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
    if (name == null || name.isEmpty) return;
    final sheet = await widget.store.createCharacter(name);
    if (!mounted) return;
    await _refresh();
    _openSheet(sheet);
  }

  void _openSheet(CharacterSheet sheet) {
    Navigator.of(context)
        .push(MaterialPageRoute(
          builder: (_) =>
              CharacterSheetScreen(store: widget.store, sheet: sheet),
        ))
        .then((_) => _refresh());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Characters')),
      body: RefreshIndicator(
        onRefresh: _refresh,
        child: AsyncListView<CharacterSheet>(
          future: _future,
          emptyMessage:
              'No characters yet. Tap + to create one — no server needed.',
          itemBuilder: (_, c) {
            final subtitle = [
              if (c.race.isNotEmpty) c.race,
              if (c.className.isNotEmpty) 'Level ${c.level} ${c.className}',
            ].join(' · ');
            return ListTile(
              leading: CircleAvatar(
                child: Text(c.name.isEmpty ? '?' : c.name[0].toUpperCase()),
              ),
              title: Text(c.name),
              subtitle: subtitle.isEmpty ? null : Text(subtitle),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => _openSheet(c),
            );
          },
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _createCharacter,
        tooltip: 'New character',
        child: const Icon(Icons.add),
      ),
    );
  }
}
