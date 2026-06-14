// Small per-record provenance chip showing the abbreviated source slug
// ('srd-2024', 'tob', 'a5e-mm', …) so records that share a name across
// sourcebooks read as distinct.

import 'package:flutter/material.dart';

class SourceBadge extends StatelessWidget {
  const SourceBadge(this.slug, {super.key});

  final String slug;

  @override
  Widget build(BuildContext context) {
    // Deliberately quiet: sources matter when planning, not during
    // play, so the label reads as background metadata.
    return Text(
      slug.toUpperCase(),
      style: Theme.of(context).textTheme.labelSmall?.copyWith(
        color: Theme.of(context).colorScheme.outline,
        fontSize: 10,
        letterSpacing: 0.5,
      ),
    );
  }
}
