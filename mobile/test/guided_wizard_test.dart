// The guided quiz walks through every question, ranks the seeded SRD
// content via the shared Rust core, and hands the picks to the create
// wizard as prefills.
//
// LocalStore does real isolate IO (sqflite ffi) — wrap open/close and
// scoring waits in tester.runAsync (see widget_test.dart for the
// pattern). The Rust core is initialized once in flutter_test_config.

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';

import 'package:lorewyld/ffi/api/guidance.dart';
import 'package:lorewyld/screens/character_guided_wizard_screen.dart';
import 'package:lorewyld/services/content_store.dart';
import 'package:lorewyld/services/local_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  sqfliteFfiInit();
  databaseFactory = databaseFactoryFfi;

  testWidgets('quiz answers produce ranked picks that prefill the wizard', (
    tester,
  ) async {
    final store = (await tester.runAsync(() async {
      final s = await LocalStore.open(path: inMemoryDatabasePath);
      await ContentStore(s).importBundle();
      return s;
    }))!;

    await tester.pumpWidget(
      MaterialApp(home: CharacterGuidedWizardScreen(store: store)),
    );
    await tester.pump();

    // Drive the quiz from the engine's own questionnaire: answer every
    // question with its first option.
    final quiz = jsonDecode(guidanceQuestionnaire()) as Map<String, dynamic>;
    final questions = quiz['questions'] as List;
    for (final (i, question) in questions.indexed) {
      final q = question as Map<String, dynamic>;
      final firstLabel = (q['options'] as List).first['label'] as String;
      expect(
        find.text(q['prompt'] as String),
        findsOneWidget,
        reason: 'question ${q['key']} should be visible',
      );
      // Display order is shuffled per quiz, so the engine's first
      // option may sit anywhere in the list.
      await tester.ensureVisible(find.text(firstLabel));
      await tester.tap(find.text(firstLabel));
      await tester.pump();
      // Page-turn animation, then (on the last answer) real-zone content
      // reads + scoring.
      await tester.pump(const Duration(milliseconds: 300));
      if (i == questions.length - 1) {
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 200)),
        );
        await tester.pump();
      }
    }

    // Results: ranked cards with fit scores and reasons per category.
    // The list builds lazily — scroll each section into view.
    expect(find.text('Class'), findsOneWidget);
    expect(find.textContaining('% fit'), findsAtLeast(3));
    expect(find.textContaining('•'), findsWidgets);

    final scrollable = find.byType(Scrollable).first;
    for (final marker in [
      'Species',
      'Background',
      'Suggested starting stats',
      'Use these picks',
    ]) {
      await tester.scrollUntilVisible(
        find.text(marker),
        200,
        scrollable: scrollable,
      );
      expect(find.text(marker), findsOneWidget);
    }
    expect(find.textContaining('Alignment:'), findsOneWidget);

    // Hand off to the create wizard with the picks prefilled.
    await tester.tap(find.text('Use these picks'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(find.widgetWithText(AppBar, 'New character'), findsOneWidget);
    // The wizard's stepper is on top (the results screen stays in the
    // navigator below it). Species/class/background steps only carry a
    // subtitle when a record is pre-selected — all three must be.
    final stepper = tester.widget<Stepper>(find.byType(Stepper));
    expect(stepper.steps[1].subtitle, isNotNull, reason: 'species prefilled');
    expect(stepper.steps[2].subtitle, isNotNull, reason: 'class prefilled');

    await tester.runAsync(store.close);
  });
}
