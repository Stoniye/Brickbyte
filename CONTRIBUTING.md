# Contributing to Brickbyte

Everyone is welcome to contribute to **Brickbyte**.

I review pull requests based on my own judgment. There is no fixed formula or checklist that guarantees a merge. If I decide not to merge a pull request, I will **always explain why**, I won’t silently close it. Of course, you will still be able to improve your PR when i decided to not merge it and i will review it again.

I will only review pull requests and issues on the main repository on [GitLab](https://gitlab.com/Stoniye/brickbyte).

That said, there are some **hard requirements**. If these are not met, the pull request will not be merged with a high chance.

---

## Must-Haves

### 1. Fits the Game

New content or features must **fit naturally into the game**. They should not feel like a mod or something randomly added.

This includes:

* Art style
* Sound effects
* Overall atmosphere
* Game feel

Whether something fits or not is ultimately judged by my own opinion.

---

### 2. Not Bloated or Unoptimized

I strongly dislike bloated or poorly optimized code.

You don’t need to over-optimize everything, but:

* Avoid unnecessary complexity
* Avoid obvious inefficiencies
* Try to get the best out of the code

I continuously try to improve my own code as well, so this applies to everyone equally.

---

### 3. Short Explanation

Your pull request must include a short explanation of:

* What you changed or added
* Why you made the change or added this feature

If relevant, also include a brief description of **how** you implemented it.

---

### 4. No (or Very Few) Additional Dependencies

I strongly prefer **no external libraries or dependencies**.

My goal is to implement as much as possible myself (ideally even avoiding things like GLEW, egui, etc.). While this may change in the future, for now:

* New dependencies are only acceptable if they are **absolutely necessary**
* The functionality would otherwise require a massive amount of work to implement from scratch

I dislike dependencies for the same reasons mentioned under *[Not Bloated or Unoptimized](https://gitlab.com/Stoniye/brickbyte/-/blob/main/CONTRIBUTING.md#2-not-bloated-or-unoptimized)*: loss of control and unnecessary complexity.

---

## Nice-to-Haves (Bonus Points)

These are **not required**, but they increase the chances of a merge and make my life easier.

### 1. Documentation

If your pull request adds a larger feature, include short documentation explaining how it works.

(Not necessary for small or simple changes)

---

### 2. Complete Features

Instead of adding just a single item or isolated mechanic, it’s better if the contribution:

* Expands an existing system, or
* Introduces a small ecosystem around a new or existing feature

This helps the game feel more cohesive and complete.

---

## Commit Message Guidelines

Please try to keep commit messages clean and consistent.

### Preferred Format

```
<type>: <short description>
```

### Examples

```
doc: added link
fix: fixed memory leak
feat: basic chunk culling
refactor: simplified mesh generation
```

### Notes

* Start the commit message with a common commit type
* Follow it with a short explanation of what changed
* Keep it brief and readable
* Details and reasoning belong in the pull request, not the commit title

This isn’t super strict, but following this format makes the history easier to read and work with.

### Common Commit Types

* `feat:` - New features
* `fix:` - Bug fixes
* `refactor:` - Code cleanups and restructuring
* `perf:` - Performance improvements
* `doc:` - Documentation changes
* `style:` - Formatting-only changes
* `chore:` - Maintenance or tooling changes
* `test:` - Adding or fixing tests
* `build:` - Changes to build system or external dependencies
* `ci:` - Continuous Integration changes (GitLab Actions)
* `revert:` - Reverting a previous commit

If your commit message doesn’t perfectly match this, that’s fine, it’s just a guideline, not a hard rule.

---

## Legal requirements

By submitting a pull request, you represent and warrant that:
- Your contribution is your original work or you have the necessary rights to submit it.
- It does not infringe any third-party copyrights, trademarks, or other intellectual property rights.
- You grant the project the right to use, modify, and distribute your contribution under the project's license.

---

## What to contribute?

You can check the [Issues tab](https://gitlab.com/Stoniye/brickbyte/-/issues) to see what needs to be done.

Issues are categorized by **difficulty** using the following tags:

* **Difficulty/Byte**: An issue or feature that requires very little work to fix or implement.
* **Difficulty/Intermediate**: An issue or feature that requires some work and basic knowledge to fix or implement.
* **Difficulty/Hard**: An issue or feature that requires significant work and solid knowledge to fix or implement.
* **Difficulty/VeryHard**: An issue or feature that requires a large amount of work and advanced knowledge to fix or implement.
* **Difficulty/MajorRefactor**: An issue or feature that requires major refactoring to fix or implement.

Issues are also categorized by **priority (importance)** using these tags:

* **Priority/Small**: A very minor issue that occurs very rarely, or only affects a few users, and does not lower gameplay quality.
* **Priority/Intermediate**: An issue that occurs occasionally, or affects a larger group of users, and may or may not lower gameplay quality.
* **Priority/Big**: An issue that occurs often, or affects most or all users, and clearly lowers gameplay quality.
* **Priority/Major**: An issue that has a huge impact on gameplay quality or makes the game partially or completely unplayable.

Additionally, some issues may be marked as:

* **GoodFirstIssue**: Beginner-friendly issues that are well-defined and suitable for new contributors.

---

### New features and suggestions

If you want to implement a **new feature**, you can look at the [vision for Brickbyte](https://gitlab.com/Stoniye/brickbyte#vision) to come up with ideas. You can also browse the following **type tags** in the [Issues tab](https://gitlab.com/Stoniye/brickbyte/-/issues):

* **Type/Mob**: Suggests a new mob
* **Type/Item**: Suggests a new item
* **Type/Block**: Suggests a new block
* **Type/Structure**: Suggests a new structure
* **Type/Biome**: Suggests a new biome
* **Type/Gameplay**: Suggests a new gameplay mechanic

> **IMPORTANT:** New features must always follow the **“Fits the game”** requirement described in the [Contributing Guidelines](https://gitlab.com/Stoniye/brickbyte/-/blob/main/CONTRIBUTING.md#1-fits-the-game) in order to be merged into the main branch.

---

## Final Notes

Even if your pull request isn’t merged, constructive discussion is always welcome.
Feedback will be given, and improvements can always be made.

Thanks for contributing to Brickbyte
