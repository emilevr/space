use anyhow::{bail, Context};
use clap::Args;
use git2::{Oid, Repository};
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{debug, info, trace, warn};
use pathdiff::diff_paths;
use rayon::prelude::*;
use semver::Version;
use std::io::Write;
use std::process::Command;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    env,
    fs::OpenOptions,
    path::PathBuf,
};
use std::{fs, path};
use toml_edit::{value, Document};

use crate::command::BuildItCommand;

const CALCULATED_VERSION_KEY: &str = "CALCULATED_VERSION";
const PREV_RELEASE_VERSION_KEY: &str = "PREV_RELEASE_VERSION";
const IS_PRE_RELEASE_KEY: &str = "IS_PRE_RELEASE";

#[derive(Args, Debug)]
pub struct VersionCommandArgs {
    /// A path to the Git repo root directory. If not specified the current directory will be used.
    #[arg(short = 'r', long = "repo-root-dir")]
    pub repo_root_dir: Option<String>,

    /// An optional version tag prefix.
    #[arg(short = 'p', long = "version-tag-prefix")]
    pub version_tag_prefix: Option<String>,

    /// The maximum number of commits to process.
    #[arg(long = "max-commit-count", default_value_t = 1000)]
    pub max_commit_count: usize,

    /// The name of the release branch, i.e. the branch from which releases are done.
    #[arg(long = "release-branch-name", default_value_t = String::from("main"))]
    pub release_branch_name: String,

    /// When this option is provided, the branch name will not be used as the pre-release name when the
    /// current branch is not the release branch, but the value specified by the pre-release-name argument
    /// will be used instead.
    #[arg(long = "no-branch-name")]
    pub no_branch_name: bool,

    /// The default pre-release name used when on the release branch or when no-branch-name is specified.
    #[arg(long = "pre-release-name", default_value_t = String::from("alpha"))]
    pub pre_release_name: String,

    /// One or more glob patterns used to find Cargo.toml manifests, which will be updated to the calculated
    /// version.
    #[arg(
        short = 'm',
        long = "manifest-globs",
        default_value = "**/Cargo.toml",
        value_delimiter = ' '
    )]
    pub manifest_globs: Vec<String>,
}

pub struct VersionCommand {
    args: VersionCommandArgs,
}

#[derive(Clone, Debug)]
struct ReleaseVersion {
    version: Version,
    commit_id: Oid,
}

trait PipelineService {
    fn detect(&self) -> bool;
    fn name(&self) -> &str;
    fn host_is_build_agent(&self) -> bool;
    fn set_env_var(&self, key: &str, value: &str) -> anyhow::Result<()>;
    fn set_pipeline_var(&self, name: &str, value: &str) -> anyhow::Result<()>;
    fn set_pipeline_version(&self, version: &Version) -> anyhow::Result<()>;
}

struct GitHubActions {}

struct LocalPipelineService {}

impl VersionCommand {
    pub fn new(args: VersionCommandArgs) -> Self {
        VersionCommand { args }
    }
}

