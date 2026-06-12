// Searchable picker over one content table, shown as a modal bottom
// sheet. Used everywhere the character builder selects from installed
// content (species, classes, backgrounds, spells, items) instead of
// accepting free text.

import 'package:flutter/material.dart';

import '../compendium/categories.dart';
import '../services/content_store.dart';

/// Opens the picker and resolves to the chosen record, or null if the
/// sheet is dismissed without a choice.
Future<Map<String, dynamic>?> showContentPicker({
  required BuildContext context,
  required ContentStore content,
  required String table,
  required String title,
  String? where,
  List<Object?>? whereArgs,
}) {
  return showModalBottomSheet<Map<String, dynamic>>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    builder: (_) => _ContentPickerSheet(
      content: content,
      table: table,
      title: title,
      where: where,
      whereArgs: whereArgs,
    ),
  );
}

class _ContentPickerSheet extends StatefulWidget {
  const _ContentPickerSheet({
    required this.content,
    required this.table,
    required this.title,
    this.where,
    this.whereArgs,
  });

  final ContentStore content;
  final String table;
  final String title;
  final String? where;
  final List<Object?>? whereArgs;

  @override
  State<_ContentPickerSheet> createState() => _ContentPickerSheetState();
}

class _ContentPickerSheetState extends State<_ContentPickerSheet> {
  final _searchCtl = TextEditingController();
  late final CompendiumCategory _category = categoryFor(widget.table);
  ContentLookups _lookups = const ContentLookups();
  late Future<List<Map<String, dynamic>>> _future = _query('');

  @override
  void initState() {
    super.initState();
    ContentLookups.load(widget.content).then((l) {
      if (mounted) setState(() => _lookups = l);
    });
  }

  @override
  void dispose() {
    _searchCtl.dispose();
    super.dispose();
  }

  Future<List<Map<String, dynamic>>> _query(String query) =>
      widget.content.listNamed(
        widget.table,
        query: query,
        where: widget.where,
        whereArgs: widget.whereArgs,
      );

  @override
  Widget build(BuildContext context) {
    final height = MediaQuery.of(context).size.height * 0.85;
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;
    return SizedBox(
      height: height,
      child: Padding(
        padding: EdgeInsets.only(bottom: bottomInset),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
              child: Text(widget.title,
                  style: Theme.of(context).textTheme.titleMedium),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: TextField(
                controller: _searchCtl,
                autofocus: true,
                decoration: const InputDecoration(
                  hintText: 'Search by name…',
                  prefixIcon: Icon(Icons.search),
                  border: OutlineInputBorder(),
                ),
                onChanged: (q) => setState(() {
                  _future = _query(q);
                }),
              ),
            ),
            Expanded(
              child: FutureBuilder<List<Map<String, dynamic>>>(
                future: _future,
                builder: (context, snap) {
                  if (snap.connectionState != ConnectionState.done) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  final rows = snap.data ?? const [];
                  if (rows.isEmpty) {
                    return const Center(child: Text('No matches.'));
                  }
                  return ListView.builder(
                    itemCount: rows.length,
                    itemBuilder: (_, i) {
                      final record = rows[i];
                      final subtitle = _category.subtitle(record, _lookups);
                      return ListTile(
                        title: Text(_category.displayName(record)),
                        subtitle: (subtitle == null || subtitle.isEmpty)
                            ? null
                            : Text(subtitle),
                        onTap: () => Navigator.of(context).pop(record),
                      );
                    },
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Read-only form field that opens a content picker on tap; displays
/// the current selection or a placeholder.
class ContentPickerField extends StatelessWidget {
  const ContentPickerField({
    super.key,
    required this.label,
    required this.value,
    required this.onTap,
    this.onCleared,
  });

  final String label;
  final String value;
  final VoidCallback onTap;
  final VoidCallback? onCleared;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: InputDecorator(
        decoration: InputDecoration(
          labelText: label,
          border: const OutlineInputBorder(),
          suffixIcon: value.isNotEmpty && onCleared != null
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  tooltip: 'Clear',
                  onPressed: onCleared,
                )
              : const Icon(Icons.arrow_drop_down),
        ),
        isEmpty: value.isEmpty,
        child: value.isEmpty ? null : Text(value),
      ),
    );
  }
}
