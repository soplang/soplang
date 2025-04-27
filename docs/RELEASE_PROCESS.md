# Soplang Release Process

This document outlines the semi-automated release process for Soplang.

## Release Workflow Overview

Soplang uses a combination of manual binary building and automated release drafting:

1. **Manual Building**: Binaries for Windows, Linux, and macOS are built manually by the development team
2. **Automated Draft Creation**: GitHub Actions creates a draft release with appropriate templates
3. **Manual Asset Upload**: Binaries are uploaded to the draft release
4. **Manual Publishing**: The release is reviewed and published by a team member

## Creating a Release

### Step 1: Create a Release Branch

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

4. This will trigger the GitHub Actions workflow to:
   - Create a pull request to update the CHANGELOG.md with entries since the last release
   - Create a git tag for the release
   - Draft a GitHub release with a template for the release notes

5. Review and merge the automatically created PR to update the changelog

### Step 2: Build Platform-Specific Binaries

After the GitHub Action completes, build the binaries for each platform:

1. **Windows Binary**:
   ```bash
   cd windows
   ./build_windows.ps1  # Or build_windows.bat
   ```
   This will create `windows/Output/soplang-setup.exe`

2. **Linux Binary**:
   ```bash
   cd linux
   ./build_linux.sh
   ```
   This will create `linux/soplang_<version>_amd64.deb` (or .rpm)

3. **macOS Binary**:
   ```bash
   cd macos
   ./build_macos.sh
   ```
   This will create `macos/Soplang-<version>.dmg`

### Step 3: Upload Binaries and Publish the Release

1. Go to the "Releases" section on GitHub
2. Find the draft release created by the workflow
3. Upload the binaries you built as assets
4. Update the download links in the release description
5. Calculate and add SHA256 checksums for each file
6. Remove the checklist section
7. Review the release and publish it

### Step 4: Merge the Release Branch

After the release is published, merge the release branch back to main:
```bash
git checkout main
git merge release/v0.2.0
git push origin main
```

## Commit Message Format

For best changelog generation, use conventional commit messages:

- `feat: add new feature` - Appears in the "Added" section
- `fix: fix a bug` - Appears in the "Fixed" section
- `change: update existing functionality` - Appears in the "Changed" section
- `docs: update documentation` - Appears in the "Documentation" section
- `build: update build process` - Appears in the "Build" section
- `remove: remove feature` - Appears in the "Removed" section

## Troubleshooting

If the automated release process fails:

1. Check the GitHub Actions logs for error details
2. Make the necessary corrections
3. Push the changes to the release branch to trigger the workflow again

If you need to make a manual release entirely:
1. Follow steps 1-3 in the "Creating a Release" section
2. Manually create a GitHub release with the appropriate tag
3. Add the changelog content and file information manually
