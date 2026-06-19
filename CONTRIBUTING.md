# Contributing to weatheroz & TUI

First off, thank you for taking the time to contribute! Contributions from the community help make this tool faster, more robust, and more delightful to use.

All contributors are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## 🛠️ Development Setup

To build and test the project locally, you need the standard Rust toolchain:

1. **Install Rust** (v1.75 or later is recommended):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Clone the repository**:
   ```bash
   git clone https://github.com/ozkayhan/weatheroz.git
   cd weatheroz
   ```
3. **Verify the installation**:
   ```bash
   cargo check
   cargo test
   ```

---

## 🌿 Branching Strategy

Our branching workflow is designed to keep our main branch stable:

1. Always create a feature branch off of the stable branch:
   - For new features: `feat/your-feature-name`
   - For bug fixes: `fix/your-bug-name`
   - For documentation updates: `docs/your-doc-update`
   - For refactoring: `refactor/your-refactor`
2. Push your branch to your remote and open a Pull Request (PR).

---

## ✍️ Coding Guidelines & Code Quality

To maintain clean, readable, and solid code (KISS, DRY, SOLID):

- **Formatter**: Ensure all code is formatted using standard `cargo fmt` before staging:
  ```bash
  cargo fmt --all -- --check
  ```
- **Linter**: Solve all lints and warnings reported by `cargo clippy`:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- **Design Principles**: Keep functions small and focused on single responsibilities (Separation of Concerns). Avoid adding hardcoded secrets, configurations, or paths.

---

## 🧪 Testing Requirements

We practice thorough testing:

- **Unit Tests**: Place unit tests in the same files as the logic (inside a `mod tests` module).
- **Integration Tests**: Place system integration and mock API cache tests in the `tests/` directory.
- **Run the test suite**:
  ```bash
  cargo test
  ```
  Every new feature or bug fix is expected to include matching test cases that prove the changes are effective and correct.

---

## 📝 Commit Conventions

We strictly follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

Structure of a commit message:
```text
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

* **feat**: A new feature (e.g. `feat(providers): add PirateWeather API support`)
* **fix**: A bug fix (e.g. `fix(cache): solve geocoding expiration checks`)
* **docs**: Documentation updates (e.g. `docs(readme): add installation details`)
* **style**: Code styling adjustments (no logical changes, e.g. `style: format imports`)
* **refactor**: Code restructuring without changing functional behavior (e.g. `refactor(tui): split renderer module`)
* **test**: Adding or fixing tests (e.g. `test(integration): cover mixed date queries`)
* **chore**: Build processes, dependency updates, metadata updates (e.g. `chore(deps): upgrade ratatui to v0.26`)

---

## 🚀 Pull Request Process

1. Fill out the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md) completely.
2. Link any related issues in the PR description (e.g., `Fixes #42`).
3. Ensure the automated CI builds successfully and all tests pass.
4. Keep PR scopes focused. If you are proposing multiple unrelated changes, please split them into separate PRs.
