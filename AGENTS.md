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

Note: Tags are pushed automatically by the CI workflow on successful builds. The workflow also creates GitHub releases with auto-incremented version tags starting at 0.00.1.

## GIT Webhook info:

WEBHOOK_URL:  https://webhook.akinus21.com/webhook/blueak-session-manager-build
WEBHOOK_SECRET: 4d82982b0a0010a706a40cf272f49c9ddfee93162a2c4b714eebc6ded10038f5

## Release Process

The CI workflow automatically creates releases on successful builds:
- Version starts at 0.00.1 and auto-increments
- Tags are created as `v{version}` (e.g. v0.00.1, v0.00.2)
- Cargo.toml is updated with the new version
- CHANGELOG.md is created/updated with release notes
- README.md version is updated if present
- A GitHub release is created with the binary attached

GitHub secrets (set via `gh secret set`):
- `WEBHOOK_URL`: The webhook endpoint for build notifications
- `WEBHOOK_SECRET`: HMAC secret for webhook signature verification