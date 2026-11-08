#!/usr/bin/env bash
# Commit pending changes in the current repository to a sync branch and open
# (or refresh) one pull request. Used by the sync workflows in the
# repositories that publish Cybercore: the org site, cybercore-web-kit and
# cybercore-packs.
#
#   open-sync-pr.sh <branch> <title> <body-file>
#
# - No changes: exits 0 and does nothing.
# - The branch is rebuilt from the default branch each run and only pushed
#   when its content differs from what is already on the remote, so an open
#   PR is updated at most once per real upstream change.
# - Needs GH_TOKEN with contents + pull-requests write. If the repository
#   does not allow GitHub Actions to create pull requests, the branch is
#   still pushed and the run prints a link to open the PR by hand.
set -euo pipefail

BRANCH="$1"
TITLE="$2"
BODY_FILE="$3"

if git diff --quiet && git diff --cached --quiet && [ -z "$(git ls-files --others --exclude-standard)" ]; then
  echo "Already in sync; nothing to do."
  exit 0
fi

REPO="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
BASE="$(git rev-parse --abbrev-ref HEAD)"

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git checkout -q -B "$BRANCH"
git add -A
git commit -q -F - <<EOF
$TITLE

$(cat "$BODY_FILE")
EOF

NEW_TREE="$(git rev-parse HEAD^{tree})"
if git fetch -q origin "$BRANCH" 2>/dev/null && [ "$(git rev-parse FETCH_HEAD^{tree})" = "$NEW_TREE" ]; then
  echo "Branch $BRANCH already has these changes."
else
  # The sync branch is owned by this workflow; replacing it is intended.
  git push -q --force origin "$BRANCH"
  echo "Pushed $BRANCH."
fi

if [ -n "$(gh pr list --repo "$REPO" --head "$BRANCH" --state open --json number -q '.[].number')" ]; then
  echo "Pull request for $BRANCH is already open; it now shows the latest changes."
  exit 0
fi

if gh pr create --repo "$REPO" --base "$BASE" --head "$BRANCH" --title "$TITLE" --body-file "$BODY_FILE"; then
  exit 0
fi

URL="https://github.com/$REPO/compare/$BASE...$BRANCH?expand=1"
echo "::warning::Could not open the pull request automatically. Allow it under Settings > Actions > General > 'Allow GitHub Actions to create and approve pull requests', or open it here: $URL"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  printf '### Sync ready\n\nBranch `%s` is pushed. [Open the pull request](%s).\n' "$BRANCH" "$URL" >> "$GITHUB_STEP_SUMMARY"
fi
