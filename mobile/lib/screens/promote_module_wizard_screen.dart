// Promote-to-Module wizard.
// Step 1: module metadata (name, slug, license, version, description).
// Step 2: note selection (defaults: Visible checked, GamemasterOnly unchecked
//         with spoiler-warning banner).
// Step 3: confirm + publish — calls POST /api/modules.

import 'package:flutter/material.dart';

import '../services/api_client.dart';
import '../services/server_connection.dart';
import '../types/lore_note.dart';
import '../types/setting.dart';

class PromoteModuleWizardScreen extends StatefulWidget {
  const PromoteModuleWizardScreen({
    super.key,
    required this.connection,
    required this.setting,
  });

  final ServerConnection connection;
  final Setting setting;

  @override
  State<PromoteModuleWizardScreen> createState() =>
      _PromoteModuleWizardScreenState();
}

class _PromoteModuleWizardScreenState extends State<PromoteModuleWizardScreen> {
  int _step = 0;
  bool _loadingNotes = true;
  bool _publishing = false;
  String? _publishError;

  // metadata
  late final TextEditingController _nameCtl;
  late final TextEditingController _slugCtl;
  final _licenseCtl = TextEditingController(text: 'CC-BY 4.0');
  final _versionCtl = TextEditingController(text: '1.0.0');
  final _descCtl = TextEditingController();

  // note selection state
  List<LoreNoteWithTags> _notes = const [];
  final Map<String, bool> _selected = {};

  @override
  void initState() {
    super.initState();
    _nameCtl = TextEditingController(text: widget.setting.name);
    _slugCtl = TextEditingController(text: _slugify(widget.setting.name));
    _loadNotes();
  }

  @override
  void dispose() {
    _nameCtl.dispose();
    _slugCtl.dispose();
    _licenseCtl.dispose();
    _versionCtl.dispose();
    _descCtl.dispose();
    super.dispose();
  }

