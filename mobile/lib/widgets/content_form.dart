// Schema-driven homebrew authoring form. Every control is derived from
// the category's FieldSchema — single-sourced in lorewyld-domain and
// reached here over FFI (fieldSchema / defaultRecord / validateRecord),
// the same rules the web form and the server enforce. No per-type form
// code: a new content type is a schema change, not a UI change.
//
// Mobile is local-first, so a save writes straight to the on-device store
// under the Homebrew (or a chosen custom) module; server sync is separate.

import 'dart:convert';

import 'package:flutter/material.dart';

import '../ffi/api/authoring.dart' as authoring;
import '../services/content_store.dart';
import '../util/uuid.dart';

class ContentFormScreen extends StatefulWidget {
  const ContentFormScreen({
    super.key,
    required this.content,
    required this.category,
    this.existing,
  });

  final ContentStore content;
  final String category;

  /// The record being edited, or null to create a new one.
  final Map<String, dynamic>? existing;

  bool get isEdit => existing != null;

  @override
  State<ContentFormScreen> createState() => _ContentFormScreenState();
}

class _ContentFormScreenState extends State<ContentFormScreen> {
  List<Map<String, dynamic>> _fields = const [];
  final Map<String, dynamic> _values = {};
  final Map<String, TextEditingController> _controllers = {};
  final Map<String, Map<String, String>> _lookups = {};
  final Map<String, String> _errors = {};
  List<Map<String, dynamic>> _localModules = const [];
  String? _moduleUuid; // create: selected target (null = Homebrew default)
  bool _loading = true;
  bool _saving = false;
  String? _generalError;

