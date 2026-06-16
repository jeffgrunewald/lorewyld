// Searchable picker over one content table, shown as a modal bottom
// sheet. Used everywhere the character builder selects from installed
// content (species, classes, backgrounds, spells, items) instead of
// accepting free text.
//
// The funnel icon opens per-table filters (source document everywhere;
// type/rarity for items; level/school for spells) plus sort order.
// Records load once; search, filters, and sort all apply in memory —
// the largest table is ~1.2k rows.

import 'package:flutter/material.dart';

import '../compendium/categories.dart';
import '../compendium/filters.dart';
import '../services/content_store.dart';
import 'filter_sheet.dart';
import 'source_badge.dart';

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
  late final List<FilterDimension> _dimensions = filterDimensionsFor(
    widget.table,
  );
  late final List<PickerSort> _sorts = sortOptionsFor(widget.table);
  late final FilterState _filters = FilterState(_sorts.first);

  ContentLookups _lookups = const ContentLookups();
  List<Map<String, dynamic>>? _all;
  String _query = '';

  @override
  void initState() {
    super.initState();
    ContentLookups.load(widget.content).then((l) {
      if (mounted) setState(() => _lookups = l);
    });
    widget.content
        .listNamed(
          widget.table,
          where: widget.where,
          whereArgs: widget.whereArgs,
        )
        .then((rows) {
          if (mounted) setState(() => _all = rows);
        });
  }

  @override
  void dispose() {
    _searchCtl.dispose();
    super.dispose();
  }

  List<Map<String, dynamic>> get _visible {
    final all = _all;
    if (all == null) return const [];
    final q = _query.trim().toLowerCase();
    final rows = [
      for (final r in all)
        if ((q.isEmpty || '${r['name']}'.toLowerCase().contains(q)) &&
            matchesFilters(r, _dimensions, _filters.selections))
          r,
    ];
    rows.sort((a, b) => _filters.sort.compare(a, b, _lookups));
    return rows;
  }

  @override
  Widget build(BuildContext context) {
    final height = MediaQuery.of(context).size.height * 0.85;
    final bottomInset = MediaQuery.of(context).viewInsets.bottom;
    final rows = _visible;
    return SizedBox(
      height: height,
      child: Padding(
        padding: EdgeInsets.only(bottom: bottomInset),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
              child: Text(
                widget.title,
                style: Theme.of(context).textTheme.titleMedium,
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _searchCtl,
                      autofocus: true,
                      decoration: const InputDecoration(
                        hintText: 'Search by name…',
                        prefixIcon: Icon(Icons.search),
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (q) => setState(() => _query = q),
                    ),
                  ),
                  if (_dimensions.isNotEmpty) ...[
                    const SizedBox(width: 8),
                    Badge.count(
                      count: _filters.activeCount,
                      isLabelVisible: _filters.activeCount > 0,
                      child: IconButton(
                        icon: const Icon(Icons.filter_list),
                        tooltip: 'Filter & sort',
                        onPressed: _all == null
                            ? null
                            : () => showContentFilterSheet(
                                context: context,
                                dimensions: _dimensions,
                                sorts: _sorts,
                                lookups: _lookups,
                                records: _all!,
                                state: _filters,
                                onChanged: () => setState(() {}),
                              ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            Expanded(
              child: _all == null
                  ? const Center(child: CircularProgressIndicator())
                  : rows.isEmpty
                  ? const Center(child: Text('No matches.'))
                  : ListView.builder(
                      itemCount: rows.length,
                      itemBuilder: (_, i) {
                        final record = rows[i];
                        final subtitle = _category.subtitle(record, _lookups);
                        final source = _lookups.sourceSlugOf(record);
                        return ListTile(
                          title: Text(_category.displayName(record)),
                          subtitle: (subtitle == null || subtitle.isEmpty)
                              ? null
                              : Text(subtitle),
                          trailing: source == null ? null : SourceBadge(source),
                          onTap: () => Navigator.of(context).pop(record),
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