impl BuildItCommand for VersionCommand {
    fn run(&mut self) -> anyhow::Result<()> {
        let pipeline_service = detect_pipeline_service();

        let repo_root_path = get_repo_root_path(&self.args.repo_root_dir)?;
        let repo = open_git_repo(&repo_root_path)?;

        let commit_id_to_tags = get_commit_id_to_tag_map(&repo)?;
        let commits = get_commits_desc_by_date(&repo, self.args.max_commit_count)?;

        let (release_versions, depth) =
            get_release_versions(&commits, &commit_id_to_tags, &self.args.version_tag_prefix);

        let branch_name = get_branch_name(&repo)?;
        let branch_component = get_prerelease_branch_component(&branch_name);

        let is_release_branch = is_release_branch(&branch_name, &self.args.release_branch_name);

        let current_release_version = release_versions.last();
        let last_commit_was_release = if let Some(last_version) = current_release_version {
            last_version.commit_id == repo.head()?.peel_to_commit()?.id()
        } else {
            false
        };
        info!("Was the last commit a release? {last_commit_was_release}");

        let pre_release_name = &self.args.pre_release_name;
        let no_branch_name = self.args.no_branch_name;

        let mut prev_release_version = if release_versions.len() > 1 {
            release_versions[release_versions.len() - 2]
                .version
                .to_string()
        } else {
            String::default()
        };

        let calculated_version = if last_commit_was_release {
            // We know current_release_version has value if last_commit_was_release is true.
            current_release_version.unwrap().version.clone()
        } else if current_release_version.is_some() {
            let next_pre_release_name = if is_release_branch || no_branch_name {
                pre_release_name.clone()
            } else {
                branch_component
            };

            let mut last_release_version = (*release_versions.last().unwrap()).clone();

            // Since depth > 0 it means the last release version is the previous version where we would want
            // to start when compiling release notes / changelog.
            prev_release_version = last_release_version.version.to_string();

            // Increment one of the version components. Once conventional commits are supported the commit
            // message prefixes since the last release version would determine which components are
            // incremented.
            last_release_version.version.patch += 1;
            let next_version_number = last_release_version.version.to_string();

            let mut calculated_version = format!("{next_version_number}-{next_pre_release_name}");
            if depth > 0 {
                calculated_version.push_str(&format!(".{depth}"));
            }

            Version::parse(&calculated_version)
                .context("The calculated next version was not SemVer2 compliant!")?
        } else {
            Version::parse("0.0.0").unwrap()
        };

        let calculated_version = if let Some(version_tag_prefix) = &self.args.version_tag_prefix {
            format!("{version_tag_prefix}{calculated_version}")
        } else {
            calculated_version.to_string()
        };

        let prev_release_version = if !prev_release_version.is_empty() {
            if let Some(version_tag_prefix) = &self.args.version_tag_prefix {
                format!("{version_tag_prefix}{prev_release_version}")
            } else {
                prev_release_version
            }
        } else {
            prev_release_version
        };

        info!("The calculated version is {calculated_version}",);
        if prev_release_version.is_empty() {
            info!("No previous release was found.");
        } else {
            info!("The previous release version was: {prev_release_version}");
        }

        set_pipeline_service_vars(
            pipeline_service.as_ref(),
            &calculated_version,
            &prev_release_version,
            !last_commit_was_release,
        )?;

        if pipeline_service.host_is_build_agent() {
            update_manifests(
                &repo_root_path,
                &self.args.manifest_globs,
                &calculated_version,
            )?;
        } else {
            info!("Not updating the version in any package manifests, as the host was not detected as a build agent.");
        }

        Ok(())
    }
}

fn update_manifests(
    repo_root_path: &PathBuf,
    manifest_globs: &[String],
    calculated_version: &str,
) -> anyhow::Result<()> {
    let calculated_version = calculated_version.to_string();

    let mut builder = GlobSetBuilder::new();
    manifest_globs
        .iter()
        .try_for_each(|pattern| {
            trace!("About to parse pattern {pattern} as glob");
            builder.add(Glob::new(pattern)?);
            anyhow::Ok(())
        })
        .context("Failed to parse glob pattern!")?;
    let set = builder.build()?;

    let paths = glob(repo_root_path, &set);

    info!("Found {} manifest files to update.", paths.len());

    paths
        .iter()
        .try_for_each(|path| {
            let read_to_string = fs::read_to_string(path)?;
            let mut document: Document = read_to_string.parse::<Document>()?;
            if document.contains_key("package") {
                let package_version = &document["package"]["version"];
                let package_version = package_version.to_string();
                let package_version = package_version.trim().trim_matches('"');
                if package_version == "0.0.0" {
                    debug!("About to update package version to {calculated_version} in manifest file {}", path.display());
                    document["package"]["version"] = value(calculated_version.to_string());
                    fs::write(path, document.to_string())?;
                    info!("Updated package version to {calculated_version} in manifest file {}", path.display());
                } else {
                    warn!("The package version in manifest file {} is {}. Only package versions that are set to 0.0.0 are updated. NOT updating.", path.display(), package_version);
                }
            } else {
                warn!("Did not find a [package] key in manifest file {}. Skipping this file.", path.display());
            }

            anyhow::Ok(())
        })
        .context("Failed to update a manifest file!")?;

    Ok(())
}

fn glob(root_path: &PathBuf, set: &GlobSet) -> Vec<PathBuf> {
    let mut paths = glob_recurse(root_path, root_path, set);
    paths.sort();
    paths.dedup();
    paths
}

