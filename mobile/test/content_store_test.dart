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
}
