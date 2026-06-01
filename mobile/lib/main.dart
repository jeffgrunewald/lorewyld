import 'package:flutter/material.dart';

import 'services/server_connection.dart';
import 'widgets/lorewyld_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final connection = ServerConnection();
  await connection.load();
  runApp(LorewyldApp(connection: connection));
}

class LorewyldApp extends StatelessWidget {
  const LorewyldApp({super.key, required this.connection});

  final ServerConnection connection;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Lorewyld',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: LorewyldShell(connection: connection),
    );
  }
}
