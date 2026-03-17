use crate::cli::SelfUpdateArgs;
use flate2::read::GzDecoder;
use fpt_core::{AppError, Result};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use semver::Version;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::{self, Write};

use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::tempdir;
use zip::ZipArchive;

const DEFAULT_REPOSITORY: &str = "loonghao/fpt-cli";
const CHECKSUM_ASSET_NAME: &str = "fpt-checksums.txt";
const SUPPORTED_TARGETS: &[&str] = &[
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

#[derive(Debug, Clone, Copy)]
enum ArchiveKind {
    TarGz,
    Zip,
}

#[derive(Debug, Clone, Copy)]
struct ReleaseTarget {
    triple: &'static str,
    archive_kind: ArchiveKind,
    archive_extension: &'static str,
    binary_name: &'static str,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    published_at: Option<String>,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub async fn run(args: SelfUpdateArgs) -> Result<Value> {
    let repository = args
        .repository
        .clone()
        .or_else(|| env::var("FPT_UPDATE_REPOSITORY").ok())
        .unwrap_or_else(|| DEFAULT_REPOSITORY.to_string());
    let (owner, repo) = split_repository(&repository)?;
    let target = detect_target()?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| {
        AppError::internal(format!(
            "could not parse the current CLI version from build metadata: {error}"
        ))
    })?;
    let current_exe = env::current_exe().map_err(|error| {
        AppError::internal(format!(
            "could not resolve the current executable path: {error}"
        ))
    })?;
    let requested_version = args.version.clone().map(normalize_version);
    let client = build_http_client()?;
    let release = fetch_release(&client, &owner, &repo, requested_version.as_deref()).await?;
    let release_version = parse_release_version(&release.tag_name)?;
    let asset = find_release_asset(&release, target, &release_version)?;
    let update_available = release_version > current_version;

    if args.check {
        return Ok(json!({
            "command": "self.update",
            "status": if update_available { "update_available" } else { "already_latest" },
            "repository": repository,
            "requested_version": requested_version,
            "current_version": current_version.to_string(),
            "available_version": release_version.to_string(),
            "tag_name": release.tag_name,
            "release_url": release.html_url,
            "published_at": release.published_at,
            "target": target.triple,
            "asset_name": asset.name,
            "download_url": asset.browser_download_url,
            "install_path": current_exe,
            "update_available": update_available,
        }));
    }

    if !update_available && requested_version.is_none() {
        return Ok(json!({
            "command": "self.update",
            "status": "already_latest",
            "repository": repository,
            "current_version": current_version.to_string(),
            "available_version": release_version.to_string(),
            "tag_name": release.tag_name,
            "target": target.triple,
            "asset_name": asset.name,
            "install_path": current_exe,
            "update_available": false,
        }));
    }

    let checksum_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == CHECKSUM_ASSET_NAME);
    let temp_dir = tempdir().map_err(|error| {
        AppError::internal(format!(
            "could not create a temporary directory for self-update: {error}"
        ))
    })?;
    let archive_path = temp_dir.path().join(&asset.name);
    let archive_bytes = download_bytes(&client, &asset.browser_download_url).await?;
    write_bytes(&archive_path, &archive_bytes)?;

    let checksum_verified = if let Some(checksum_asset) = checksum_asset {
        let checksums = download_text(&client, &checksum_asset.browser_download_url).await?;
        verify_checksum(&checksums, &asset.name, &archive_bytes)?;
        true
    } else {
        false
    };

    let extracted_binary = extract_binary(&archive_path, target, temp_dir.path())?;
    self_replace::self_replace(&extracted_binary).map_err(|error| {
        AppError::internal(format!(
            "could not replace the current executable during self-update: {error}"
        ))
    })?;

    Ok(json!({
        "command": "self.update",
        "status": if release_version > current_version { "updated" } else { "reinstalled" },
        "repository": repository,
        "requested_version": requested_version,
        "previous_version": current_version.to_string(),
        "current_version": release_version.to_string(),
        "tag_name": release.tag_name,
        "release_url": release.html_url,
        "published_at": release.published_at,
        "target": target.triple,
        "asset_name": asset.name,
        "download_url": asset.browser_download_url,
        "install_path": current_exe,
        "checksum_verified": checksum_verified,
    }))
}

