// Shared loading / error / empty / list scaffolding around a
// FutureBuilder. Error and empty states render inside a ListView so an
// enclosing RefreshIndicator keeps working (pull-to-refresh needs a
// scrollable even when there's nothing to scroll).

import 'package:flutter/material.dart';

class AsyncListView<T> extends StatelessWidget {
  const AsyncListView({
    super.key,
    required this.future,
    required this.emptyMessage,
    required this.itemBuilder,
  });

  final Future<List<T>> future;
  final String emptyMessage;
  final Widget Function(BuildContext context, T item) itemBuilder;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<T>>(
      future: future,
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
        final items = snap.data ?? const [];
        if (items.isEmpty) {
          return ListView(
            children: [
              const SizedBox(height: 80),
              Center(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 24),
                  child: Text(emptyMessage, textAlign: TextAlign.center),
                ),
              ),
            ],
          );
        }
        return ListView.separated(
          itemCount: items.length,
          separatorBuilder: (_, _) => const Divider(height: 1),
          itemBuilder: (context, i) => itemBuilder(context, items[i]),
        );
      },
    );
  }
}
