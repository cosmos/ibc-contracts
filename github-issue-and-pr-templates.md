> Repo files. These live under `.github/` in each repository. Three files below: two issue forms and one pull request template.

## .github/ISSUE_TEMPLATE/bug_report.yml

```yaml
name: Bug report
description: Report a problem so we can reproduce and fix it
labels: ["bug"]
body:
  - type: markdown
    attributes:
      value: |
        Do not report security vulnerabilities here. See SECURITY.md.
  - type: textarea
    id: what-happened
    attributes:
      label: What happened
      description: A clear description of the bug and what you expected instead.
    validations:
      required: true
  - type: textarea
    id: repro
    attributes:
      label: Steps to reproduce
      description: Exact steps, commands, and a code sample or executable test case if possible.
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Version
      description: Which release or commit are you running?
    validations:
      required: true
  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: OS, runtime versions, network, and deployment context.
```

## .github/ISSUE_TEMPLATE/feature_request.yml

```yaml
name: Feature or enhancement
description: Propose a change or new capability
labels: ["enhancement"]
body:
  - type: markdown
    attributes:
      value: |
        For substantial changes, expect to be asked for an ADR. Consider opening a Discussion first.
  - type: textarea
    id: problem
    attributes:
      label: Problem
      description: What need or use case does this address? Point to an adopter need where possible.
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposed change
      description: A step-by-step description of the enhancement.
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
```

## .github/pull_request_template.md

```markdown
## Description

What does this change and why. Link the issue: fixes #

## Checklist

- [ ] One logical change, diff kept as small as practical
- [ ] Commits signed off for the DCO (git commit -s), legal name used
- [ ] AI assistance, if any, disclosed with Co-Authored-By or Assisted-By
- [ ] Commit messages follow Conventional Commits
- [ ] Tests added or updated, covering expected and error paths
- [ ] CI is green
- [ ] SPDX license header on new files
- [ ] Changelog entry added if user-facing
- [ ] ADR merged first if this changes architecture, a public interface, or a spec
```