fn split_repository(repository: &str) -> Result<(String, String)> {
    let (owner, repo) = repository.split_once('/').ok_or_else(|| {
        AppError::invalid_input("repository override must use the format `owner/repo`")
            .with_operation("split_repository")
            .with_invalid_field("repository")
            .with_received_value(repository)
            .with_expected_shape("`owner/repo`, for example `loonghao/fpt-cli`")
    })?;

    if owner.is_empty() || repo.is_empty() {
        return Err(AppError::invalid_input(
            "repository override must use the format `owner/repo`",
        )
        .with_operation("split_repository")
        .with_invalid_field("repository")
        .with_received_value(repository)
        .with_expected_shape("`owner/repo`, for example `loonghao/fpt-cli`"));
    }

    Ok((owner.to_string(), repo.to_string()))
}

fn detect_target() -> Result<ReleaseTarget> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86_64") => Ok(ReleaseTarget {
            triple: "x86_64-pc-windows-msvc",
            archive_kind: ArchiveKind::Zip,
            archive_extension: "zip",
            binary_name: "fpt.exe",
        }),
        ("linux", "x86_64") => Ok(ReleaseTarget {
            triple: "x86_64-unknown-linux-gnu",
            archive_kind: ArchiveKind::TarGz,
            archive_extension: "tar.gz",
            binary_name: "fpt",
        }),
        ("macos", "x86_64") => Ok(ReleaseTarget {
            triple: "x86_64-apple-darwin",
            archive_kind: ArchiveKind::TarGz,
            archive_extension: "tar.gz",
            binary_name: "fpt",
        }),
        ("macos", "aarch64") => Ok(ReleaseTarget {
            triple: "aarch64-apple-darwin",
            archive_kind: ArchiveKind::TarGz,
            archive_extension: "tar.gz",
            binary_name: "fpt",
        }),
        _ => Err(AppError::unsupported(format!(
            "self-update is only supported for targets {}; current host is `{arch}-{os}`",
            SUPPORTED_TARGETS.join(", ")
        ))
        .with_operation("detect_target")
        .with_detail("os", os)
        .with_detail("arch", arch)
        .with_allowed_values(SUPPORTED_TARGETS.iter().copied())
        .with_hint("Download a pre-built binary for your platform from the GitHub releases page, or build from source.")),
    }
}

fn find_release_asset<'a>(
    release: &'a GitHubRelease,
    target: ReleaseTarget,
    version: &Version,
) -> Result<&'a GitHubReleaseAsset> {
    let candidates = [
        versioned_asset_name_for(target, version),
        legacy_asset_name_for(target),
    ];

    release
        .assets
        .iter()
        .find(|asset| candidates.iter().any(|candidate| asset.name == *candidate))
        .ok_or_else(|| {
            AppError::unsupported(format!(
                "release `{}` does not include a compatible asset for target `{}`; expected one of: {}",
                release.tag_name,
                target.triple,
                candidates.join(", ")
            ))
            .with_operation("find_release_asset")
            .with_detail("tag_name", &release.tag_name)
            .with_detail("target", target.triple)
            .with_allowed_values(candidates.iter().map(String::as_str))
            .with_hint("Check the GitHub releases page to confirm which assets are available for this release.")
        })
}

fn legacy_asset_name_for(target: ReleaseTarget) -> String {
    format!("fpt-{}.{}", target.triple, target.archive_extension)
}

fn versioned_asset_name_for(target: ReleaseTarget, version: &Version) -> String {
    format!(
        "fpt-v{}-{}.{}",
        version, target.triple, target.archive_extension
    )
}

fn normalize_version(version: String) -> String {
    if version.starts_with('v') {
        version
    } else {
        format!("v{version}")
    }
}