fn glob_recurse(repo_root_path: &PathBuf, path: &PathBuf, set: &GlobSet) -> Vec<PathBuf> {
    if path.is_dir() {
        fs::read_dir(path)
            .into_iter()
            .flatten()
            .par_bridge()
            .filter_map(|result| result.ok())
            .flat_map(|entry| glob_recurse(repo_root_path, &entry.path(), set))
            .collect()
    } else {
        let path = diff_paths(path, repo_root_path).unwrap();
        let path = path
            .to_string_lossy()
            .replace(path::MAIN_SEPARATOR_STR, "/");

        if !set.matches(&path).is_empty() {
            vec![PathBuf::from(path)]
        } else {
            vec![]
        }
    }
}

fn get_repo_root_path(repo_root_dir: &Option<String>) -> Result<PathBuf, anyhow::Error> {
    let repo_root_path = if let Some(repo_root_dir) = repo_root_dir {
        PathBuf::from(repo_root_dir)
    } else {
        env::current_dir()?
    };
    Ok(repo_root_path)
}

fn open_git_repo(repo_root_path: &PathBuf) -> anyhow::Result<Repository> {
    let repo = match Repository::open(repo_root_path) {
        Ok(repo) => repo,
        Err(e) => bail!(
            "Could not open the Git repository at {}: {e}",
            &repo_root_path.display()
        ),
    };
    Ok(repo)
}

fn get_commits_desc_by_date(
    repo: &Repository,
    max_count: usize,
) -> Result<Vec<git2::Commit<'_>>, anyhow::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;

    Ok(revwalk
        .filter_map(|id| {
            let id = id.unwrap();
            if let Ok(commit) = repo.find_commit(id) {
                Some(commit)
            } else {
                None
            }
        })
        .take(max_count)
        .collect())
}

