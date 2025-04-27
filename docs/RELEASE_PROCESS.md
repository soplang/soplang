# Soplang Release Process

This document outlines the automated release process for Soplang.

## Automated Release Workflow

Soplang uses GitHub Actions to automate the release process. The workflow is triggered when code is pushed to a branch that matches the pattern `release/v*` (e.g., `release/v1.2.0`).

### How to Create a Release

1. Create a new branch from `main` using the naming convention `release/vX.Y.Z` (e.g., `release/v0.2.0`):
   ```bash
   git checkout main
   git pull
   git checkout -b release/v0.2.0
   ```

2. Make any final adjustments needed for the release (e.g., update version numbers in files)

3. Push the branch to GitHub:
   ```bash
   git push origin release/v0.2.0
   ```

4. The GitHub Actions workflow will automatically:
   - Build the project
   - Update the CHANGELOG.md
   - Create a GitHub release with the appropriate tag

5. After the workflow completes successfully, merge the release branch back to main:
   ```bash
   git checkout main
   git merge release/v0.2.0
   git push origin main
   ```

## Workflow Details

The automated release process performs the following steps:

1. **Build the project**
   - Builds Soplang using the universal build script
   - Commits any modified files resulting from the build process

2. **Update the changelog**
   - Creates CHANGELOG.md if it doesn't exist
   - Generates a changelog entry based on commits since the last release
   - Commits the updated changelog

3. **Create a GitHub release**
   - Tags the repository with the version number
   - Creates a GitHub release using the changelog as release notes

## Commit Message Format

To ensure the changelog is generated correctly, use conventional commit messages:

- `feat: add new feature` - Appears in the "Added" section
- `fix: fix a bug` - Appears in the "Fixed" section
- `change: update existing functionality` - Appears in the "Changed" section
- `docs: update documentation` - Appears in the "Documentation" section
- `build: update build process` - Appears in the "Build" section
- `remove: remove feature` - Appears in the "Removed" section

## Manual Release Process

If you need to create a release manually:

1. Build the project
2. Update the CHANGELOG.md
3. Commit and tag the release
4. Create a GitHub release with the appropriate tag

## Troubleshooting

If the automated release process fails:

1. Check the GitHub Actions logs for error details
2. Make the necessary corrections
3. Push the changes to the release branch to trigger the workflow again
