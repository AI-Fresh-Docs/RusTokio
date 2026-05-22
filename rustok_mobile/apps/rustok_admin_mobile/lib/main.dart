import 'package:app_graphql/app_graphql.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:graphql/client.dart';

import 'registry/mobile_module_registry.dart';
import 'routes/app_router.dart';

const _defaultServerBaseUrl = String.fromEnvironment(
  'RUSTOK_SERVER_BASE_URL',
  defaultValue: 'http://localhost:8080',
);
const _defaultTenantSlug = String.fromEnvironment(
  'RUSTOK_TENANT_SLUG',
  defaultValue: 'default',
);
const _defaultLocale = String.fromEnvironment(
  'RUSTOK_LOCALE',
  defaultValue: 'en',
);

final mobileRegistryProvider = Provider((ref) => buildMobileModuleRegistry());

final graphQlConfigProvider = Provider<GraphQlClientConfig>((ref) {
  return GraphQlClientConfig(
    baseUri: Uri.parse(_defaultServerBaseUrl),
    context: const GraphQlRequestContext(
      tenantSlug: _defaultTenantSlug,
      locale: _defaultLocale,
    ),
  );
});

final graphQlClientProvider = Provider<GraphQLClient>((ref) {
  final config = ref.watch(graphQlConfigProvider);
  return const GraphQlClientFactory().create(config);
});

void main() {
  runApp(const ProviderScope(child: RusTokAdminMobileApp()));
}

class RusTokAdminMobileApp extends ConsumerWidget {
  const RusTokAdminMobileApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.watch(graphQlClientProvider);
    final router = buildRouter(ref.watch(mobileRegistryProvider));
    return MaterialApp.router(
      title: 'RusTok Admin Mobile',
      theme: ThemeData(useMaterial3: true, colorSchemeSeed: Colors.deepPurple),
      routerConfig: router,
    );
  }
}
