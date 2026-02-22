//! Version-aware CDN download and local file cache manager.
//!
//! Downloads and caches MTGJSON data files from the CDN. Checks Meta.json for
//! version changes and re-downloads when stale. Individual files are downloaded
//! lazily on first access.

use crate::config;
use crate::error::{MtgjsonError, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Downloads and caches MTGJSON data files from the CDN.
///
/// Checks Meta.json for version changes and re-downloads when stale.
/// Individual files are downloaded lazily on first access.
pub struct CacheManager {
    /// Directory where cached files are stored.
    pub cache_dir: PathBuf,
    /// If true, never download from CDN (use cached files only).
    pub offline: bool,
    timeout: Duration,
    client: Option<Client>,
    remote_ver: Option<String>,
}

impl CacheManager {
    /// Create a new cache manager.
    ///
    /// If `cache_dir` is `None`, uses the platform-appropriate default cache directory.
    /// Creates the cache directory if it does not exist.
    pub fn new(cache_dir: Option<PathBuf>, offline: bool, timeout: Duration) -> Result<Self> {
        let dir = cache_dir.unwrap_or_else(config::default_cache_dir);
        fs::create_dir_all(&dir)?;
        Ok(Self {
            cache_dir: dir,
            offline,
            timeout,
            client: None,
            remote_ver: None,
        })
    }

    /// Lazy HTTP client, created on first use.
    pub fn client(&mut self) -> &Client {
        if self.client.is_none() {
            self.client = Some(
                Client::builder()
                    .timeout(self.timeout)
                    .redirect(reqwest::redirect::Policy::limited(10))
                    .build()
                    .expect("failed to build HTTP client"),
            );
        }
        self.client.as_ref().unwrap()
    }

    /// Read the locally cached version string from `version.txt`.
    fn local_version(&self) -> Option<String> {
        let version_file = self.cache_dir.join("version.txt");
        if version_file.exists() {
            fs::read_to_string(&version_file)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    /// Save a version string to `version.txt` in the cache directory.
    fn save_version(&self, version: &str) {
        let version_file = self.cache_dir.join("version.txt");
        let _ = fs::write(version_file, version);
    }

    /// Fetch the current MTGJSON version from Meta.json on the CDN.
    ///
    /// Returns the version string (e.g. `"5.2.2+20240101"`), or `None` if
    /// offline or the CDN is unreachable. Caches the result for subsequent calls.
    pub fn remote_version(&mut self) -> Result<Option<String>> {
        if self.remote_ver.is_some() {
            return Ok(self.remote_ver.clone());
        }
        if self.offline {
            return Ok(None);
        }
        let client = self.client().clone();
        match client.get(config::META_URL).send() {
            Ok(resp) => {
                let resp = resp.error_for_status()?;
                let data: serde_json::Value = resp.json()?;
                // Try data.version first, then meta.version
                let version = data
                    .get("data")
                    .and_then(|d| d.get("version"))
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        data.get("meta")
                            .and_then(|m| m.get("version"))
                            .and_then(|v| v.as_str())
                    })
                    .map(|s| s.to_string());
                self.remote_ver = version.clone();
                Ok(version)
            }
            Err(e) => {
                eprintln!("Failed to fetch MTGJSON version from CDN: {}", e);
                Ok(None)
            }
        }
    }

    /// Check if local cache is out of date compared to the CDN.
    ///
    /// Returns `true` if there is no local cache or the CDN has a newer version.
    /// Returns `false` if up to date or if the CDN is unreachable.
    pub fn is_stale(&mut self) -> Result<bool> {
        let local = self.local_version();
        match local {
            None => Ok(true),
            Some(local_ver) => {
                let remote = self.remote_version()?;
                match remote {
                    None => Ok(false), // Can't check, assume fresh
                    Some(remote_ver) => Ok(local_ver != remote_ver),
                }
            }
        }
    }

    /// Download a single file from the CDN.
    ///
    /// Downloads to a temp file first and renames on success, so an
    /// interrupted download never leaves a corrupt partial file behind.
    fn download_file(&mut self, filename: &str, dest: &Path) -> Result<()> {
        let url = format!("{}/{}", config::CDN_BASE, filename);
        eprintln!("Downloading {}", url);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_dest = dest.with_extension(
            format!(
                "{}.tmp",
                dest.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
            ),
        );

        let client = self.client().clone();
        let result = (|| -> Result<()> {
            let resp = client.get(&url).send()?.error_for_status()?;
            let bytes = resp.bytes()?;
            fs::write(&tmp_dest, &bytes)?;
            fs::rename(&tmp_dest, dest)?;
            Ok(())
        })();

        if result.is_err() {
            // Clean up partial temp file on any error
            let _ = fs::remove_file(&tmp_dest);
        }

        result
    }

    /// Ensure a parquet file is cached locally, downloading if needed.
    ///
    /// # Arguments
    ///
    /// * `view_name` - Logical view name (e.g. `"cards"`, `"sets"`).
    ///
    /// # Returns
    ///
    /// Local filesystem path to the cached parquet file.
    pub fn ensure_parquet(&mut self, view_name: &str) -> Result<PathBuf> {
        let parquet_files = config::parquet_files();
        let filename = parquet_files.get(view_name).ok_or_else(|| {
            MtgjsonError::NotFound(format!("Unknown parquet view: {}", view_name))
        })?;

        let local_path = self.cache_dir.join(filename);

        if !local_path.exists() || self.is_stale()? {
            if self.offline {
                if local_path.exists() {
                    return Ok(local_path);
                }
                return Err(MtgjsonError::NotFound(format!(
                    "Parquet file {} not cached and offline mode is enabled",
                    filename
                )));
            }
            self.download_file(filename, &local_path)?;
            // Update version after successful download
            if let Ok(Some(version)) = self.remote_version() {
                self.save_version(&version);
            }
        }

        Ok(local_path)
    }

    /// Ensure a JSON file is cached locally, downloading if needed.
    ///
    /// # Arguments
    ///
    /// * `name` - Logical file name (e.g. `"meta"`, `"all_prices_today"`).
    ///
    /// # Returns
    ///
    /// Local filesystem path to the cached JSON file.
    pub fn ensure_json(&mut self, name: &str) -> Result<PathBuf> {
        let json_files = config::json_files();
        let filename = json_files.get(name).ok_or_else(|| {
            MtgjsonError::NotFound(format!("Unknown JSON file: {}", name))
        })?;

        let local_path = self.cache_dir.join(filename);

        if !local_path.exists() || self.is_stale()? {
            if self.offline {
                if local_path.exists() {
                    return Ok(local_path);
                }
                return Err(MtgjsonError::NotFound(format!(
                    "JSON file {} not cached and offline mode is enabled",
                    filename
                )));
            }
            self.download_file(filename, &local_path)?;
            // Update version after successful download
            if let Ok(Some(version)) = self.remote_version() {
                self.save_version(&version);
            }
        }

        Ok(local_path)
    }

    /// Load and parse a JSON file (handles `.gz` transparently).
    ///
    /// If the cached file is corrupt (truncated download, disk error),
    /// it is deleted automatically so the next call re-downloads a fresh copy.
    pub fn load_json(&mut self, name: &str) -> Result<serde_json::Value> {
        let path = self.ensure_json(name)?;

        let parse_result = if path.extension().and_then(|e| e.to_str()) == Some("gz") {
            let file = fs::File::open(&path)?;
            let reader = BufReader::new(file);
            let decoder = GzDecoder::new(reader);
            let mut buf_reader = BufReader::new(decoder);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;
            serde_json::from_str(&contents).map_err(MtgjsonError::from)
        } else {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).map_err(MtgjsonError::from)
        };

        match parse_result {
            Ok(value) => Ok(value),
            Err(e) => {
                eprintln!(
                    "Corrupt cache file {}: {} -- removing",
                    path.display(),
                    e
                );
                let _ = fs::remove_file(&path);
                Err(MtgjsonError::NotFound(format!(
                    "Cache file '{}' was corrupt and has been removed. \
                     Retry to re-download. Original error: {}",
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown"),
                    e
                )))
            }
        }
    }

    /// Remove all cached files and recreate the cache directory.
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Close the HTTP client, if open.
    pub fn close(&mut self) {
        self.client = None;
    }
}