  String _slugify(String s) =>
      s.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '-').replaceAll(RegExp(r'^-+|-+$'), '');

  Future<void> _loadNotes() async {
    try {
      final notes = await widget.connection.api!.listLoreNotes(
        scopeKind: NoteScopeKind.setting,
        scopeTarget: widget.setting.uuid,
      );
      setState(() {
        _notes = notes;
        for (final n in notes) {
          // Spoiler-safe defaults: Visible checked, AuthorOnly/GamemasterOnly
          // unchecked. Users can re-check explicitly.
          _selected[n.note.uuid] = n.note.visibility == NoteVisibility.visible;
        }
        _loadingNotes = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _loadingNotes = false;
        _publishError = 'Failed to load setting notes: $e';
      });
    }
  }

  Future<void> _commit() async {
    final selectedUuids = _selected.entries
        .where((e) => e.value)
        .map((e) => e.key)
        .toList();

    setState(() {
      _publishing = true;
      _publishError = null;
    });
    final name = _nameCtl.text.trim();
    final description = _descCtl.text.trim();
    try {
      await widget.connection.api!.publishModule(
        sourceSettingUuid: widget.setting.uuid,
        name: name,
        slug: _slugCtl.text.trim().toLowerCase(),
        license: _licenseCtl.text.trim(),
        description: description.isEmpty ? null : description,
        authors: [
          widget.connection.user?.displayName ?? '',
        ].where((s) => s.isNotEmpty).toList(),
        versionString: _versionCtl.text.trim(),
        selectedNoteUuids: selectedUuids,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Published $name.')),
      );
      Navigator.of(context).pop();
    } on ApiException catch (e) {
      setState(() => _publishError = '${e.code}: ${e.message}');
    } catch (e) {
      setState(() => _publishError = 'Publish failed: $e');
    } finally {
      if (mounted) setState(() => _publishing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final selectedCount = _selected.values.where((v) => v).length;
    final gamemasterChecked = _notes
        .where((n) => n.note.visibility == NoteVisibility.gamemasterOnly)
        .where((n) => _selected[n.note.uuid] == true)
        .length;
    return Scaffold(
      appBar: AppBar(title: const Text('Promote to module')),
      body: Stepper(
        currentStep: _step,
        controlsBuilder: (context, details) {
          return Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Row(
              children: [
                if (_step < 2)
                  FilledButton(
                    onPressed: details.onStepContinue,
                    child: const Text('Next'),
                  ),
                if (_step == 2)
                  FilledButton(
                    onPressed: _publishing ? null : _commit,
                    child: _publishing
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Publish'),
                  ),
                const SizedBox(width: 8),
                if (_step > 0)
                  TextButton(
                    onPressed: details.onStepCancel,
                    child: const Text('Back'),
                  ),
              ],
            ),
          );
        },
        onStepContinue: () {
          if (_step < 2) setState(() => _step += 1);
        },
        onStepCancel: () {
          if (_step > 0) setState(() => _step -= 1);
        },
        steps: [
          Step(
            title: const Text('Metadata'),
            isActive: _step >= 0,
            content: _metadataStep(),
          ),
          Step(
            title: Text('Notes ($selectedCount selected)'),
            isActive: _step >= 1,
            content: _notesStep(gamemasterChecked),
          ),
          Step(
            title: const Text('Review & publish'),
            isActive: _step >= 2,
            content: _reviewStep(selectedCount, gamemasterChecked),
          ),
        ],
      ),
    );
  }

  Widget _metadataStep() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          controller: _nameCtl,
          decoration: const InputDecoration(labelText: 'Name'),
          onChanged: (v) {
            if (_slugCtl.text.isEmpty) _slugCtl.text = _slugify(v);
          },
        ),
        TextField(
          controller: _slugCtl,
          decoration: const InputDecoration(
            labelText: 'Slug',
            helperText: 'URL-safe identifier, lowercase + dashes',
          ),
        ),
        TextField(
          controller: _licenseCtl,
          decoration: const InputDecoration(labelText: 'License'),
        ),
        TextField(
          controller: _versionCtl,
          decoration: const InputDecoration(
            labelText: 'Version',
            helperText: 'semver, e.g. 1.0.0',
          ),
        ),
        TextField(
          controller: _descCtl,
          decoration: const InputDecoration(
            labelText: 'Description (optional)',
          ),
          minLines: 2,
          maxLines: 4,
        ),
      ],
    );
  }

  Widget _notesStep(int gamemasterChecked) {
    if (_loadingNotes) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      );
    }
    if (_notes.isEmpty) {
      return const Text(
        'This setting has no lore notes to publish yet. Cancel, add some notes, and come back.',
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (_notes.any((n) =>
            n.note.visibility != NoteVisibility.visible))
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context)
                    .colorScheme
                    .secondaryContainer
                    .withValues(alpha: 0.4),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Text(
                'Hidden (Only me / DM only) notes are unchecked to prevent '
                'accidental spoiler publication. Re-check explicitly if you '
                'do mean to publish them.',
              ),
            ),
          ),
        for (final n in _notes)
          CheckboxListTile(
            value: _selected[n.note.uuid] ?? false,
            title: Text(n.note.title),
            subtitle: Text(
              n.tags.isEmpty
                  ? _visibilityLabel(n.note.visibility)
                  : '${_visibilityLabel(n.note.visibility)} · ${n.tags.map((t) => t.slug).join(' · ')}',
            ),
            controlAffinity: ListTileControlAffinity.leading,
            onChanged: (v) =>
                setState(() => _selected[n.note.uuid] = v ?? false),
          ),
      ],
    );
  }

  Widget _reviewStep(int selectedCount, int gamemasterChecked) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (_publishError != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Text(
              _publishError!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
        Text('Module: ${_nameCtl.text.trim()} v${_versionCtl.text.trim()}'),
        Text('Slug: ${_slugCtl.text.trim()}'),
        Text('License: ${_licenseCtl.text.trim()}'),
        Text('Notes selected: $selectedCount'),
        if (gamemasterChecked > 0)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.errorContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                'Heads up: $gamemasterChecked DM-only note(s) are included. '
                'These will be visible to everyone who imports the module.',
              ),
            ),
          ),
      ],
    );
  }

  String _visibilityLabel(NoteVisibility v) => switch (v) {
        NoteVisibility.visible => 'Visible',
        NoteVisibility.authorOnly => 'Only me',
        NoteVisibility.gamemasterOnly => 'DM only',
      };
}
