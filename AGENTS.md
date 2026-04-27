## Building

**IMPORTANT: Do NOT install Rust locally.** GitHub Actions handles all building automatically. The CI workflow builds the binary and reports any errors back to you.

If you need to verify code changes without building:
1. Review the code logic manually
2. Check for syntax errors by reading the files
3. Push to GitHub and wait for the build results

## Git Push Workflow

Since gh CLI is not authenticated, use SSH directly:

```bash
cd /home/opencode/projects/blueak-session-manager
git add -A
git commit -m "<description>"
GIT_SSH_COMMAND="ssh -i /config/.ssh/github -o StrictHostKeyChecking=no" git push origin main
```

Note: Tags are pushed automatically by the CI workflow on successful builds. The workflow also creates GitHub releases with auto-incremented version tags starting at 0.0.1.

## Release Process

The CI workflow automatically creates releases on successful builds:
- Version auto-increments from highest existing tag
- Tags are created as `v{version}` (e.g. v0.0.1, v0.0.2)
- Cargo.toml is updated with the new version
- CHANGELOG.md is created/updated with release notes
- A GitHub release is created with the binary attached
- Homebrew tap formula is updated automatically

## GitHub Secrets

Set via `gh secret set`:
- `WEBHOOK_URL`: The webhook endpoint for build notifications
- `WEBHOOK_SECRET`: HMAC secret for webhook signature verification
- `TAP_TOKEN`: GitHub token for updating homebrew-tap repository