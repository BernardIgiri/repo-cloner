# Github Repo Cloner

A simple CLI tool for cloning Git repositories into a predefined directory structure, organizing repositories by domain and author.

## Features

- Clones Git repositories to a structured folder hierarchy: `base_path/domain/author/repo_name`.
- Supports both real and dry-run execution modes.
- Ensures that necessary directories exist before cloning.
- Allows specifying a custom base path (defaults to the current working directory).

## Installation

You can install the tool using `cargo`:

```bash
git clone https://github.com/BernardIgiri/repo-cloner.git
cd repo-cloner
cargo install --path .
```

## Usage

Run the tool with:

```bash
repo-cloner [git-url] [--base-path <path>] [--dry-run]
```

### Arguments

- `git-url` *(required)* – The URL of the Git repository to clone.
- `--base-path` *(optional)* – The directory where repositories should be cloned (defaults to the current working directory).
- `--dry-run` *(optional)* – Setting this prints the commands instead of executing them.

### Example Usages

#### Clone a repository to the default location:

```bash
repo-cloner https://github.com/example-user/example-repo.git
```

This clones `example-repo` into `./github.com/example-user/example-repo`.

#### Clone a repository to a custom base path:

```bash
repo-cloner https://github.com/example-user/example-repo.git --base-path /home/user/repos
```

This clones `example-repo` into `/home/user/repos/github.com/example-user/example-repo`.

#### Perform a dry run:

```bash
repo-cloner https://github.com/example-user/example-repo.git --dry-run
```

This prints the commands that would be executed without actually cloning the repository.

## How It Works

1. Parses the Git URL to extract the domain, author, and repository name.
2. Constructs the destination path in the format:
   ```
   base_path/domain/author/repo_name
   ```
3. Ensures the directory structure exists.
4. Clones the repository into the structured location.
5. Prints the success message or the dry-run equivalent.

## Running Tests

You can run the included tests using:

```bash
cargo test
```
