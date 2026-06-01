// Reusable markdown editor — paired text input + live preview, plus a
// tag chip input that resolves slugs against the server (auto-creating
// missing tags via the existing API on save). Used by lore-note edit
// screens for backstory, setting lore, campaign notes, etc.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../services/api_client.dart';

class MarkdownEditor extends StatefulWidget {
  const MarkdownEditor({
    super.key,
    required this.api,
    required this.initialTitle,
    required this.initialBody,
    required this.initialTagSlugs,
    required this.onSave,
    this.onDelete,
    this.saving = false,
    this.deleting = false,
  });

  final ApiClient api;
  final String initialTitle;
  final String initialBody;
  final List<String> initialTagSlugs;
  final void Function({
    required String title,
    required String body,
    required List<String> tagSlugs,
  }) onSave;
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
            api: widget.api,
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
                  builder: (_, __) => Markdown(
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

/// Chip-style tag input with autocomplete suggestions from the server.
/// New (previously-unknown) slugs are sent through to the server on save;
/// the server creates them lazily via `resolve_or_create_tags`.
class TagChipInput extends StatefulWidget {
  const TagChipInput({
    super.key,
    required this.api,
    required this.slugs,
    required this.onChanged,
  });

  final ApiClient api;
  final List<String> slugs;
  final ValueChanged<List<String>> onChanged;

  @override
  State<TagChipInput> createState() => _TagChipInputState();
}

class _TagChipInputState extends State<TagChipInput> {
  final _ctl = TextEditingController();

  @override
  void dispose() {
    _ctl.dispose();
    super.dispose();
  }

  void _addCurrent() {
    final raw = _ctl.text.trim().toLowerCase();
    if (raw.isEmpty) return;
    final next = List<String>.of(widget.slugs);
    if (!next.contains(raw)) next.add(raw);
    _ctl.clear();
    widget.onChanged(next);
  }

  void _remove(String slug) {
    final next = List<String>.of(widget.slugs)..remove(slug);
    widget.onChanged(next);
  }

  Future<List<String>> _suggestions(String pattern) async {
    if (pattern.isEmpty) return const [];
    try {
      final tags = await widget.api.listTags(query: pattern, limit: 8);
      return tags
          .map((t) => t.slug)
          .where((s) => !widget.slugs.contains(s))
          .toList();
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
                .map((slug) => InputChip(
                      label: Text(slug),
                      onDeleted: () => _remove(slug),
                    ))
                .toList(),
          ),
        const SizedBox(height: 4),
        Autocomplete<String>(
          optionsBuilder: (value) async => _suggestions(value.text),
          onSelected: (value) {
            _ctl.text = value;
            _addCurrent();
          },
          fieldViewBuilder: (context, controller, focusNode, onSubmitted) {
            // Keep our outer controller in sync so _addCurrent can read it.
            controller.addListener(() => _ctl.text = controller.text);
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
              onSubmitted: (_) {
                _addCurrent();
                controller.clear();
              },
            );
          },
        ),
      ],
    );
  }
}
