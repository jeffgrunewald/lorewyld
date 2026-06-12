// The Compendium — search and browse every content record installed on
// this device (the bundled SRD plus any future content modules), fully
// offline. Categories group records by type; the search field at the
// top queries all categories at once.
//
// Server-published lore modules remain reachable from here when logged
// in, via the tile at the bottom of the category list.

import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';

import '../compendium/categories.dart';
import '../services/content_store.dart';
import '../services/server_connection.dart';
import 'modules_browse_screen.dart';

class CompendiumScreen extends StatefulWidget {
  const CompendiumScreen({
    super.key,
    required this.content,
    required this.connection,
  });

  final ContentStore content;
  final ServerConnection connection;

  @override
  State<CompendiumScreen> createState() => _CompendiumScreenState();
}

class _CompendiumScreenState extends State<CompendiumScreen> {
  static const _searchLimit = 25;

  final _searchCtl = TextEditingController();
  ContentLookups _lookups = const ContentLookups();
  Map<String, int> _counts = const {};
  Map<String, List<Map<String, dynamic>>> _results = const {};
  int _searchSeq = 0;

  @override
  void initState() {
    super.initState();
    _loadStatic();
  }

  @override
  void dispose() {
    _searchCtl.dispose();
    super.dispose();
  }

  Future<void> _loadStatic() async {
    final lookups = await ContentLookups.load(widget.content);
    final counts = <String, int>{
      for (final c in compendiumCategories)
        c.table: await widget.content.count(c.table),
    };
    if (!mounted) return;
    setState(() {
      _lookups = lookups;
      _counts = counts;
    });
  }

  Future<void> _search(String query) async {
    final seq = ++_searchSeq;
    if (query.trim().isEmpty) {
      setState(() => _results = const {});
      return;
    }
    final grouped = <String, List<Map<String, dynamic>>>{};
    for (final category in compendiumCategories) {
      final rows = await widget.content
          .listNamed(category.table, query: query, limit: _searchLimit);
      if (rows.isNotEmpty) grouped[category.table] = rows;
    }
    // A newer keystroke superseded this query while it ran.
    if (!mounted || seq != _searchSeq) return;
    setState(() => _results = grouped);
  }

  @override
  Widget build(BuildContext context) {
    final searching = _searchCtl.text.trim().isNotEmpty;
    return Scaffold(
      appBar: AppBar(title: const Text('Compendium')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
            child: TextField(
              controller: _searchCtl,
              decoration: InputDecoration(
                hintText: 'Search spells, creatures, items…',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: searching
                    ? IconButton(
                        icon: const Icon(Icons.clear),
                        tooltip: 'Clear',
                        onPressed: () {
                          _searchCtl.clear();
                          _search('');
                        },
                      )
                    : null,
                border: const OutlineInputBorder(),
              ),
              onChanged: _search,
            ),
          ),
          Expanded(
            child: searching ? _searchResults() : _categoryList(),
          ),
        ],
      ),
    );
  }

  Widget _categoryList() {
    final loggedIn = widget.connection.isLoggedIn;
    return ListView(
      children: [
        for (final category in compendiumCategories)
          ListTile(
            leading: Icon(category.icon),
            title: Text(category.label),
            trailing: Text(
              '${_counts[category.table] ?? ''}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            onTap: () => Navigator.of(context).push(MaterialPageRoute(
              builder: (_) => CompendiumCategoryScreen(
                content: widget.content,
                category: category,
                lookups: _lookups,
              ),
            )),
          ),
        if (loggedIn) ...[
          const Divider(),
          ListTile(
            leading: const Icon(Icons.cloud_outlined),
            title: const Text('Server modules'),
            subtitle: const Text('Lore modules published on your server'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => Navigator.of(context).push(MaterialPageRoute(
              builder: (_) =>
                  ModulesBrowseScreen(connection: widget.connection),
            )),
          ),
        ],
      ],
    );
  }

  Widget _searchResults() {
    if (_results.isEmpty) {
      return const Center(child: Text('No matches.'));
    }
    return ListView(
      children: [
        for (final entry in _results.entries) ...[
          _GroupHeader(label: categoryFor(entry.key).label),
          for (final record in entry.value)
            _EntryTile(
              record: record,
              category: categoryFor(entry.key),
              lookups: _lookups,
            ),
        ],
      ],
    );
  }
}

class _GroupHeader extends StatelessWidget {
  const _GroupHeader({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
      child: Text(
        label,
        style: Theme.of(context).textTheme.titleSmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
      ),
    );
  }
}

class _EntryTile extends StatelessWidget {
  const _EntryTile({
    required this.record,
    required this.category,
    required this.lookups,
  });

