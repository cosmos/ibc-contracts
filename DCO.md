# DCO

All code submitted to `ibc-contracts` must have a [Developer Certificate of Origin](https://developercertificate.org/) (DCO) sign-off.

The sign-off certifies that you are able to submit the contribution under the license of the repository, and for it to be redistributed under that same license. It must use your legal name, not a pseudonym. Git has a built-in mechanism to add it via the `-s` or `--signoff` argument to `git commit`, provided your `user.name` and `user.email` have been set up correctly.

## TL;DR

If you don't want to break the DCO check, ensure all your commits have a sign-off.

```bash
git config user.name "FIRST_NAME LAST_NAME"
git config user.email "MY_NAME@example.com"
git commit -s -m "fix(ics20): reject transfers with an empty denom"
```

This appends a line to your commit message:

```text
Signed-off-by: FIRST_NAME LAST_NAME <MY_NAME@example.com>
```

If you use the GitHub web UI for commits, make sure the `Signed-off-by` line uses the same email address as the commit author. This can be your GitHub `users.noreply.github.com` email if you keep your email address private.

## Signing off automatically

You can set up a git alias so that a sign-off is always added:

```bash
git config --global alias.ci "commit -s"
```

Then use `git ci -m "..."` instead of `git commit -m "..."`.

## Only a human may sign off

A sign-off is a statement by a person, so only a human may sign off on a commit. Where AI or LLM tooling assisted with a change, disclose it with a `Co-Authored-By` or `Assisted-By` line naming the tool, model, version, and context size. The human contributor still signs off and remains responsible for the change:

```text
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
Signed-off-by: Jane Contributor <jane@example.com>
```

## If the DCO check is failing on your pull request

The check reports which commits are missing a sign-off. To fix it you need to rewrite those commits and force-push.

For the most recent commit only:

```bash
git commit --amend --no-edit -s
git push --force-with-lease
```

For several commits, rebase over the range and sign off each one:

```bash
git rebase --signoff HEAD~<number-of-commits>
git push --force-with-lease
```

To sign off every commit on your branch since it diverged from `main`:

```bash
git rebase --signoff main
git push --force-with-lease
```

> [!NOTE]
> Always use `--force-with-lease` rather than `--force`, so that you do not overwrite work pushed by somebody else.

If you have any questions, ask in [GitHub Discussions](https://github.com/cosmos/ibc-contracts/discussions) or in the community chat.
