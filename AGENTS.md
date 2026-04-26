## Building

**IMPORTANT: Do NOT install Rust locally.** GitHub Actions handles all building automatically. The CI workflow builds the binary and reports any errors back to you.

If you need to verify code changes without building:
1. Review the code logic manually
2. Check for syntax errors by reading the files
3. Push to GitHub and wait for the build results

## Git Push Workflow

Since gh CLI is not authenticated, use SSH directly:

```bash
cd /home/opencode/projects/aktools
git add -A
git commit -m "<description>"
GIT_SSH_COMMAND="ssh -i /config/.ssh/github -o StrictHostKeyChecking=no" git push origin main
```
## GIT Webhook info:

WEBHOOK_URL:  https://webhook.akinus21.com/webhook/blueak-session-manager-build
WEBHOOK_SECRET: 4d82982b0a0010a706a40cf272f49c9ddfee93162a2c4b714eebc6ded10038f5
