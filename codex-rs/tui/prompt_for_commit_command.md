Inspect the current git repository changes and draft a commit message that is useful for later review and rollback.

Workflow

- Check the current working tree with `git status --short --untracked-files=all`.
- Review staged and unstaged changes with `git diff --staged` and `git diff`.
- Look at recent commit style with `git log --oneline -n 10`.
- If there are no meaningful changes, say so clearly instead of inventing a commit.

Output Requirements

- Start with a recommended commit subject line.
- Then provide a concise commit body, when needed, explaining the main changes and why they matter.
- Call out if the current changes look like they should be split into multiple commits.
- Keep the proposed message specific to the actual diff; do not use generic wording.
- Do not run `git commit` unless the user explicitly asks for it.