  final Map<String, dynamic> record;
  final CompendiumCategory category;
  final ContentLookups lookups;

  @override
  Widget build(BuildContext context) {
    final subtitle = category.subtitle(record, lookups);
    return ListTile(
      title: Text(category.displayName(record)),
      subtitle:
          (subtitle == null || subtitle.isEmpty) ? null : Text(subtitle),
      onTap: () => Navigator.of(context).push(MaterialPageRoute(
        builder: (_) => CompendiumEntryScreen(
          record: record,
          category: category,
          lookups: lookups,
        ),
      )),
    );
  }
}

class CompendiumCategoryScreen extends StatefulWidget {
  const CompendiumCategoryScreen({
    super.key,
    required this.content,
    required this.category,
    required this.lookups,
  });

  final ContentStore content;
  final CompendiumCategory category;
  final ContentLookups lookups;

  @override
  State<CompendiumCategoryScreen> createState() =>
      _CompendiumCategoryScreenState();
}

class _CompendiumCategoryScreenState extends State<CompendiumCategoryScreen> {
  final _searchCtl = TextEditingController();
  late Future<List<Map<String, dynamic>>> _future;

  @override
  void initState() {
    super.initState();
    _future = widget.content.listNamed(widget.category.table);
  }

  @override
  void dispose() {
    _searchCtl.dispose();
    super.dispose();
  }

