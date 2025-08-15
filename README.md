# git-author-rewrite

A small Rust CLI tool to rewrite all commit authors in a Git repository from the root.

It:

* Prompts you for a new author **name** and **email** (with current repo defaults prefilled)
* Sets the **local** Git `user.name` and `user.email`
* Starts an interactive rebase from the **first commit (root)**
* **Automatically** marks every commit as `edit`
* Amends each commit to use the new author info
* Continues the rebase until finished

Useful for fixing commit author info in a repository’s history without editing each commit manually.

---

## Features

* **Auto-mark all commits**: No need to open an editor; all `pick` lines become `edit` automatically.
* **Manual mode**: Use `--manual` to edit the rebase todo list yourself.
* **Safe**: Explicit error handling, clear success/failure messages.
* **Cross-platform**: Works anywhere `git` is available in `PATH`.

---

## Installation

### From source

```sh
git clone https://github.com/yourusername/git-author-rewrite.git
cd git-author-rewrite
cargo install --path .
```

---

## Usage

```sh
git-author-rewrite
```

Manual mode:

```sh
git-author-rewrite --manual
```

