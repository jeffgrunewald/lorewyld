import 'package:flutter_test/flutter_test.dart';
import 'package:lorewyld/dice/dice_expression_builder.dart';

void main() {
  group('formatDiceExpression', () {
    test('joins terms in insertion order with modifier last', () {
      expect(formatDiceExpression({10: 2, 8: 1}, 20), '2d10 + 1d8 + 20');
    });

    test('drops zero counts and non-positive modifiers', () {
      expect(formatDiceExpression({10: 2, 6: 0}, 0), '2d10');
      expect(formatDiceExpression({}, 0), '');
    });

    test('modifier alone renders as a bare number', () {
      expect(formatDiceExpression({}, 5), '5');
    });
  });

  group('parseDiceExpression', () {
    test('round-trips the builder format', () {
      final parsed = parseDiceExpression('2d10 + 1d8 + 20')!;
      expect(parsed.terms, {10: 2, 8: 1});
      expect(parsed.modifier, 20);
      expect(formatDiceExpression(parsed.terms, parsed.modifier),
          '2d10 + 1d8 + 20');
    });

    test('tolerates no spaces, uppercase D, and omitted count', () {
      expect(parseDiceExpression('18d10+72')!.terms, {10: 18});
      expect(parseDiceExpression('18d10+72')!.modifier, 72);
      expect(parseDiceExpression('D8')!.terms, {8: 1});
      expect(parseDiceExpression('2D6+d4')!.terms, {6: 2, 4: 1});
    });

    test('merges duplicate die terms', () {
      expect(parseDiceExpression('1d8 + 2d8')!.terms, {8: 3});
    });

    test('empty input parses to an empty expression', () {
      final parsed = parseDiceExpression('  ')!;
      expect(parsed.terms, isEmpty);
      expect(parsed.modifier, 0);
    });

    test('rejects anything beyond a plain sum of allowed dice', () {
      expect(parseDiceExpression('2d10 - 1'), isNull);
      expect(parseDiceExpression('1d100'), isNull);
      expect(parseDiceExpression('1d7'), isNull);
      expect(parseDiceExpression('2d10, 1d8'), isNull);
      expect(parseDiceExpression('5 + 5'), isNull);
      expect(parseDiceExpression('2d10 +'), isNull);
      expect(parseDiceExpression('100d10'), isNull);
      expect(parseDiceExpression('1d8 + 10000'), isNull);
    });
  });
}
