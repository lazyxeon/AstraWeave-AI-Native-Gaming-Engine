# Supplemental Documentation Archive

This directory consolidates the extensive reports, fix summaries, and process notes that previously lived at the repository root. Files are grouped alphabetically to make it easier to scan for specific topics.

Key highlights:

- **Policies & Contribution**: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and security materials now live here so project-wide guidelines remain in a single location.
- **Operational Reports**: Completion summaries, fix reports, and project status documents are preserved for historical reference.
- **Guides & Playbooks**: Quick-start, verification, and setup guides sit alongside the reports for quick lookup when onboarding new contributors.

If you maintain tooling that referenced any of these files by their old root-level paths, update the paths to `docs/supplemental-docs/<file-name>` or the new [`docs/root-archive/`](../root-archive/README.md) location. The root `README.md` and workspace automation have already been updated accordingly.

**Note:** The canonical `roadmap.md` now lives in `docs/root-archive/roadmap.md` alongside the other archived reports. A mirrored copy remains in this folder so mdBook navigation continues to work. Update the archive file first, then run `scripts/sync-roadmap.sh` to refresh this mirror.

