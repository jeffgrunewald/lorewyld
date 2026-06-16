// Reusable markdown editor — paired text input + live preview, plus a
// tag chip input. Tag autocomplete is supplied by the caller as a
// callback so the editor works against the local store offline and the
// server API alike.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';

/// Returns slug suggestions for a partial input.
typedef TagSuggestionsProvider = Future<List<String>> Function(String pattern);

class MarkdownEditor extends StatefulWidget {
  const MarkdownEditor({
    super.key,
    required this.tagSuggestions,
    required this.initialTitle,
    required this.initialBody,
    required this.initialTagSlugs,
    required this.onSave,
    this.onDelete,
    this.saving = false,
    this.deleting = false,
  });

  final TagSuggestionsProvider tagSuggestions;
  final String initialTitle;
  final String initialBody;
  final List<String> initialTagSlugs;
  final void Function({
    required String title,
    required String body,
    required List<String> tagSlugs,
  })
  onSave;
  final VoidCallback? onDelete;
  final bool saving;
  final bool deleting;

  @override
  State<MarkdownEditor> createState() => _MarkdownEditorState();
}

class _MarkdownEditorState extends State<MarkdownEditor>
    with SingleTickerProviderStateMixin {
  late final TextEditingController _titleCtl;
  late final TextEditingController _bodyCtl;
  late List<String> _tagSlugs;
  late final TabController _tabs;

  @override
  void initState() {
    super.initState();
    _titleCtl = TextEditingController(text: widget.initialTitle);
    _bodyCtl = TextEditingController(text: widget.initialBody);
    _tagSlugs = List.of(widget.initialTagSlugs);
    _tabs = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _titleCtl.dispose();
    _bodyCtl.dispose();
    _tabs.dispose();
    super.dispose();
  }

  void _commit() {
    widget.onSave(
      title: _titleCtl.text.trim(),
      body: _bodyCtl.text,
      tagSlugs: List.of(_tagSlugs),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
          child: TextField(
            controller: _titleCtl,
            decoration: const InputDecoration(
              labelText: 'Title',
              border: OutlineInputBorder(),
            ),
            textInputAction: TextInputAction.next,
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: TagChipInput(
            suggestions: widget.tagSuggestions,
            slugs: _tagSlugs,
            onChanged: (slugs) => setState(() => _tagSlugs = slugs),
          ),
        ),
        TabBar(
          controller: _tabs,
          tabs: const [
            Tab(text: 'Edit'),
            Tab(text: 'Preview'),
          ],
        ),
        Expanded(
          child: TabBarView(
            controller: _tabs,
            children: [
              Padding(
                padding: const EdgeInsets.all(16),
                child: TextField(
                  controller: _bodyCtl,
                  maxLines: null,
                  expands: true,
                  textAlignVertical: TextAlignVertical.top,
                  decoration: const InputDecoration(
                    hintText: 'Write markdown here…',
                    border: OutlineInputBorder(),
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(16),
                child: ListenableBuilder(
                  listenable: _bodyCtl,
                  builder: (_, _) => Markdown(
                    data: _bodyCtl.text,
                    padding: EdgeInsets.zero,
                    shrinkWrap: false,
                  ),
                ),
              ),
            ],
          ),
        ),
        SafeArea(
          top: false,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: Row(
              children: [
                if (widget.onDelete != null)
                  TextButton.icon(
                    onPressed: widget.deleting ? null : widget.onDelete,
                    icon: const Icon(Icons.delete_outline),
                    label: const Text('Delete'),
                  ),
                const Spacer(),
                FilledButton.icon(
                  onPressed: widget.saving ? null : _commit,
                  icon: widget.saving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.save),
                  label: const Text('Save'),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// Chip-style tag input with caller-supplied autocomplete. Freeform
/// slugs are allowed; unknown ones are created lazily wherever the note
/// ends up (local store, or server-side on push).
class TagChipInput extends StatefulWidget {
  const TagChipInput({
    super.key,
    required this.suggestions,
    required this.slugs,
    required this.onChanged,
  });

  final TagSuggestionsProvider suggestions;
  final List<String> slugs;
  final ValueChanged<List<String>> onChanged;

  @override
  State<TagChipInput> createState() => _TagChipInputState();
}

class _TagChipInputState extends State<TagChipInput> {
  // The Autocomplete widget owns (and disposes) this controller; we keep
  // a reference so selection and submission share one add-slug path.
  TextEditingController? _fieldCtl;

  void _add(String raw) {
    final slug = raw.trim().toLowerCase();
    if (slug.isEmpty) return;
    final next = List<String>.of(widget.slugs);
    if (!next.contains(slug)) next.add(slug);
    _fieldCtl?.clear();
    widget.onChanged(next);
  }

  void _remove(String slug) {
    final next = List<String>.of(widget.slugs)..remove(slug);
    widget.onChanged(next);
  }

  Future<List<String>> _suggestions(String pattern) async {
    if (pattern.isEmpty) return const [];
    try {
      final slugs = await widget.suggestions(pattern);
      return slugs.where((s) => !widget.slugs.contains(s)).toList();
    } catch (_) {
      return const [];
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (widget.slugs.isNotEmpty)
          Wrap(
            spacing: 6,
            runSpacing: 4,
            children: widget.slugs
                .map(
                  (slug) => InputChip(
                    label: Text(slug),
                    onDeleted: () => _remove(slug),
                  ),
                )
                .toList(),
          ),
        const SizedBox(height: 4),
        Autocomplete<String>(
          optionsBuilder: (value) async => _suggestions(value.text),
          onSelected: _add,
          fieldViewBuilder: (context, controller, focusNode, onSubmitted) {
            _fieldCtl = controller;
            return TextField(
              controller: controller,
              focusNode: focusNode,
              decoration: const InputDecoration(
                labelText: 'Add tag',
                hintText: 'e.g. npc, location, fey-realm',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              inputFormatters: [
                FilteringTextInputFormatter.allow(RegExp(r'[a-z0-9\- ]')),
              ],
              onSubmitted: (text) => _add(text),
            );
          },
        ),
      ],
    );
  }
}
