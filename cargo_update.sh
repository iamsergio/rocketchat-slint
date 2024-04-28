# SPDX-License-Identifier: MIT
# SPDX-FileCopyrightText: Sergio Martins

# Runs cargo update and creates a PR

# If called by a workflow, you'll need to manually close/open the PR
# to trigger all other workflows.

cargo update

if [ -n "$(git status --porcelain)" ]; then
    git add Cargo.lock
    git commit -m "chore: Cargo update"
    git checkout -B temp/cargo-update-`git log -1 --pretty=format:"%H"`
    git push --set-upstream origin `git branch --show-current`
    gh pr create --base main --title "chore: cargo update" --body "Automatically created via GH action."
fi
