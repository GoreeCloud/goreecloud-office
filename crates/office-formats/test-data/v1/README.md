# Governed Office Native Package v1 Test Data

These files are repository-local copies of the authoritative GoreeCloud Office Native Package v1 schema and fixture records stored in GoreeCloud/Data, Schemas, and APIs.

They were imported byte-for-byte from the connected authoritative Drive records on 2026-09-18. Drive remains the governing source for the schema records until governance explicitly changes that authority.

## Schema sources and imported SHA-256

- office-package-manifest-v1.schema.json — Drive 1zzUD4cT2vHvANB6GiCaRFBPtMlavi2Un — c58fc49a052d46fdf585ccb31d5a6ad5cd0c5dffa48f61d43ae4fbdbec181c8d
- office-package-metadata-v1.schema.json — Drive 1xaaflcQuSm-9O0Xkmntg7ZxsgjSf0PgJ — 9a48a123281488309b7882ba528126d085a1d53bf317547c69807b649799bf18
- office-package-relationships-v1.schema.json — Drive 1N-FATNBj-YkhY1FP8JAnjdhHjo3GYg8C — c1eab675d451e351735293550b5ba3151deffdb67f7b493094776f846c80a868
- office-package-compatibility-v1.schema.json — Drive 15ZOGIlQuIci-ByfHC6vivPoU97OQDHh3 — 82d3b813aaac5bcf14f8c4117b08822bb611dae4745ca5b41bb9fa7860b62b52
- office-package-integrity-v1.schema.json — Drive 1QUOtD20EPytsSqIR55jPhyDFo78Wt5V5 — f14a6c45967c80f8756bd9854c5203a077c6af45460c80dd4aa601d25a41e688
- office-writer-document-v1.schema.json — Drive 1g6pJSjop1eVX6eFrVOCUJ_NeFHQPPdLn — b1dd619ce3c67e7d515eac8314b5f910b813984fe6e12a953c31038d3c59047c

## Fixture source

- office-writer-v1-valid-minimal.gcwriter — Drive 1FhFV6vfgQa_iyAVvHjMLIRJkQmhz40iu — 6579cb7127a0ac82e65187d87272e399696acaf480ff3b9f98f6e48d0cc36cfd

The repository tests currently prove that these assets are present with the expected byte sizes, schema identifiers, Draft 2020-12 declaration, and ZIP signature/size for the fixture. This import does not yet establish JSON Schema evaluation, concrete ZIP parsing, cryptographic digest verification, or production package read/write.
