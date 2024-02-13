# Contributing Guidelines

Thank you for your interest in contributing to the Radix node!

## Clarification on GitHub Issue Usage and Feature Requests

We want to clarify that GitHub Issues are primarily meant for the purpose of reporting problems or concerns, rather than functioning as an open bug tracker. This means that reported issues on Github may be closed and reported in our internal tracking system or added to our roadmap.

If you are thinking of requesting a feature, make sure it’s not already part of our upcoming features outlined in the [Roadmap](https://docs.radixdlt.com/docs/roadmap). If you have a feature suggestion, we kindly ask that you share it through [Discord](http://discord.gg/radixdlt) or [Telegram](https://t.me/RadixDevelopers).

Our primary focus is on the priorities outlined in our Roadmap. We appreciate your understanding that addressing reported issues may not always align with our immediate roadmap goals.

# Table of contents
- [Code of conduct](#code-of-conduct)
- [Reporting Issues](#reporting-issues)
- [Before you contribute](#before-you-contribute)
- [Get started](#get-started)
  - [Setting up your environment](#setting-up-your-environment)
- [Branching strategy](#branching-strategy)
  - [Rebasing](#rebasing)
- [Contribute](#contribute)
  - [Code style](#code-style)
  - [Code structure](#code-structure)
  - [Testing](#testing)
  - [Commit messages](#commit-messages)
  - [Opening a pull request](#opening-a-pull-request)
  - [Review process](#review-process)
  - [License](#license)

# Code of conduct

This project adheres to the Contributor Covenant [code of conduct](CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code.
Please report unacceptable behavior to [hello@radixdlt.com](mailto:hello@radixdlt.com).

# Reporting Issues

Ensure the bug was not already reported by searching on GitHub under [Issues](https://github.com/radixdlt/babylon-node/issues).

If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/radixdlt/babylon-node/issues). Make sure to provide as much detail as possible, including:

- A clear and descriptive title.
- Steps to reproduce the issue.
- Expected behavior and actual behavior.
- Your operating system and other relevant information.
- If possible, include screenshots or code snippets that illustrate the issue.

# Before you contribute

Prior to commencing any work on a PR, we strongly advise initiating a discussion with the team via Discord, Telegram, or GitHub Issues (for bugs).

Submitting a Pull Request does not guarantee the acceptance of your proposed changes.

# Get started

## Setting up your environment

Please refer to the [development docs](./docs/development) on how to setup your local environment.

# Contribute

## Branching strategy

Please read the [branching strategy documentation](./docs/branching-strategy.md) to know how to branch correctly.

## Code style

Write clear, concise, and well-documented code.

We rely on automatic code formatting. The following command will format both the Java and Rust code:

```shell
$ ./gradlew spotlessApply
```

### Braces Always Required

* Braces are always required with `if`, `else`, `for`, `do` and `while` statements, even when the body of the statement is empty or contains only a single statement.

### Use of "this." for Field Access

* Use of the `this` keyword is preferred in situations where there may be ambiguity in field and variable names, such as in setters and constructors.

## Code structure

### Javadoc locations

* Properly formatted and complete Javadoc must be included for all fields and methods with either `public` or `protected` visibility.
* Note that overridden instance methods or implemented `interface` methods need not have Javadoc if the inherited Javadoc is correct and suitable.  In particular methods that override superclass methods and change the behaviour of the method should document the new behaviour.

## Testing

1. Ensure that your changes do not break existing tests (refer to the [Development Docs](https://github.com/radixdlt/babylon-node/tree/main/docs/development) on how to run the tests).
2. Write new tests for your code if applicable.

## Commit messages

Commit your changes with a descriptive commit message.

*  We prefer [convential commit messages](https://www.conventionalcommits.org/en/v1.0.0/), although don't enforce it
*  Separate the subject from body with a blank line
*  Limit the subject line to 50 characters
*  Capitalise the subject line
*  Do not end the subject line with a period
*  Use the imperative mood in the subject line
*  Use the body to explain what and why vs. how, separating paragraphs with an empty line.

## Opening a pull request

* Fork the codebase and make changes, following these guidelines.
* Submit a new GitHub pull request with the proposed patch for review.
* Ensure the **pull request** description clearly describes the problem and solution. Include the relevant issue number if applicable.

## Review Process

Pull requests will be reviewed by project maintainers. Reviewers may provide feedback, request changes, or approve the pull request.

We appreciate your patience during this process, and we aim to be responsive and constructive in our feedback.

## License

By contributing to RadixDLT node you agree that your contributions will be licensed under [Radix License 1.0 (modified Apache 2.0)](LICENSE).
