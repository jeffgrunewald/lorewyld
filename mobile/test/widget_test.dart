import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:lorewyld/dice/dice_icon.dart';
import 'package:lorewyld/dice/dice_roller_screen.dart';
import 'package:lorewyld/dice/dice_type.dart';
import 'package:lorewyld/main.dart';

Finder _diceButton(DiceType type) => find.ancestor(
      of: find.byWidgetPredicate(
        (w) => w is DiceIcon && w.type == type,
      ),
      matching: find.byType(OutlinedButton),
    );

void main() {
  testWidgets('home screen shows Lorewyld branding', (tester) async {
    await tester.pumpWidget(const LorewyldApp());
    expect(find.bySemanticsLabel('Lorewyld'), findsWidgets);
  });

  testWidgets('dice roller opens, enqueues, rolls, and clears', (tester) async {
    await tester.pumpWidget(const LorewyldApp());

    await tester.tap(find.bySemanticsLabel('Open dice roller'));
    await tester.pumpAndSettle();
    expect(find.byType(DiceRollerScreen), findsOneWidget);

    for (var i = 0; i < 3; i++) {
      await tester.tap(_diceButton(DiceType.d6));
      await tester.pump();
    }
    expect(find.text('x3'), findsOneWidget);

    expect(find.text('Roll'), findsOneWidget);
    await tester.tap(find.text('Roll'));
    await tester.pumpAndSettle();
    expect(find.textContaining('Total:'), findsOneWidget);
    expect(find.text('Clear'), findsOneWidget);

    await tester.tap(find.text('Clear'));
    await tester.pumpAndSettle();
    expect(find.text('x3'), findsNothing);
    expect(find.textContaining('Total:'), findsNothing);
    expect(find.text('Roll'), findsOneWidget);
  });
}
