# Contributing to Agedashi

## Commit Message Format

This project uses **Conventional Commits** for automatic semantic versioning. Please format your commit messages as follows:

### Format

```
<type>: <description>

[optional body]

[optional footer]
```

### Types

- **feat**: A new feature (triggers **minor** version bump: 0.1.0 → 0.2.0)
- **fix**: A bug fix (triggers **patch** version bump: 0.1.0 → 0.1.1)
- **docs**: Documentation changes (no version bump)
- **style**: Code style changes (formatting, etc., no version bump)
- **refactor**: Code refactoring (no version bump)
- **test**: Adding or updating tests (no version bump)
- **chore**: Maintenance tasks (no version bump)

### Breaking Changes

Add `!` after the type or `BREAKING CHANGE:` in the footer to trigger a **major** version bump (0.1.0 → 1.0.0):

```
feat!: redesign CLI interface

BREAKING CHANGE: --output flag renamed to --format
```

### Examples

**Minor version bump (new feature):**
```
feat: add support for Azure resources
```

**Patch version bump (bug fix):**
```
fix: correct icon sizing in PDF output
```

**Major version bump (breaking change):**
```
feat!: change default output format to SVG

BREAKING CHANGE: The default output format is now SVG instead of PNG.
Users can use --output png to get the old behavior.
```

**No version bump:**
```
docs: update README with installation instructions
```

## Release Process

### Automatic Releases

Releases are created automatically via GitHub Actions:

- **Develop branch**: Merging to `develop` creates a prerelease (e.g., `v0.2.0-beta.42`)
- **Main branch**: Merging to `main` creates a full release (e.g., `v0.2.0`)

### Version Calculation

The version is automatically calculated based on commits since the last release:

1. If any commit has `BREAKING CHANGE` or `!`: **major** bump
2. If any commit starts with `feat:`: **minor** bump
3. If any commit starts with `fix:`: **patch** bump
4. Otherwise: **patch** bump

### Manual Release

To create a release manually:

```bash
# Make sure you're on the branch you want to release
git checkout main  # or develop

# Push a commit that will trigger the release
git commit --allow-empty -m "chore: trigger release"
git push origin main  # or develop
```

## Development Workflow

1. Create a feature branch from `develop`
2. Make changes with conventional commit messages
3. Push and create a PR to `develop`
4. Once merged, a beta prerelease is automatically created
5. When ready for production, merge `develop` → `main`
6. A full release is automatically created

## Questions?

If you have questions about contributing, please open an issue!
