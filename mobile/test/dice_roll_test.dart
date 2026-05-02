import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:lorewyld/dice/dice_type.dart';

void main() {
  group('DiceType.roll', () {
    test('every die type produces values in [1, sides]', () {
      final random = Random.secure();
      for (final type in DiceType.values) {
        for (var i = 0; i < 10000; i++) {
          final result = type.roll(random);
          expect(result, greaterThanOrEqualTo(1),
              reason: '${type.label} produced $result (< 1)');
          expect(result, lessThanOrEqualTo(type.sides),
              reason: '${type.label} produced $result (> ${type.sides})');
        }
      }
    });

    test('d20 covers every face over 100k rolls', () {
      final random = Random.secure();
      final counts = List<int>.filled(21, 0);
      for (var i = 0; i < 100000; i++) {
        counts[DiceType.d20.roll(random)] += 1;
      }
      expect(counts[0], 0, reason: 'face 0 should never appear');
      for (var face = 1; face <= 20; face++) {
        expect(counts[face], greaterThan(3000),
            reason: 'face $face under-represented: ${counts[face]} / 100000');
      }
    });

    test('d100 covers full 1..100 range', () {
      final random = Random.secure();
      final seen = <int>{};
      for (var i = 0; i < 50000 && seen.length < 100; i++) {
        seen.add(DiceType.d10Percentile.roll(random));
      }
      expect(seen.length, 100, reason: 'd100 missed faces: ${seen.length}/100');
    });
  });
}