  void _search(String query) {
    setState(() {
      _future = widget.content.listNamed(widget.category.table, query: query);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(widget.category.label)),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
            child: TextField(
              controller: _searchCtl,
              decoration: InputDecoration(
                hintText: 'Filter ${widget.category.label.toLowerCase()}…',
                prefixIcon: const Icon(Icons.search),
                border: const OutlineInputBorder(),
              ),
              onChanged: _search,
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
                  itemBuilder: (_, i) => _EntryTile(
                    record: rows[i],
                    category: widget.category,
                    lookups: widget.lookups,
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class CompendiumEntryScreen extends StatelessWidget {
  const CompendiumEntryScreen({
    super.key,
    required this.record,
    required this.category,
    required this.lookups,
  });

  final Map<String, dynamic> record;
  final CompendiumCategory category;
  final ContentLookups lookups;

  @override
  Widget build(BuildContext context) {
    final facts = _facts();
    final sections = _sections();
    final description = _description();
    return Scaffold(
      appBar: AppBar(title: Text(category.displayName(record))),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (category.subtitle(record, lookups) case final String s
              when s.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Text(s, style: Theme.of(context).textTheme.titleSmall),
            ),
          if (facts.isNotEmpty) ...[
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
                              width: 120,
                              child: Text(
                                label,
                                style:
                                    Theme.of(context).textTheme.labelLarge,
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
            const SizedBox(height: 16),
          ],
          if (description != null && description.isNotEmpty) ...[
            MarkdownBody(data: description),
            const SizedBox(height: 16),
          ],
          for (final (title, entries) in sections) ...[
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            for (final (name, desc) in entries)
              Card(
                margin: const EdgeInsets.only(bottom: 8),
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Text(name,
                          style: Theme.of(context).textTheme.titleSmall),
                      const SizedBox(height: 4),
                      MarkdownBody(data: desc),
                    ],
                  ),
                ),
              ),
            const SizedBox(height: 8),
          ],
        ],
      ),
    );
  }

  String? _description() => switch (category.table) {
        'spell' => [
            record['description'] as String? ?? '',
            if (record['higher_level'] case final String h when h.isNotEmpty)
              '**At higher levels.** $h',
          ].join('\n\n'),
        _ => record['desc'] as String?,
      };

  List<(String, String)> _facts() {
    final r = record;
    switch (category.table) {
      case 'spell':
        return [
          if (r['casting_time'] case final String v)
            ('Casting time', humanizeSlug(v)),
          if (r['range_text'] case final String v) ('Range', v),
          if (r['duration'] case final String v)
            ('Duration', humanizeSlug(v)),
          ('Components', _spellComponents(r)),
          if (r['concentration'] == true) ('Concentration', 'Yes'),
          if (r['ritual'] == true) ('Ritual', 'Yes'),
        ];
      case 'creature':
        return [
          if (r['armor_class'] case final num v)
            (
              'Armor class',
              '${v.truncate()}${r['armor_detail'] is String && (r['armor_detail'] as String).isNotEmpty ? ' (${r['armor_detail']})' : ''}'
            ),
          if (r['hit_points'] case final num v)
            (
              'Hit points',
              '${v.truncate()}${r['hit_dice'] is String ? ' (${r['hit_dice']})' : ''}'
            ),
          if (_speeds(r) case final String v when v.isNotEmpty) ('Speed', v),
          if (r['ability_scores'] case final Map<String, dynamic> scores)
            ('Abilities', _abilityLine(scores)),
          if (r['experience_points'] case final num v)
            ('XP', '${v.truncate()}'),
          if (r['languages'] case final String v when v.isNotEmpty)
            ('Languages', v),
        ];
      case 'class':
        return [
          if (r['hit_dice'] != null) ('Hit die', 'd${r['hit_dice']}'),
          if (r['prof_saving_throws'] case final String v)
            ('Saving throws', v),
          if (r['prof_armor'] case final String v) ('Armor', v),
          if (r['prof_weapons'] case final String v) ('Weapons', v),
          if (r['prof_skills'] case final String v) ('Skills', v),
        ];
      case 'species':
        return [
          if (lookups.nameOf(lookups.sizes, r['size']) case final String v)
            ('Size', v),
          if (r['speed'] case final num v) ('Speed', '${v.truncate()} ft.'),
          if (r['asi_desc'] case final String v when v.isNotEmpty)
            ('Ability scores', v),
        ];
      case 'feat':
        return [
          if (r['has_prerequisite'] == true)
            ('Prerequisite', '${r['prerequisite']}'),
        ];
      case 'item':
        return [
          if (lookups.nameOf(lookups.itemCategories, r['category_uuid'])
              case final String v)
            ('Category', v),
          if (r['cost'] case final String v) ('Cost', '$v gp'),
          if (r['weight'] case final num v) ('Weight', '$v lb.'),
          if (r['is_magic'] == true && r['rarity'] is String)
            ('Rarity', humanizeSlug(r['rarity'] as String)),
          if (r['requires_attunement'] == true) ('Attunement', 'Required'),
        ];
      case 'weapon':
        return [
          ('Category', r['is_simple'] == true ? 'Simple' : 'Martial'),
          if (r['damage_dice'] case final String v)
            ('Damage', '$v ${r['damage_type'] ?? ''}'.trim()),
          if (_weaponProperties(r) case final String v when v.isNotEmpty)
            ('Properties', v),
        ];
      case 'armor':
        return [
          if (r['category'] case final String v)
            ('Category', humanizeSlug(v)),
          if (r['ac_display'] case final String v) ('Armor class', v),
          if (r['grants_stealth_disadvantage'] == true)
            ('Stealth', 'Disadvantage'),
        ];
      default:
        return const [];
    }
  }

  List<(String, List<(String, String)>)> _sections() {
    List<(String, String)> namedList(Object? value) => [
          for (final e in value as List<dynamic>? ?? const [])
            if (e case {'name': final String name, 'desc': final String desc})
              (name, desc),
        ];
    final sections = switch (category.table) {
      'species' => [('Traits', namedList(record['traits']))],
      'background' => [('Benefits', namedList(record['benefits']))],
      'feat' => [('Benefits', namedList(record['benefits']))],
      'class' => [('Features', namedList(record['features']))],
      'creature' => [('Actions', namedList(record['actions']))],
      _ => <(String, List<(String, String)>)>[],
    };
    return sections.where((s) => s.$2.isNotEmpty).toList();
  }

  String _spellComponents(Map<String, dynamic> r) {
    final parts = [
      if (r['verbal'] == true) 'V',
      if (r['somatic'] == true) 'S',
      if (r['material'] == true)
        r['material_specified'] is String &&
                (r['material_specified'] as String).isNotEmpty
            ? 'M (${r['material_specified']})'
            : 'M',
    ];
    return parts.isEmpty ? 'None' : parts.join(', ');
  }

  String _speeds(Map<String, dynamic> r) {
    final speed = r['speed'];
    if (speed is! Map<String, dynamic>) return '';
    return speed.entries
        .where((e) => e.value is num && (e.value as num) > 0)
        .map((e) => '${e.key} ${(e.value as num).truncate()} ft.')
        .join(', ');
  }

  String _abilityLine(Map<String, dynamic> scores) => [
        for (final key in const [
          'strength',
          'dexterity',
          'constitution',
          'intelligence',
          'wisdom',
          'charisma'
        ])
          if (scores[key] case final num v)
            '${key.substring(0, 3).toUpperCase()} ${v.truncate()}',
      ].join(' · ');

  String _weaponProperties(Map<String, dynamic> r) => [
        for (final p in r['properties'] as List<dynamic>? ?? const [])
          if (p case {'property_uuid': final String uuid})
            [
              lookups.weaponProperties[uuid] ?? 'Unknown',
              if (p['detail'] case final String d when d.isNotEmpty) '($d)',
            ].join(' '),
      ].join(', ');
}