fn build_http_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&format!("fpt-cli/{}", env!("CARGO_PKG_VERSION"))).map_err(
            |error| {
                AppError::internal(format!(
                    "could not build the GitHub user-agent header: {error}"
                ))
            },
        )?,
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );

    if let Ok(token) = env::var("GITHUB_TOKEN") {
        let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|error| {
            AppError::invalid_input(format!(
                "environment variable `GITHUB_TOKEN` is not a valid HTTP header value: {error}"
            ))
            .with_operation("build_http_client")
            .with_invalid_field("GITHUB_TOKEN")
            .with_hint("Ensure `GITHUB_TOKEN` contains only ASCII characters and no whitespace.")
        })?;
        headers.insert(AUTHORIZATION, value);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|error| {
            AppError::internal(format!("could not create the GitHub HTTP client: {error}"))
        })
}

async fn fetch_release(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    requested_version: Option<&str>,
) -> Result<GitHubRelease> {
    let url = match requested_version {
        Some(version) => {
            format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{version}")
        }
        None => format!("https://api.github.com/repos/{owner}/{repo}/releases/latest"),
    };

    client
        .get(url)
        .send()
        .await
        .map_err(|error| {
            AppError::network(format!(
                "could not request GitHub release metadata: {error}"
            ))
            .with_operation("fetch_release")
            .with_transport("rest")
            .with_resource(format!("repos/{owner}/{repo}/releases"))
            .retryable(true)
        })?
        .error_for_status()
        .map_err(|error| {
            AppError::network(format!("GitHub release metadata request failed: {error}"))
                .with_operation("fetch_release")
                .with_transport("rest")
                .with_resource(format!("repos/{owner}/{repo}/releases"))
                .with_hint("Check the repository name and ensure the release tag exists.")
        })?
        .json::<GitHubRelease>()
        .await
        .map_err(|error| {
            AppError::network(format!(
                "could not decode GitHub release metadata as JSON: {error}"
            ))
            .with_operation("fetch_release")
            .with_transport("rest")
            .with_expected_shape(
                "a GitHub release JSON object with `tag_name`, `html_url`, and `assets`",
            )
        })
}

fn parse_release_version(tag_name: &str) -> Result<Version> {
    let version = tag_name.strip_prefix('v').unwrap_or(tag_name);
    Version::parse(version).map_err(|error| {
        AppError::internal(format!(
            "release tag `{}` is not a valid semantic version: {error}",
            tag_name
        ))
    })
}

async fn download_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let bytes = client
        .get(url)
        .header(ACCEPT, "application/octet-stream")
        .send()
        .await
        .map_err(|error| {
            AppError::network(format!("could not download the release asset: {error}"))
        })?
        .error_for_status()
        .map_err(|error| AppError::network(format!("release asset download failed: {error}")))?
        .bytes()
        .await
        .map_err(|error| {
            AppError::network(format!(
                "could not read the downloaded release asset body: {error}"
            ))
        })?;

    Ok(bytes.to_vec())
}

async fn download_text(client: &reqwest::Client, url: &str) -> Result<String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|error| {
            AppError::network(format!("could not download the checksum file: {error}"))
        })?
        .error_for_status()
        .map_err(|error| AppError::network(format!("checksum file download failed: {error}")))?
        .text()
        .await
        .map_err(|error| {
            AppError::network(format!("could not read the checksum file body: {error}"))
        })
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = File::create(path).map_err(|error| {
        AppError::internal(format!(
            "could not create temporary file `{}`: {error}",
            path.display()
        ))
    })?;
    file.write_all(bytes).map_err(|error| {
        AppError::internal(format!(
            "could not write temporary file `{}`: {error}",
            path.display()
        ))
    })
}

