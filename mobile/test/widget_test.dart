// Smoke tests for the local-first mobile shell: the app boots straight
// into the home screen with no server connection, and local features
// are reachable offline.
//
// LocalStore does real isolate IO (sqflite ffi), which never completes
// inside testWidgets' FakeAsync zone — wrap open/close in
// tester.runAsync.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:lorewyld/main.dart';
import 'package:lorewyld/screens/character_list_screen.dart';
import 'package:lorewyld/services/content_store.dart';
import 'package:lorewyld/services/local_store.dart';
import 'package:lorewyld/services/server_connection.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  sqfliteFfiInit();
  databaseFactory = databaseFactoryFfi;

  /// Opens a store with SRD content already imported, so the app's
  /// first-launch seed gate passes straight through (its real-zone
  /// import IO can't run inside testWidgets' FakeAsync zone).
  Future<LocalStore> openSeededStore(WidgetTester tester) async {
    return (await tester.runAsync(() async {
      final store = await LocalStore.open(path: inMemoryDatabasePath);
      await ContentStore(store).importBundle();
      return store;
    }))!;
  }

  Future<void> pumpPastSeedGate(WidgetTester tester) async {
    // The gate's importBundle early-returns on its seeded check, but
    // that query resolves in the real zone — give it a beat, then pump.
    await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)));
    await tester.pump();
  }

  testWidgets('app boots offline and shows the lorewyld brand',
      (tester) async {
    final connection = ServerConnection();
    final store = await openSeededStore(tester);
    await tester.pumpWidget(LorewyldApp(connection: connection, store: store));
    await pumpPastSeedGate(tester);
    await tester.pumpAndSettle(const Duration(milliseconds: 100));
    expect(find.bySemanticsLabel('Lorewyld'), findsWidgets);
    await tester.runAsync(store.close);
  });

  testWidgets('home screen offers local features without a server',
      (tester) async {
    final connection = ServerConnection();
    final store = await openSeededStore(tester);
    await tester.pumpWidget(LorewyldApp(connection: connection, store: store));
    await pumpPastSeedGate(tester);
    await tester.pumpAndSettle(const Duration(milliseconds: 100));
    expect(find.text('Characters'), findsOneWidget);
    expect(find.text('Settings & lore'), findsOneWidget);
    expect(find.text('Search'), findsOneWidget);
    expect(find.text('Working locally — no server connection'), findsOneWidget);
    // The Compendium is fully local — available without a login.
    expect(find.text('Compendium'), findsOneWidget);
    await tester.runAsync(store.close);
  });

  testWidgets('compendium lists content categories offline', (tester) async {
    final connection = ServerConnection();
    final store = await openSeededStore(tester);
    await tester.pumpWidget(LorewyldApp(connection: connection, store: store));
    await pumpPastSeedGate(tester);
    await tester.pumpAndSettle(const Duration(milliseconds: 100));

    await tester.tap(find.text('Compendium'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    // Category counts/lookups resolve in the real zone.
    await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)));
    await tester.pump();

    expect(find.text('Spells'), findsOneWidget);
    expect(find.text('Creatures'), findsOneWidget);
    expect(find.text('Species'), findsOneWidget);
    expect(find.text('Backgrounds'), findsOneWidget);
    await tester.runAsync(store.close);
  });

  testWidgets('creating a character opens its sheet (regression: setState '
      'must not receive a Future-returning closure)', (tester) async {
    final store = await openSeededStore(tester);
    await tester.pumpWidget(
      MaterialApp(home: CharacterListScreen(store: store)),
    );
    // No pumpAndSettle anywhere in this test: the list's FutureBuilder
    // futures resolve only in the real zone (runAsync), so its loading
    // spinner would keep pumpAndSettle from ever settling. Bounded
    // pumps instead.
    await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 50)));
    await tester.pump();

    // The + button opens the creation wizard.
    await tester.tap(find.byType(FloatingActionButton));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    await tester.enterText(find.byType(TextField), 'Thistle Quickfoot');
    await tester.pump();

    // Species/class/background are optional — jump straight to the last
    // step via its header and create.
    await tester.tap(find.text('Background & alignment'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    await tester.tap(find.text('Create'));
    await tester.pump();
    await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    // The wizard pops to the list, which refreshes (real-zone IO) and
    // then pushes the sheet.
    await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    // The new character's sheet screen is pushed, with its name in the
    // app bar — and no "setState() callback argument returned a Future"
    // exception was thrown.
    expect(tester.takeException(), isNull);
    expect(find.widgetWithText(AppBar, 'Thistle Quickfoot'), findsOneWidget);
    await tester.runAsync(store.close);
  });
}
