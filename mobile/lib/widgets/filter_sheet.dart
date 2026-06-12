// Shared filter & sort bottom sheet for filterable content lists —
// used by the character-builder pickers and the Compendium category
// screens. Mutates the caller's FilterState in place and reports each
// change via [onChanged] so the list behind the sheet updates live.

import 'package:flutter/material.dart';

import '../compendium/categories.dart';
import '../compendium/filters.dart';

Future<void> showContentFilterSheet({
  required BuildContext context,
  required List<FilterDimension> dimensions,
  required List<PickerSort> sorts,
  required ContentLookups lookups,
  required List<Map<String, dynamic>> records,
  required FilterState state,
  required VoidCallback onChanged,
}) {
  return showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    builder: (_) => StatefulBuilder(
      builder: (context, setSheetState) {
        void both(VoidCallback fn) {
          setSheetState(fn);
          onChanged();
        }

        return SizedBox(
          height: MediaQuery.of(context).size.height * 0.7,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 16, 8, 0),
                child: Row(
                  children: [
                    Expanded(
                      child: Text('Filter & sort',
                          style: Theme.of(context).textTheme.titleMedium),
                    ),
                    TextButton(
                      onPressed: state.activeCount == 0 &&
                              state.sort == sorts.first
                          ? null
                          : () => both(() => state.reset(sorts.first)),
                      child: const Text('Reset'),
                    ),
                  ],
                ),
              ),
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                  children: [
                    if (sorts.length > 1) ...[
                      const _FilterSectionLabel('Sort by'),
                      Wrap(
                        spacing: 8,
                        runSpacing: 4,
                        children: [
                          for (final s in sorts)
                            ChoiceChip(
                              label: Text(s.label),
                              selected: state.sort.key == s.key,
                              onSelected: (_) =>
                                  both(() => state.sort = s),
                            ),
                        ],
                      ),
                    ],
                    for (final dim in dimensions) ...[
                      _FilterSectionLabel(dim.label),
                      Wrap(
                        spacing: 8,
                        runSpacing: 4,
                        children: [
                          for (final option in dim.options(records, lookups))
                            FilterChip(
                              label: Text(option.label),
                              selected: state.selections[dim.key]
                                      ?.contains(option.value) ??
                                  false,
                              onSelected: (selected) => both(() {
                                final set = state.selections
                                    .putIfAbsent(dim.key, () => {});
                                selected
                                    ? set.add(option.value)
                                    : set.remove(option.value);
                              }),
                            ),
                        ],
                      ),
                    ],
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(16),
                child: FilledButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('Done'),
                ),
              ),
            ],
          ),
        );
      },
    ),
  );
}

class _FilterSectionLabel extends StatelessWidget {
  const _FilterSectionLabel(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 16, bottom: 8),
      child: Text(text, style: Theme.of(context).textTheme.titleSmall),
    );
  }
}
