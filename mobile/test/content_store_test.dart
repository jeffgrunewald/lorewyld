// First-launch SRD bundle import against an in-memory SQLite database,
// using the real bundled asset.

import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:lorewyld/services/content_store.dart';
import 'package:lorewyld/services/local_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  sqfliteFfiInit();
  databaseFactory = databaseFactoryFfi;

  late LocalStore store;
  late ContentStore content;

  setUp(() async {
    store = await LocalStore.open(path: inMemoryDatabasePath);
    content = ContentStore(store);
  });

  tearDown(() => store.close());

  test('imports the bundled SRD content once', () async {
    expect(await content.isSeeded, isFalse);

    final progress = <double>[];
    await content.importBundle(onProgress: progress.add);

    expect(await content.isSeeded, isTrue);
    expect(progress.last, 1.0);

    final spells = await content.listSpells(level: 3);
    expect(spells, isNotEmpty);
    final fireball = await content.getByKey('spell', 'srd-2024_fireball');
    expect(fireball?['name'], 'Fireball');
    expect(fireball?['damage_roll'], '8d6');

    final classes = await content.listClasses(basesOnly: true);
    expect(classes.length, 12);
    final species = await content.listSpecies();
    expect(species, isNotEmpty);

    // Second call is a no-op, not a constraint violation.
    await content.importBundle();
  });

  test('compendium reads: name search, counts, lookups', () async {
    await content.importBundle();

    // LIKE search matches substrings case-insensitively.
    final fire = await content.listNamed('spell', query: 'fire');
    expect(fire.map((s) => s['name']), contains('Fireball'));

    // Extra where-clause composes with the name query.
    final bases = await content.listNamed(
      'class',
      query: 'a',
      where: 'subclass_of IS NULL',
    );
    expect(bases.map((c) => c['name']), contains('Barbarian'));
    expect(bases.every((c) => c['subclass_of'] == null), isTrue);

    expect(await content.count('background'), greaterThan(0));
    expect(await content.count('alignment'), 9);

    // Raw lookup reads stay wire-true; display humanizing happens in
    // ContentLookups.load.
    final schools = await content.lookupNames('spell_school');
    expect(schools.values, contains('evocation'));

    final alignments = await content.listAlignments();
    expect(alignments.map((a) => a['name']), contains('lawful_good'));
    expect((await content.listBackgrounds()).map((b) => b['name']),
        contains('Acolyte'));
  });
}
