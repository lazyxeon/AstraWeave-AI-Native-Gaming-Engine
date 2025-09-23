# SBOM Generation for AstraWeave

This script generates a Software Bill of Materials (SBOM) for the workspace using CycloneDX.

## Usage

Run from the workspace root:

```
cargo install cargo-cyclonedx --locked  # if not already installed
cargo cyclonedx -o sbom.xml
```

- The output `sbom.xml` can be uploaded to CI artifacts or used for compliance.
- See https://cyclonedx.org/ for more info.
