const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

const SCHEMAS: [(&str, &str, usize); 6] = [
    (
        include_str!("../test-data/v1/schemas/office-package-manifest-v1.schema.json"),
        "urn:goreecloud:office:schema:package-manifest:v1",
        2149,
    ),
    (
        include_str!("../test-data/v1/schemas/office-package-metadata-v1.schema.json"),
        "urn:goreecloud:office:schema:package-metadata:v1",
        859,
    ),
    (
        include_str!("../test-data/v1/schemas/office-package-relationships-v1.schema.json"),
        "urn:goreecloud:office:schema:package-relationships:v1",
        1305,
    ),
    (
        include_str!("../test-data/v1/schemas/office-package-compatibility-v1.schema.json"),
        "urn:goreecloud:office:schema:package-compatibility:v1",
        823,
    ),
    (
        include_str!("../test-data/v1/schemas/office-package-integrity-v1.schema.json"),
        "urn:goreecloud:office:schema:package-integrity:v1",
        1251,
    ),
    (
        include_str!("../test-data/v1/schemas/office-writer-document-v1.schema.json"),
        "urn:goreecloud:office:schema:writer-document:v1",
        2186,
    ),
];

const WRITER_FIXTURE: &[u8] =
    include_bytes!("../test-data/v1/fixtures/office-writer-v1-valid-minimal.gcwriter");

#[test]
fn governed_schema_assets_are_present_with_expected_identity_and_size() {
    for (schema, id, expected_bytes) in SCHEMAS {
        assert_eq!(schema.len(), expected_bytes);
        assert!(schema.contains(DRAFT_2020_12));
        assert!(schema.contains(id));
    }
}

#[test]
fn governed_writer_fixture_is_present_with_expected_zip_signature_and_size() {
    assert_eq!(WRITER_FIXTURE.len(), 1915);
    assert_eq!(&WRITER_FIXTURE[..4], b"PK\x03\x04");
}
