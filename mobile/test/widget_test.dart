// Smoke tests for the mobile shell. The dice-roller flow tests from
// the pre-content-platform shape no longer apply directly — the home
// screen is now gated on having a connected server. Those tests are
// removed for v1; restore them under a fake-connection test harness in
// v1.5 when we have a proper test rig for ServerConnection.

import 'package:flutter_test/flutter_test.dart';

import 'package:lorewyld/main.dart';
import 'package:lorewyld/services/server_connection.dart';

void main() {
  testWidgets('app boots and shows the lorewyld brand on the app bar',
      (tester) async {
    final connection = ServerConnection();
    await tester.pumpWidget(LorewyldApp(connection: connection));
    await tester.pumpAndSettle(const Duration(milliseconds: 100));
    expect(find.bySemanticsLabel('Lorewyld'), findsWidgets);
  });

  testWidgets('with no stored credentials, the connect screen renders',
      (tester) async {
    final connection = ServerConnection();
    await tester.pumpWidget(LorewyldApp(connection: connection));
    await tester.pumpAndSettle(const Duration(milliseconds: 100));
    expect(find.text('Connect to a server'), findsOneWidget);
    expect(find.text('Connect'), findsOneWidget);
  });
}
