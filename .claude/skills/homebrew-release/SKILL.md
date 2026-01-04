---
name: homebrew-release
description: Publishes a new Homebrew release with bottles. Creates GitHub release, updates formula, and triggers bottle builds. Use when user says "release", "publish", "homebrew", or "new version".
allowed-tools: Bash, Read, Edit, Write
---

# Homebrew Release

Publishes a new version of workbench to the Homebrew tap with pre-built bottles.

## Prerequisites

- Clean git working directory
- All tests passing
- GitHub CLI (`gh`) authenticated

## Release Steps

### 1. Determine version

Ask the user what version to release, or infer from context. Follow semver (vX.Y.Z).

### 2. Create GitHub release

Use `gh release create` which creates both the tag and release.

The `--generate-notes` flag auto-generates release notes from PR titles and commits since the last release.

For pre-1.0.0 versions, add `--prerelease` to mark as a prerelease:

```bash
# For v1.0.0+
gh release create vX.Y.Z --title "vX.Y.Z" --generate-notes

# For v0.x.x (prerelease)
gh release create v0.X.Y --title "v0.X.Y" --generate-notes --prerelease
```

### 3. Get the SHA256

After the release is created, get the tarball checksum:

```bash
curl -sL https://github.com/markmals/workbench/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
```

### 4. Update the formula

Clone or navigate to the homebrew-tap and update the formula:

```bash
# Clone if needed
gh repo clone markmals/homebrew-tap /tmp/homebrew-tap 2>/dev/null || true
cd /tmp/homebrew-tap
git checkout main && git pull

# Create branch
git checkout -b bump-vX.Y.Z

# Update Formula/workbench.rb:
# - Change url version: .../refs/tags/vX.Y.Z.tar.gz
# - Change sha256 to the new checksum
```

Edit `Formula/workbench.rb`:
- Update `url` to use new version tag
- Update `sha256` with new checksum

### 5. Create PR for bottle builds

**Important**: The PR must contain actual formula changes (version, sha256, etc.) for `brew test-bot` to build bottles. Empty commits or PRs without formula changes will not trigger bottle builds.

```bash
git add Formula/workbench.rb
git commit -m "workbench vX.Y.Z"
git push -u origin bump-vX.Y.Z
gh pr create --title "workbench vX.Y.Z" --body "Version bump with bottle builds"
```

### 6. Inform user about next steps

Tell the user:
- PR created at the returned URL
- CI will build bottles for macOS 15 and 26
- Once CI passes, add the `pr-pull` label to merge:
  ```bash
  gh pr edit <PR#> --repo markmals/homebrew-tap --add-label pr-pull
  ```

## Formula Location

The formula is at: `markmals/homebrew-tap` → `Formula/workbench.rb`

## Version in Formula

The formula URL pattern:
```ruby
url "https://github.com/markmals/workbench/archive/refs/tags/vX.Y.Z.tar.gz"
sha256 "<64-char-lowercase-hash>"
```
