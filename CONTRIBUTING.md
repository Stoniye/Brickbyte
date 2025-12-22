# Contributing to Brickbyte

Everyone is welcome to contribute to **Brickbyte**.

I review pull requests based on my own judgment. There is no fixed formula or checklist that guarantees a merge. If I decide not to merge a pull request, I will **always explain why**, I won’t silently close it.

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

## Final Notes

Even if your pull request isn’t merged, constructive discussion is always welcome.
Feedback will be given, and improvements can always be made.

Thanks for contributing to Brickbyte