fn verify_checksum(checksums: &str, asset_name: &str, archive_bytes: &[u8]) -> Result<()> {
    let expected = checksums
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let checksum = parts.next()?;
            let name = parts.next()?.trim_start_matches('*');
            (name == asset_name).then(|| checksum.to_ascii_lowercase())
        })
        .next()
        .ok_or_else(|| {
            AppError::internal(format!(
                "checksum file `{}` does not contain an entry for asset `{}`",
                CHECKSUM_ASSET_NAME, asset_name
            ))
            .with_operation("verify_checksum")
            .with_resource(CHECKSUM_ASSET_NAME)
            .with_detail("asset_name", asset_name)
        })?;

    let actual = format!("{:x}", Sha256::digest(archive_bytes));
    if actual != expected {
        return Err(AppError::network(format!(
            "checksum verification failed for `{}`; expected `{}`, got `{}`",
            asset_name, expected, actual
        ))
        .with_operation("verify_checksum")
        .with_resource(asset_name)
        .with_detail("expected_checksum", &expected)
        .with_detail("actual_checksum", &actual)
        .with_hint(
            "The downloaded asset may be corrupted or tampered with. Try re-running the update.",
        ));
    }

    Ok(())
}

fn extract_binary(
    archive_path: &Path,
    target: ReleaseTarget,
    output_dir: &Path,
) -> Result<PathBuf> {
    let destination = output_dir.join(target.binary_name);
    match target.archive_kind {
        ArchiveKind::TarGz => {
            extract_tar_gz_binary(archive_path, target.binary_name, &destination)?
        }
        ArchiveKind::Zip => extract_zip_binary(archive_path, target.binary_name, &destination)?,
    }
    ensure_executable(&destination)?;
    Ok(destination)
}

fn extract_tar_gz_binary(archive_path: &Path, binary_name: &str, destination: &Path) -> Result<()> {
    let file = File::open(archive_path).map_err(|error| {
        AppError::internal(format!(
            "could not open release archive `{}`: {error}",
            archive_path.display()
        ))
    })?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(map_archive_error)? {
        let mut entry = entry.map_err(map_archive_error)?;
        let path = entry.path().map_err(map_archive_error)?;
        if path.file_name().and_then(|value| value.to_str()) == Some(binary_name) {
            let mut output = File::create(destination).map_err(|error| {
                AppError::internal(format!(
                    "could not create extracted binary `{}`: {error}",
                    destination.display()
                ))
            })?;
            io::copy(&mut entry, &mut output).map_err(map_archive_error)?;
            return Ok(());
        }
    }

    Err(AppError::internal(format!(
        "binary `{}` was not found inside archive `{}`",
        binary_name,
        archive_path.display()
    )))
}

fn extract_zip_binary(archive_path: &Path, binary_name: &str, destination: &Path) -> Result<()> {
    let file = File::open(archive_path).map_err(|error| {
        AppError::internal(format!(
            "could not open release archive `{}`: {error}",
            archive_path.display()
        ))
    })?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        AppError::internal(format!("could not read the zip archive structure: {error}"))
    })?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| {
            AppError::internal(format!("could not read zip entry {index}: {error}"))
        })?;
        if entry.name().ends_with(binary_name) && !entry.is_dir() {
            let mut output = File::create(destination).map_err(|error| {
                AppError::internal(format!(
                    "could not create extracted binary `{}`: {error}",
                    destination.display()
                ))
            })?;
            io::copy(&mut entry, &mut output).map_err(|error| {
                AppError::internal(format!("could not extract zip entry to disk: {error}"))
            })?;
            return Ok(());
        }
    }

    Err(AppError::internal(format!(
        "binary `{}` was not found inside archive `{}`",
        binary_name,
        archive_path.display()
    )))
}

fn ensure_executable(path: &Path) -> Result<()> {
    #[cfg(not(unix))]
    let _ = path;

    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|error| {
                AppError::internal(format!(
                    "could not read file metadata for `{}`: {error}",
                    path.display()
                ))
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|error| {
            AppError::internal(format!(
                "could not set executable permissions on `{}`: {error}",
                path.display()
            ))
        })?;
    }

    Ok(())
}

fn map_archive_error(error: impl std::fmt::Display) -> AppError {
    AppError::internal(format!(
        "could not extract the release archive contents: {error}"
    ))
    .with_operation("extract_archive")
    .with_hint("The archive may be corrupted. Try re-running the update to download a fresh copy.")
}
