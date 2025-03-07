use clap::Parser;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use url::Url;

/// A simple CLI tool to clone git repositories to a specific directory structure.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL of the git repository to clone
    git_url: String,

    /// Optional base path where the repository should be cloned (defaults to PWD)
    #[arg(short, long)]
    base_path: Option<String>,

    /// Perform a dry run (print the commands without executing them)
    #[arg(long)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();
    let base_path = args.base_path.unwrap_or_else(|| {
        env::current_dir()
            .expect("Failed to get current directory")
            .to_string_lossy()
            .to_string()
    });

    if args.dry_run {
        let cloner = RepoCloner::new(DryRunRepoCommands);
        cloner.run(&args.git_url, &base_path);
    } else {
        let cloner = RepoCloner::new(SystemRepoCommands);
        cloner.run(&args.git_url, &base_path);
    };
}

trait RepoCommands {
    fn git_clone(&self, url: &str, clone_path: &Path);
    fn cd_destination(&self, clone_path: &Path);
    fn display_success(&self);
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
}

struct SystemRepoCommands;

impl RepoCommands for SystemRepoCommands {
    fn git_clone(&self, url: &str, clone_path: &Path) {
        Command::new("git")
            .arg("clone")
            .arg(url)
            .arg(clone_path)
            .status()
            .expect("Failed to clone repository");
    }

    fn cd_destination(&self, clone_path: &Path) {
        println!("cd {}", clone_path.to_string_lossy());
    }

    fn display_success(&self) {
        println!("Repository cloned successfully.");
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }
}

struct DryRunRepoCommands;

impl RepoCommands for DryRunRepoCommands {
    fn git_clone(&self, url: &str, clone_path: &Path) {
        println!("DRY RUN: git clone {} {}", url, clone_path.display());
    }

    fn cd_destination(&self, clone_path: &Path) {
        println!("DRY RUN: cd {}", clone_path.display());
    }

    fn display_success(&self) {
        println!("DRY RUN: Repository cloned successfully.");
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        println!("DRY RUN: mkdir -p {}", path.display());
        Ok(())
    }
}

struct RepoCloner<C: RepoCommands> {
    commands: C,
}

impl<C: RepoCommands> RepoCloner<C> {
    fn new(commands: C) -> Self {
        RepoCloner { commands }
    }

    fn run(&self, git_url: &str, base_path: &str) {
        if let Some((domain, author, project)) = self.parse_git_url(git_url) {
            let clone_dir = self.create_directory_structure(base_path, &domain, &author);
            let project_path = clone_dir.join(project);

            self.commands.git_clone(git_url, &project_path);
            self.commands.cd_destination(&project_path);
            self.commands.display_success();
        } else {
            eprintln!("Failed to parse the git URL.");
        }
    }

    fn parse_git_url(&self, git_url: &str) -> Option<(String, String, String)> {
        let parsed_url = Url::parse(git_url).ok()?;
        let domain = parsed_url.host_str()?.to_string();
        let mut path_segments = parsed_url.path_segments()?;
        let author = path_segments.next()?.to_string();
        let project = path_segments.next()?.to_string().replace(".git", "");
        Some((domain, author, project))
    }

    fn create_directory_structure(&self, base_path: &str, domain: &str, author: &str) -> PathBuf {
        let path = PathBuf::from(base_path).join(domain).join(author);
        self.commands
            .create_dir_all(&path)
            .expect("Failed to create directories");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockRepoCommands {
        pub cloned_repos: RefCell<Vec<(String, PathBuf)>>,
        pub navigated_paths: RefCell<Vec<PathBuf>>,
        pub success: RefCell<bool>,
        pub created_paths: RefCell<Vec<PathBuf>>,
    }

    impl RepoCommands for MockRepoCommands {
        fn git_clone(&self, url: &str, clone_path: &Path) {
            self.cloned_repos
                .borrow_mut()
                .push((url.to_string(), clone_path.to_path_buf()));
        }

        fn cd_destination(&self, clone_path: &Path) {
            self.navigated_paths
                .borrow_mut()
                .push(clone_path.to_path_buf());
        }

        fn display_success(&self) {
            self.success.replace_with(|_| true);
        }

        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            self.created_paths.borrow_mut().push(path.to_path_buf());
            Ok(())
        }
    }

    impl MockRepoCommands {
        pub fn new() -> Self {
            Self {
                cloned_repos: RefCell::new(vec![]),
                navigated_paths: RefCell::new(vec![]),
                success: RefCell::new(false),
                created_paths: RefCell::new(vec![]),
            }
        }
    }

    #[test]
    fn test_clone_repo() {
        let mock_commands = MockRepoCommands::new();
        let cloner = RepoCloner::new(mock_commands);
        cloner.run("https://github.com/author/project.git", "/base/path");

        let cloned_repos = cloner.commands.cloned_repos.borrow();
        assert_eq!(cloned_repos.len(), 1);
        assert_eq!(cloned_repos[0].0, "https://github.com/author/project.git");
        assert_eq!(
            cloned_repos[0].1,
            PathBuf::from("/base/path/github.com/author/project")
        );

        let navigated_paths = cloner.commands.navigated_paths.borrow();
        assert_eq!(navigated_paths.len(), 1);
        assert_eq!(
            navigated_paths[0],
            PathBuf::from("/base/path/github.com/author/project")
        );

        let success = cloner.commands.success.take();
        assert!(success);
    }

    #[test]
    fn test_clone_libjpeg_turbo() {
        let mock_commands = MockRepoCommands::new();
        let cloner = RepoCloner::new(mock_commands);
        cloner.run(
            "https://github.com/libjpeg-turbo/libjpeg-turbo.git",
            "/base/path",
        );

        let cloned_repos = cloner.commands.cloned_repos.borrow();
        assert_eq!(cloned_repos.len(), 1);
        assert_eq!(
            cloned_repos[0].0,
            "https://github.com/libjpeg-turbo/libjpeg-turbo.git"
        );
        assert_eq!(
            cloned_repos[0].1,
            PathBuf::from("/base/path/github.com/libjpeg-turbo/libjpeg-turbo")
        );

        let navigated_paths = cloner.commands.navigated_paths.borrow();
        assert_eq!(navigated_paths.len(), 1);
        assert_eq!(
            navigated_paths[0],
            PathBuf::from("/base/path/github.com/libjpeg-turbo/libjpeg-turbo")
        );

        let success = cloner.commands.success.take();
        assert!(success);
    }

    #[test]
    fn test_clone_gitlab() {
        let mock_commands = MockRepoCommands::new();
        let cloner = RepoCloner::new(mock_commands);
        cloner.run(
            "https://gitlab.com/emeraldjayde/gitlab-vscode-extension.git",
            "/base/path",
        );

        let cloned_repos = cloner.commands.cloned_repos.borrow();
        assert_eq!(cloned_repos.len(), 1);
        assert_eq!(
            cloned_repos[0].0,
            "https://gitlab.com/emeraldjayde/gitlab-vscode-extension.git"
        );
        assert_eq!(
            cloned_repos[0].1,
            PathBuf::from("/base/path/gitlab.com/emeraldjayde/gitlab-vscode-extension")
        );

        let navigated_paths = cloner.commands.navigated_paths.borrow();
        assert_eq!(navigated_paths.len(), 1);
        assert_eq!(
            navigated_paths[0],
            PathBuf::from("/base/path/gitlab.com/emeraldjayde/gitlab-vscode-extension")
        );

        let success = cloner.commands.success.take();
        assert!(success);
    }
}
