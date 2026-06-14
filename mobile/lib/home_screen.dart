import 'package:flutter/material.dart';

import 'dice/dice_icon.dart';
import 'dice/dice_roller_screen.dart';
import 'dice/dice_type.dart';
import 'screens/character_list_screen.dart';
import 'screens/compendium_screen.dart';
import 'screens/module_management_screen.dart';
import 'screens/search_screen.dart';
import 'screens/setting_list_screen.dart';
import 'services/content_store.dart';
import 'services/local_store.dart';
import 'services/server_connection.dart';

/// Navigation hub. Everything here works fully offline — the Compendium
/// browses the locally installed content modules.
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.connection, required this.store});

  final ServerConnection connection;
  final LocalStore store;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  @override
  void initState() {
    super.initState();
    widget.connection.addListener(_onConnectionChanged);
  }

  @override
  void dispose() {
    widget.connection.removeListener(_onConnectionChanged);
    super.dispose();
  }

  void _onConnectionChanged() {
    if (mounted) setState(() {});
  }

  void _push(Widget screen) {
    Navigator.of(context).push(MaterialPageRoute(builder: (_) => screen));
  }

  @override
  Widget build(BuildContext context) {
    final connection = widget.connection;
    final loggedIn = connection.isLoggedIn;
    final greeting = loggedIn && connection.user != null
        ? 'Welcome, ${connection.user!.username}'
        : 'Welcome';
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Image.asset(
              'assets/branding/wordmark.png',
              height: 120,
              fit: BoxFit.contain,
              semanticLabel: 'Lorewyld',
            ),
            const SizedBox(height: 24),
            Text(greeting, style: Theme.of(context).textTheme.titleMedium),
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                loggedIn
                    ? 'Connected to ${connection.serverUrl}'
                    : 'Working locally — no server connection',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
            const SizedBox(height: 24),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              alignment: WrapAlignment.center,
              children: [
                FilledButton.icon(
                  onPressed: () =>
                      _push(CharacterListScreen(store: widget.store)),
                  icon: const Icon(Icons.person_outline),
                  label: const Text('Characters'),
                ),
                FilledButton.icon(
                  onPressed: () => _push(
                    SettingListScreen(
                      connection: connection,
                      store: widget.store,
                    ),
                  ),
                  icon: const Icon(Icons.library_books_outlined),
                  label: const Text('Settings & lore'),
                ),
                FilledButton.tonalIcon(
                  onPressed: () => _push(SearchScreen(store: widget.store)),
                  icon: const Icon(Icons.search),
                  label: const Text('Search'),
                ),
                FilledButton.tonalIcon(
                  onPressed: () => _push(
                    CompendiumScreen(
                      content: ContentStore(widget.store),
                      connection: connection,
                    ),
                  ),
                  icon: const Icon(Icons.collections_bookmark_outlined),
                  label: const Text('Compendium'),
                ),
                FilledButton.tonalIcon(
                  onPressed: () =>
                      _push(ModuleManagementScreen(store: widget.store)),
                  icon: const Icon(Icons.inventory_2_outlined),
                  label: const Text('Modules'),
                ),
              ],
            ),
          ],
        ),
      ),
      floatingActionButton: _D20FloatingButton(
        onPressed: () => _push(const DiceRollerScreen()),
      ),
    );
  }
}

class _D20FloatingButton extends StatelessWidget {
  const _D20FloatingButton({required this.onPressed});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: 'Open dice roller',
      child: GestureDetector(
        onTap: onPressed,
        behavior: HitTestBehavior.opaque,
        child: const SizedBox(
          width: 72,
          height: 72,
          child: DiceIcon(type: DiceType.d20, size: 72),
        ),
      ),
    );
  }
}
