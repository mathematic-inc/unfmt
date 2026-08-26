# Contributing

Mathematic uses AI agents to maintain this repository. Reviewing an unsolicited pull request usually takes longer than having an agent implement a proposal after we have agreed on it.

## Start with a Discussion

If you want to contribute:

1. [Start a Discussion](https://github.com/mathematic-inc/unfmt/discussions/new) describing the problem and your proposed solution.
2. Wait for a Mathematic maintainer to review the proposal and decide whether to implement it.
3. If we accept the proposal, a Mathematic maintainer or agent will open the pull request.

When we implement your proposal, the pull request will link to the Discussion and credit you as the proposal's original author.

GitHub restricts pull request creation to Mathematic maintainers and repository collaborators with write, maintain, or admin access, plus authorized maintenance agents. Everyone else should use Discussions.

## Development

Install the pinned tools with `mise install`. Run the same Clippy check as CI with:

```sh
mise exec -- cargo clippy --workspace --all-features --all-targets -- -D warnings
```
