// A content module as described by the shipped bundle manifest
// (assets/content/srd-bundle.meta.json). The manifest — not the
// database — is the source of truth for what *ships*, so the module
// management UI can describe modules whether or not they're installed.

class BundledModule {
  final String slug;
  final String name;

  /// License wire value ('cc-by-4.0', 'ogl-1.0a', 'unlicensed').
  final String license;
  final String? licenseUrl;
  final String? publisher;
  final List<String> authors;
  final String? description;
  final String? websiteUrl;

  /// Source document names contained in this module.
  final List<String> documents;

  /// Record family → count ('spells': 339, 'creatures': 369, …).
  final Map<String, int> recordCounts;

  const BundledModule({
    required this.slug,
    required this.name,
    required this.license,
    this.licenseUrl,
    this.publisher,
    this.authors = const [],
    this.description,
    this.websiteUrl,
    this.documents = const [],
    this.recordCounts = const {},
  });

  factory BundledModule.fromJson(Map<String, dynamic> json) => BundledModule(
    slug: json['slug'] as String,
    name: json['name'] as String,
    license: json['license'] as String,
    licenseUrl: json['license_url'] as String?,
    publisher: json['publisher'] as String?,
    authors: [
      for (final a in json['authors'] as List<dynamic>? ?? const [])
        a as String,
    ],
    description: json['description'] as String?,
    websiteUrl: json['website_url'] as String?,
    documents: [
      for (final d in json['documents'] as List<dynamic>? ?? const [])
        d as String,
    ],
    recordCounts: {
      for (final e
          in (json['record_counts'] as Map<String, dynamic>? ?? const {})
              .entries)
        e.key: (e.value as num).toInt(),
    },
  );

  int get totalRecords =>
      recordCounts.values.fold(0, (sum, count) => sum + count);
}