  String get _title =>
      '${widget.isEdit ? 'Edit' : 'New'} ${_humanize(widget.category)}';

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    for (final c in _controllers.values) {
      c.dispose();
    }
    super.dispose();
  }

  Future<void> _load() async {
    final schemaJson = authoring.fieldSchema(category: widget.category);
    final schema = jsonDecode(schemaJson) as Map<String, dynamic>?;
    if (schema == null) {
      setState(() {
        _loading = false;
        _generalError = '"${widget.category}" is not authorable.';
      });
      return;
    }
    _fields = (schema['fields'] as List<dynamic>).cast<Map<String, dynamic>>();

    // Seed each field's initial value (edit: from the record; create: from
    // the skeleton) and a text controller where the control needs one.
    for (final field in _fields) {
      final key = field['key'] as String;
      final kind = field['kind'] as String;
      final initial = widget.existing?[key];
      _values[key] = initial;
      if (_needsController(kind)) {
        _controllers[key] = TextEditingController(
          text: _controllerText(kind, initial),
        );
      }
      if (kind == 'select_lookup') {
        final table = field['table'] as String;
        _lookups[table] = await widget.content.lookupNames(table);
      }
    }

    _localModules = await widget.content.localModules();
    _moduleUuid = widget.existing?['content_module_uuid'] as String?;
    if (!mounted) return;
    setState(() => _loading = false);
  }

  static bool _needsController(String kind) => switch (kind) {
    'text' || 'markdown' || 'int' || 'float' || 'json' => true,
    _ => false,
  };

  static String _controllerText(String kind, dynamic value) {
    if (value == null) return '';
    if (kind == 'json') return const JsonEncoder.withIndent('  ').convert(value);
    return '$value';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(_title)),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _generalError != null && _fields.isEmpty
          ? Center(child: Text(_generalError!))
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                if (!widget.isEdit) _modulePicker(),
                for (final field in _fields) _buildField(field),
                if (_generalError != null) ...[
                  const SizedBox(height: 8),
                  Text(
                    _generalError!,
                    style: TextStyle(color: Theme.of(context).colorScheme.error),
                  ),
                ],
                const SizedBox(height: 16),
                FilledButton(
                  onPressed: _saving ? null : _save,
                  child: Text(widget.isEdit ? 'Save' : 'Create'),
                ),
              ],
            ),
    );
  }

  Widget _modulePicker() {
    return Padding(
      padding: const EdgeInsets.only(bottom: 14),
      child: DropdownButtonFormField<String?>(
        initialValue: _moduleUuid,
        decoration: const InputDecoration(
          labelText: 'Save to module',
          border: OutlineInputBorder(),
        ),
        items: [
          const DropdownMenuItem(value: null, child: Text('Homebrew (default)')),
          for (final m in _localModules)
            if (m['slug'] != ContentStore.homebrewModuleSlug)
              DropdownMenuItem(
                value: m['uuid'] as String,
                child: Text('${m['name']}'),
              ),
        ],
        onChanged: (v) => setState(() => _moduleUuid = v),
      ),
    );
  }

  Widget _buildField(Map<String, dynamic> field) {
    final key = field['key'] as String;
    final kind = field['kind'] as String;
    final label = field['label'] as String;
    final required = field['required'] == true;
    final help = field['help'] as String?;
    final error = _errors[key];

    Widget control;
    switch (kind) {
      case 'bool':
        control = SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(label),
          value: _values[key] == true,
          onChanged: (v) => setState(() => _values[key] = v),
        );
        break;
      case 'select_enum':
        control = _enumDropdown(
          field,
          label,
          required,
          (field['options'] as List<dynamic>).cast<Map<String, dynamic>>(),
        );
        break;
      case 'select_lookup':
        final table = field['table'] as String;
        final entries = (_lookups[table] ?? const {}).entries.toList()
          ..sort((a, b) => a.value.compareTo(b.value));
        control = _lookupDropdown(
          field,
          label,
          required,
          [for (final e in entries) {'value': e.key, 'label': e.value}],
        );
        break;
      case 'enum_list':
        control = _enumChips(
          field,
          label,
          (field['options'] as List<dynamic>).cast<Map<String, dynamic>>(),
        );
        break;
      default:
        control = TextField(
          controller: _controllers[key],
          decoration: InputDecoration(
            labelText: label + (required ? ' *' : ''),
            border: const OutlineInputBorder(),
          ),
          keyboardType: switch (kind) {
            'int' => TextInputType.number,
            'float' => const TextInputType.numberWithOptions(decimal: true),
            'markdown' || 'json' => TextInputType.multiline,
            _ => TextInputType.text,
          },
          maxLines: switch (kind) {
            'markdown' => 6,
            'json' => 5,
            _ => 1,
          },
          style: kind == 'json'
              ? const TextStyle(fontFamily: 'monospace', fontSize: 13)
              : null,
        );
    }

    return Padding(
      padding: const EdgeInsets.only(bottom: 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          control,
          if (help != null)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(help, style: Theme.of(context).textTheme.bodySmall),
            ),
          if (error != null)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                error,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ),
        ],
      ),
    );
  }

  Widget _enumDropdown(
    Map<String, dynamic> field,
    String label,
    bool required,
    List<Map<String, dynamic>> options,
  ) {
    final key = field['key'] as String;
    return DropdownButtonFormField<String?>(
      initialValue: _values[key] as String?,
      decoration: InputDecoration(
        labelText: label + (required ? ' *' : ''),
        border: const OutlineInputBorder(),
      ),
      items: [
        if (!required)
          const DropdownMenuItem(value: null, child: Text('— none —')),
        for (final o in options)
          DropdownMenuItem(
            value: o['value'] as String,
            child: Text('${o['label']}'),
          ),
      ],
      onChanged: (v) => setState(() => _values[key] = v),
    );
  }

  Widget _lookupDropdown(
    Map<String, dynamic> field,
    String label,
    bool required,
    List<Map<String, dynamic>> options,
  ) {
    final key = field['key'] as String;
    return DropdownButtonFormField<String?>(
      initialValue: _values[key] as String?,
      isExpanded: true,
      decoration: InputDecoration(
        labelText: label + (required ? ' *' : ''),
        border: const OutlineInputBorder(),
      ),
      items: [
        DropdownMenuItem(
          value: null,
          child: Text(required ? '— select —' : '— none —'),
        ),
        for (final o in options)
          DropdownMenuItem(
            value: o['value'] as String,
            child: Text('${o['label']}', overflow: TextOverflow.ellipsis),
          ),
      ],
      onChanged: (v) => setState(() => _values[key] = v),
    );
  }

  Widget _enumChips(
    Map<String, dynamic> field,
    String label,
    List<Map<String, dynamic>> options,
  ) {
    final key = field['key'] as String;
    final selected = ((_values[key] as List<dynamic>?) ?? const [])
        .cast<String>()
        .toSet();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelLarge),
        const SizedBox(height: 6),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: [
            for (final o in options)
              FilterChip(
                label: Text('${o['label']}'),
                selected: selected.contains(o['value']),
                onSelected: (on) => setState(() {
                  final set = selected;
                  if (on) {
                    set.add(o['value'] as String);
                  } else {
                    set.remove(o['value']);
                  }
                  _values[key] = set.toList();
                }),
              ),
          ],
        ),
      ],
    );
  }

  /// Reads the live controls into a field→value input map. Returns null
  /// and populates [_errors] if a JSON field doesn't parse.
  Map<String, dynamic>? _collect() {
    final input = <String, dynamic>{};
    var bad = false;
    for (final field in _fields) {
      final key = field['key'] as String;
      final kind = field['kind'] as String;
      switch (kind) {
        case 'text':
        case 'markdown':
          input[key] = _controllers[key]!.text;
          break;
        case 'int':
          final t = _controllers[key]!.text.trim();
          input[key] = t.isEmpty ? null : (int.tryParse(t) ?? t);
          break;
        case 'float':
          final t = _controllers[key]!.text.trim();
          input[key] = t.isEmpty ? null : (double.tryParse(t) ?? t);
          break;
        case 'json':
          final t = _controllers[key]!.text.trim();
          if (t.isEmpty) {
            if (field['required'] == true) input[key] = null;
          } else {
            try {
              input[key] = jsonDecode(t);
            } catch (e) {
              _errors[key] = 'Invalid JSON: $e';
              bad = true;
            }
          }
          break;
        case 'bool':
          input[key] = _values[key] == true;
          break;
        case 'select_enum':
        case 'select_lookup':
          input[key] = _values[key]; // String? (null when unselected)
          break;
        case 'enum_list':
          input[key] = ((_values[key] as List<dynamic>?) ?? const []).toList();
          break;
      }
    }
    return bad ? null : input;
  }

  Future<void> _save() async {
    setState(() {
      _errors.clear();
      _generalError = null;
    });
    final input = _collect();
    if (input == null) {
      setState(() {});
      return;
    }
    // Pre-submit validation via the same lorewyld-domain rules.
    final errs =
        jsonDecode(
              authoring.validateRecord(
                category: widget.category,
                inputJson: jsonEncode(input),
              ),
            )
            as List<dynamic>;
    if (errs.isNotEmpty) {
      setState(() {
        for (final e in errs) {
          final m = e as Map<String, dynamic>;
          _errors[m['field'] as String? ?? ''] = m['message'] as String? ?? '';
        }
      });
      return;
    }

    setState(() => _saving = true);
    try {
      final (moduleUuid, documentUuid) = await _resolveModule();
      final record = await _assemble(input, moduleUuid, documentUuid);
      await widget.content.upsertRecord(widget.category, record);
      if (mounted) Navigator.of(context).pop(true);
    } catch (e) {
      if (mounted) {
        setState(() {
          _saving = false;
          _generalError = 'Save failed: $e';
        });
      }
    }
  }

  Future<(String, String)> _resolveModule() async {
    if (widget.isEdit) {
      final moduleUuid = widget.existing!['content_module_uuid'] as String;
      final doc =
          widget.existing!['document_uuid'] as String? ??
          await widget.content.ensureModuleDocument(moduleUuid, 'Homebrew');
      return (moduleUuid, doc);
    }
    if (_moduleUuid == null) {
      final hb = await widget.content.ensureHomebrewModule();
      return (hb.moduleUuid, hb.documentUuid);
    }
    final name = _localModules.firstWhere(
      (m) => m['uuid'] == _moduleUuid,
      orElse: () => const {'name': 'Homebrew'},
    )['name'];
    final doc = await widget.content.ensureModuleDocument(
      _moduleUuid!,
      '$name',
    );
    return (_moduleUuid!, doc);
  }

  Future<Map<String, dynamic>> _assemble(
    Map<String, dynamic> input,
    String moduleUuid,
    String documentUuid,
  ) async {
    final skeleton =
        jsonDecode(authoring.defaultRecord(category: widget.category))
            as Map<String, dynamic>;
    skeleton.addAll(input);

    final uuid = widget.existing?['uuid'] as String? ?? generateUuidV4();
    final now = DateTime.now().toUtc().toIso8601String();
    skeleton['uuid'] = uuid;
    skeleton['content_module_uuid'] = moduleUuid;
    if (skeleton.containsKey('document_uuid')) {
      skeleton['document_uuid'] = documentUuid;
    }
    skeleton['key'] =
        widget.existing?['key'] as String? ?? '${widget.category}-$uuid';
    skeleton['slug'] = _slugify('${skeleton['name'] ?? ''}', uuid);
    skeleton['created_at'] = widget.existing?['created_at'] as String? ?? now;
    skeleton['updated_at'] = now;
    return skeleton;
  }

  static String _slugify(String name, String fallback) {
    final slug = name
        .toLowerCase()
        .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return slug.isEmpty ? fallback : slug;
  }

  static String _humanize(String s) => s
      .split(RegExp(r'[_-]'))
      .where((w) => w.isNotEmpty)
      .map((w) => '${w[0].toUpperCase()}${w.substring(1)}')
      .join(' ');
}
