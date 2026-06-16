// Manage the content modules shipped with the app: see what's
// installed, uninstall sourcebooks to reclaim space, and reinstall
// anything previously removed (the bundle always ships everything).
//
// The list is driven by the bundle manifest, not the database, so
// uninstalled modules remain visible — greyed out, with their removal
// date — and can be reinstalled.

import 'package:flutter/material.dart';

import '../services/content_store.dart';
import '../services/local_store.dart';
import '../types/bundled_module.dart';
import '../types/content_module.dart' show licenseDisplayName;

class ModuleManagementScreen extends StatefulWidget {
  const ModuleManagementScreen({super.key, required this.store});

  final LocalStore store;

  @override
  State<ModuleManagementScreen> createState() => _ModuleManagementScreenState();
}

class _ModuleManagementScreenState extends State<ModuleManagementScreen> {
  late final ContentStore _content = ContentStore(widget.store);

  List<BundledModule> _modules = const [];
  Set<String> _installed = const {};
  Map<String, String> _removed = const {};
  bool _loaded = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final modules = await _content.bundledModules();
    final installed = await _content.installedModuleSlugs();
    final removed = await _content.removedModules();
    if (!mounted) return;
    setState(() {
      _modules = modules;
      _installed = installed;
      _removed = removed;
      _loaded = true;
    });
  }

  Future<void> _openDetail(BundledModule module) async {
    final changed = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => ModuleInfoScreen(
          content: _content,
          module: module,
          installed: _installed.contains(module.slug),
          removedAt: _removed[module.slug],
        ),
      ),
    );
    if (changed == true) await _load();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Content modules')),
      body: !_loaded
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              children: [
                for (final module in _modules)
                  _ModuleTile(
                    module: module,
                    pinned: module.slug == ContentStore.pinnedModuleSlug,
                    installed: _installed.contains(module.slug),
                    removedAt: _removed[module.slug],
                    onTap: () => _openDetail(module),
                  ),
              ],
            ),
    );
  }
}

class _ModuleTile extends StatelessWidget {
  const _ModuleTile({
    required this.module,
    required this.pinned,
    required this.installed,
    required this.removedAt,
    required this.onTap,
  });

  final BundledModule module;
  final bool pinned;
  final bool installed;
  final String? removedAt;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final removed = removedAt != null;
    final subtitle = [
      licenseDisplayName(module.license),
      ?module.publisher,
      if (removed) 'Removed ${removedAt!.split('T').first}',
    ].join(' · ');
    final tile = ListTile(
      leading: Icon(
        removed ? Icons.cancel_outlined : Icons.inventory_2_outlined,
        color: removed ? theme.colorScheme.outline : null,
      ),
      title: Text(module.name),
      subtitle: Text(subtitle),
      trailing: pinned
          ? Text('Required', style: theme.textTheme.labelSmall)
          : removed
          ? Text(
              'Removed',
              style: theme.textTheme.labelSmall?.copyWith(
                color: theme.colorScheme.error,
              ),
            )
          : Icon(
              Icons.check_circle_outline,
              size: 18,
              color: theme.colorScheme.primary,
            ),
      onTap: onTap,
    );
    // Uninstalled modules read as absent but stay tappable.
    return removed ? Opacity(opacity: 0.55, child: tile) : tile;
  }
}

class ModuleInfoScreen extends StatefulWidget {
  const ModuleInfoScreen({
    super.key,
    required this.content,
    required this.module,
    required this.installed,
    required this.removedAt,
  });

  final ContentStore content;
  final BundledModule module;
  final bool installed;
  final String? removedAt;

  @override
  State<ModuleInfoScreen> createState() => _ModuleInfoScreenState();
}

class _ModuleInfoScreenState extends State<ModuleInfoScreen> {
  bool _working = false;
  double _progress = 0;

  bool get _pinned => widget.module.slug == ContentStore.pinnedModuleSlug;

  Future<void> _uninstall() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Uninstall ${widget.module.name}?'),
        content: Text(
          'Its ${widget.module.totalRecords} records will be removed from '
          'this device. You can reinstall it from this screen at any '
          'time — the content ships with the app.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton.tonal(
            onPressed: () => Navigator.pop(ctx, true),
            style: FilledButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Uninstall'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    setState(() => _working = true);
    await widget.content.uninstallModule(widget.module.slug);
    if (!mounted) return;
    Navigator.of(context).pop(true);
  }

  Future<void> _reinstall() async {
    setState(() {
      _working = true;
      _progress = 0;
    });
    await widget.content.reinstallModule(
      widget.module.slug,
      onProgress: (p) {
        if (mounted) setState(() => _progress = p);
      },
    );
    if (!mounted) return;
    Navigator.of(context).pop(true);
  }

  @override
  Widget build(BuildContext context) {
    final module = widget.module;
    final theme = Theme.of(context);
    final facts = <(String, String)>[
      ('License', licenseDisplayName(module.license)),
      if (module.publisher case final String p) ('Publisher', p),
      if (module.authors.isNotEmpty) ('Authors', module.authors.join(', ')),
      if (module.documents.isNotEmpty) ('Sources', module.documents.join('\n')),
      if (module.websiteUrl case final String w) ('Website', w),
      if (module.licenseUrl case final String l) ('License text', l),
    ];
    return Scaffold(
      appBar: AppBar(title: Text(module.name)),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (module.description case final String desc) ...[
            Text(desc, style: theme.textTheme.bodyLarge),
            const SizedBox(height: 16),
          ],
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (final (label, value) in facts)
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 2),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          SizedBox(
                            width: 110,
                            child: Text(
                              label,
                              style: theme.textTheme.labelLarge,
                            ),
                          ),
                          Expanded(child: Text(value)),
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
          if (module.recordCounts.isNotEmpty) ...[
            const SizedBox(height: 16),
            Text('Contents', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                for (final entry in module.recordCounts.entries)
                  Chip(
                    label: Text('${entry.value} ${entry.key}'),
                    visualDensity: VisualDensity.compact,
                  ),
              ],
            ),
          ],
          const SizedBox(height: 24),
          if (_pinned)
            Text(
              'Required module — it provides the shared rules vocabulary '
              '(schools, sizes, conditions, …) that every other module '
              'references, and cannot be removed.',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.outline,
              ),
            )
          else if (_working && widget.removedAt != null) ...[
            Text('Reinstalling…', style: theme.textTheme.bodySmall),
            const SizedBox(height: 8),
            LinearProgressIndicator(value: _progress),
          ] else if (widget.removedAt != null)
            FilledButton.tonalIcon(
              onPressed: _reinstall,
              icon: const Icon(Icons.download_outlined),
              label: const Text('Reinstall module'),
            )
          else if (widget.installed)
            FilledButton.tonalIcon(
              onPressed: _working ? null : _uninstall,
              style: FilledButton.styleFrom(
                foregroundColor: theme.colorScheme.error,
              ),
              icon: const Icon(Icons.delete_outline),
              label: const Text('Uninstall module'),
            ),
          const SizedBox(height: 32),
        ],
      ),
    );
  }
}