fn get_release_versions(
    commits: &[git2::Commit<'_>],
    commit_id_to_tags: &BTreeMap<Oid, HashSet<String>>,
    version_tag_prefix: &Option<String>,
) -> (Vec<ReleaseVersion>, i32) {
    let mut depth = 0;
    let release_versions: Vec<_> = commits
        .iter()
        .rev()
        .filter_map(|commit| {
            let commit_id = commit.id();
            if let Some(tags) = commit_id_to_tags.get(&commit_id) {
                // Get all the semver compliant, non-pre-release tags for this commit.
                let mut semver_tags: Vec<_> = tags
                    .iter()
                    .filter_map(|tag| {
                        let tag = if let Some(version_tag_prefix) = version_tag_prefix {
                            tag.strip_prefix(version_tag_prefix)?
                        } else {
                            tag
                        };

                        if let Ok(version) = Version::parse(tag) {
                            if version.pre.is_empty() {
                                return Some(version);
                            }
                        }
                        None
                    })
                    .collect();

                match semver_tags.len() {
                    0 => {}
                    1 => {
                        depth = 0;
                        return Some(ReleaseVersion {
                            version: semver_tags[0].clone(),
                            commit_id,
                        });
                    }
                    _ => {
                        depth = 0;
                        semver_tags.sort();
                        semver_tags.reverse();
                        return Some(ReleaseVersion {
                            version: semver_tags.first().unwrap().clone(),
                            commit_id,
                        });
                    }
                };
            }

            depth += 1;
            None
        })
        .collect();

    info!(
        "Detected {} release version(s) and final depth (commits since previous release version) was {depth}",
        release_versions.len()
    );

    (release_versions, depth)
}

fn is_release_branch(branch_name: &str, release_branch_name: &str) -> bool {
    info!("Configured release branch name is {}", release_branch_name);
    let current_is_release_branch = branch_name == release_branch_name;
    info!("Current branch is release branch? {current_is_release_branch}");
    current_is_release_branch
}

fn get_branch_name(repo: &Repository) -> anyhow::Result<String> {
    let branch_name = repo.head()?.name().unwrap().to_string();
    if let Some(clean_branch_name) = branch_name.strip_prefix("refs/heads/") {
        Ok(clean_branch_name.to_string())
    } else {
        Ok(branch_name)
    }
}

fn get_prerelease_branch_component(branch_name: &str) -> String {
    let branch_name = branch_name.replace('/', ".");
    debug!("Branch name component for potential pre-release version: {branch_name}");
    branch_name
}

fn set_pipeline_service_vars(
    pipeline_service: &dyn PipelineService,
    calculated_version: &str,
    prev_release_version: &str,
    is_pre_release: bool,
) -> anyhow::Result<()> {
    let is_pre_release = is_pre_release.to_string();

    pipeline_service.set_env_var(CALCULATED_VERSION_KEY, calculated_version)?;
    pipeline_service.set_env_var(PREV_RELEASE_VERSION_KEY, prev_release_version)?;
    pipeline_service.set_env_var(IS_PRE_RELEASE_KEY, &is_pre_release)?;

    pipeline_service.set_pipeline_var(CALCULATED_VERSION_KEY, calculated_version)?;
    pipeline_service.set_pipeline_var(PREV_RELEASE_VERSION_KEY, prev_release_version)?;
    pipeline_service.set_pipeline_var(IS_PRE_RELEASE_KEY, &is_pre_release)?;

    Ok(())
}

fn get_commit_id_to_tag_map(
    repo: &Repository,
) -> anyhow::Result<BTreeMap<git2::Oid, HashSet<String>>> {
    let tag_names = repo.tag_names(None)?;
    let mut commit_id_to_tags: BTreeMap<git2::Oid, HashSet<String>> = BTreeMap::new();
    for tag in tag_names.into_iter() {
        let tag = tag.unwrap();
        let obj = repo.revparse_single(tag)?;
        if let Some(commit) = obj.as_commit() {
            let commit_id = commit.id();
            if let Some(value) = commit_id_to_tags.get_mut(&commit_id) {
                value.insert(tag.to_string());
            } else {
                let mut tags = HashSet::new();
                tags.insert(tag.to_string());
                commit_id_to_tags.insert(commit_id, tags);
            };
        }
    }
    Ok(commit_id_to_tags)
}

fn detect_pipeline_service() -> Box<dyn PipelineService> {
    let mut services: VecDeque<_> = vec![Box::new(GitHubActions {})]
        .into_iter()
        .filter(|service| service.detect())
        .collect();

    let service = if services.is_empty() {
        Box::new(LocalPipelineService {})
    } else {
        services.pop_front().unwrap() as Box<dyn PipelineService>
    };

    info!("Detected pipeline service: {}", service.name());

    service
}

impl PipelineService for GitHubActions {
    fn detect(&self) -> bool {
        env::var_os("GITHUB_ACTIONS").is_some()
    }

    fn name(&self) -> &str {
        "GitHub Actions"
    }

    fn host_is_build_agent(&self) -> bool {
        true
    }

    fn set_pipeline_version(&self, _: &Version) -> anyhow::Result<()> {
        // Not supported by GitHub Actions
        Ok(())
    }

    fn set_env_var(&self, key: &str, value: &str) -> anyhow::Result<()> {
        append_to_github_pipeline_file("GITHUB_ENV", key, value)?;
        append_to_github_pipeline_file("GITHUB_OUTPUT", key, value)?;
        Ok(())
    }

    fn set_pipeline_var(&self, name: &str, value: &str) -> anyhow::Result<()> {
        let output = Command::new("echo")
            .arg(format!("::set-env name={}::{}", name, value))
            .output()
            .context("Failed to set pipeline variable!")?;

        if !output.status.success() {
            bail!("Unable to set pipeline variable named {name}!");
        }

        Ok(())
    }
}

fn append_to_github_pipeline_file(
    file_env_var: &str,
    key: &str,
    value: &str,
) -> anyhow::Result<()> {
    let github_env_file_path = env::var(file_env_var).unwrap();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(github_env_file_path)
        .context(format!("Failed to open ${file_env_var} file!"))?;
    writeln!(&mut file, "\n{}={}", key, value)
        .context(format!("Failed to write to ${file_env_var} file!"))?;
    file.flush()?;
    Ok(())
}

impl PipelineService for LocalPipelineService {
    fn detect(&self) -> bool {
        // Cannot be reliably detected so always return false.
        false
    }

    fn name(&self) -> &str {
        "local or unsupported"
    }

    fn host_is_build_agent(&self) -> bool {
        false
    }

    fn set_pipeline_version(&self, _: &Version) -> anyhow::Result<()> {
        // Nothing to do.
        Ok(())
    }

    fn set_env_var(&self, key: &str, value: &str) -> anyhow::Result<()> {
        env::set_var(key, value);
        Ok(())
    }

    fn set_pipeline_var(&self, _: &str, _: &str) -> anyhow::Result<()> {
        // Nothing to do
        Ok(())
    }
}
