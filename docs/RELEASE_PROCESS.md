# Soplang Release Process

This document outlines the semi-automated release process for Soplang.

## Release Workflow Overview

Soplang uses a combination of manual binary building and automated processes:

1. **Manual Building**: Binaries for Windows, Linux, and macOS are built manually by the development team
2. **Automated Docker Images**: Docker images are automatically built and pushed to Docker Hub and GitHub Container Registry
3. **Automated Draft Creation**: GitHub Actions creates a draft release with appropriate templates
4. **Manual Asset Upload**: Binaries are uploaded to the draft release
5. **Manual Publishing**: The release is reviewed and published by a team member

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
6. Verify that Docker images have been built and pushed successfully
7. Remove the checklist section
8. Review the release and publish it

### Step 4: Verify Docker Image Deployment

After publishing the release, the Docker workflow will automatically build and push Docker images:

1. Verify that the images have been pushed to Docker Hub at [soplang/soplang](https://hub.docker.com/r/soplang/soplang)
2. Verify that the images have been pushed to GitHub Container Registry at [ghcr.io/soplang/soplang](https://github.com/soplang/soplang/pkgs/container/soplang)
3. Test the Docker images by running:
   ```bash
   docker pull soplang/soplang:latest
   docker run -it --rm soplang/soplang -v
   ```

### Step 5: Merge the Release Branch

After the release is published and verified, merge the release branch back to main:
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

### Attribution and GitHub Links

For proper attribution in the changelog and release notes:

1. Ensure your git config has your GitHub username as the author name:
   ```bash
   git config --global user.name "sharafdin"
   ```

2. In commit messages, you can manually add attribution like:
   ```
   feat: add error messages [@sharafdin](https://github.com/sharafdin)
   ```

3. The automated changelog will create GitHub links for authors in the format:
   ```
   [@username](https://github.com/username)
   ```

4. When editing the release notes, ensure usernames are properly linked if the automation doesn't catch them

## Docker Image Tagging

The Docker automation will create the following tags:

1. Full version: `soplang/soplang:0.2.0`
2. Minor version: `soplang/soplang:0.2`
3. Major version: `soplang/soplang:0`
4. Latest tag: `soplang/soplang:latest` (only on main branch)

The same tags are also created in the GitHub Container Registry with the prefix `ghcr.io/soplang/`.

## Troubleshooting

If the automated release process fails:

1. Check the GitHub Actions logs for error details
2. Make the necessary corrections
3. Push the changes to the release branch to trigger the workflow again

If Docker image building fails:
1. Check the Docker workflow logs in the GitHub Actions tab
2. Common issues include permission problems or missing secrets
3. Ensure `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` secrets are configured in the repository
