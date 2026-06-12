// Mirror of `lorewyld_types::content_module::ContentModule`.

/// Mirror of `lorewyld_types::content_module::LicenseKind` wire values.
/// CC-BY-4.0 and OGL 1.0a are the supported distribution licenses;
/// `unlicensed` marks homebrew published without one (never valid for
/// pre-bundled content).
const licenseKinds = ['cc-by-4.0', 'ogl-1.0a', 'unlicensed'];

String licenseDisplayName(String wire) => switch (wire) {
      'cc-by-4.0' => 'CC-BY-4.0',
      'ogl-1.0a' => 'OGL 1.0a',
      'unlicensed' => 'Unlicensed',
      _ => wire,
    };

class ContentModule {
  final String uuid;
  final String name;
  final String slug;
  final String license;
  final String? licenseUrl;
  final int schemaVersion;
  final List<String> authors;
  final String? publisher;
  final String? description;
  final String? websiteUrl;
  final bool isActive;
  final int ordering;
  final String versionString;
  final String? previousVersionUuid;
  final DateTime? publishedAt;
  final DateTime createdAt;
  final DateTime updatedAt;

  const ContentModule({
    required this.uuid,
    required this.name,
    required this.slug,
    required this.license,
    this.licenseUrl,
    required this.schemaVersion,
    required this.authors,
    this.publisher,
    this.description,
    this.websiteUrl,
    required this.isActive,
    required this.ordering,
    required this.versionString,
    this.previousVersionUuid,
    this.publishedAt,
    required this.createdAt,
    required this.updatedAt,
  });

  factory ContentModule.fromJson(Map<String, dynamic> json) => ContentModule(
        uuid: json['uuid'] as String,
        name: json['name'] as String,
        slug: json['slug'] as String,
        license: json['license'] as String,
        licenseUrl: json['license_url'] as String?,
        schemaVersion: (json['schema_version'] as num).toInt(),
        authors: (json['authors'] as List<dynamic>? ?? const [])
            .map((e) => e as String)
            .toList(),
        publisher: json['publisher'] as String?,
        description: json['description'] as String?,
        websiteUrl: json['website_url'] as String?,
        isActive: json['is_active'] as bool? ?? true,
        ordering: (json['ordering'] as num?)?.toInt() ?? 0,
        versionString: json['version_string'] as String? ?? '1.0.0',
        previousVersionUuid: json['previous_version_uuid'] as String?,
        publishedAt: (json['published_at'] as String?) != null
            ? DateTime.parse(json['published_at'] as String)
            : null,
        createdAt: DateTime.parse(json['created_at'] as String),
        updatedAt: DateTime.parse(json['updated_at'] as String),
      );
}
