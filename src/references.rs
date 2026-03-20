//! Reference values configuration for benchmark scoring.
//!
//! Reference values define the baseline performance used for scoring.
//! A score of 1000 means the system matches the reference performance.
//! Scores above 1000 indicate better performance, below 1000 indicates slower.
//!
//! To customize reference values, create a config file at:
//! - Linux/macOS: ~/.config/qluelessbench/references.json
//! - Windows: %APPDATA%/qluelessbench/references.json
//!
//! Or set the QLUELESSBENCH_REFS environment variable to a JSON file path.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceValues {
    pub fileio: FileIOReferences,
    pub compression: CompressionReferences,
    pub image_processing: ImageProcessingReferences,
    pub text_processing: TextProcessingReferences,
    pub database: DatabaseReferences,
    pub mathematical: MathematicalReferences,
    pub archive: ArchiveReferences,
    pub memory: MemoryReferences,
    pub cryptography: CryptographyReferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIOReferences {
    pub sequential_write_mbps: f64,
    pub sequential_read_mbps: f64,
    pub random_access_ops: f64,
    pub copy_mbps: f64,
    pub delete_files_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionReferences {
    pub zip_level_1_mbps: f64,
    pub zip_level_6_mbps: f64,
    pub zip_level_9_mbps: f64,
    pub gzip_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProcessingReferences {
    pub resize_mbps: f64,
    pub blur_mbps: f64,
    pub sharpen_mbps: f64,
    pub format_convert_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProcessingReferences {
    pub search_mbps: f64,
    pub regex_mbps: f64,
    pub string_ops_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseReferences {
    pub insert_ops: f64,
    pub update_ops: f64,
    pub delete_ops: f64,
    pub search_ops: f64,
    pub indexed_lookup_ops: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalReferences {
    pub array_ops_gflops: f64,
    pub matrix_mult_gflops: f64,
    pub statistics_gflops: f64,
    pub primes_mops: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReferences {
    pub create_zip_mbps: f64,
    pub extract_zip_mbps: f64,
    pub create_tar_mbps: f64,
    pub extract_tar_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReferences {
    pub alloc_dealloc_mbps: f64,
    pub vec_ops_mbps: f64,
    pub hashmap_ops_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographyReferences {
    pub aes_encrypt_mbps: f64,
    pub sha256_mbps: f64,
}

impl Default for ReferenceValues {
    fn default() -> Self {
        ReferenceValues {
            fileio: FileIOReferences {
                sequential_write_mbps: 200.0,
                sequential_read_mbps: 300.0,
                random_access_ops: 50000.0,
                copy_mbps: 250.0,
                delete_files_per_sec: 500.0,
            },
            compression: CompressionReferences {
                zip_level_1_mbps: 100.0,
                zip_level_6_mbps: 50.0,
                zip_level_9_mbps: 20.0,
                gzip_mbps: 80.0,
            },
            image_processing: ImageProcessingReferences {
                resize_mbps: 50.0,
                blur_mbps: 30.0,
                sharpen_mbps: 40.0,
                format_convert_mbps: 60.0,
            },
            text_processing: TextProcessingReferences {
                search_mbps: 200.0,
                regex_mbps: 100.0,
                string_ops_mbps: 300.0,
            },
            database: DatabaseReferences {
                insert_ops: 5000.0,
                update_ops: 4000.0,
                delete_ops: 4500.0,
                search_ops: 3000.0,
                indexed_lookup_ops: 10000.0,
            },
            mathematical: MathematicalReferences {
                array_ops_gflops: 10.0,
                matrix_mult_gflops: 8.0,
                statistics_gflops: 5.0,
                primes_mops: 2.0,
            },
            archive: ArchiveReferences {
                create_zip_mbps: 80.0,
                extract_zip_mbps: 60.0,
                create_tar_mbps: 100.0,
                extract_tar_mbps: 70.0,
            },
            memory: MemoryReferences {
                alloc_dealloc_mbps: 5000.0,
                vec_ops_mbps: 3000.0,
                hashmap_ops_mbps: 2000.0,
            },
            cryptography: CryptographyReferences {
                aes_encrypt_mbps: 500.0,
                sha256_mbps: 300.0,
            },
        }
    }
}

impl ReferenceValues {
    pub fn load() -> Self {
        if let Some(path) = find_config_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(values) = serde_json::from_str(&content) {
                    return values;
                }
            }
        }
        ReferenceValues::default()
    }

    pub fn save_to_default_location() -> Result<PathBuf, anyhow::Error> {
        let path = get_default_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&ReferenceValues::default())?;
        fs::write(&path, json)?;
        Ok(path)
    }

    pub fn as_map(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        map.insert(
            "fileio.write".to_string(),
            self.fileio.sequential_write_mbps,
        );
        map.insert("fileio.read".to_string(), self.fileio.sequential_read_mbps);
        map.insert("fileio.random".to_string(), self.fileio.random_access_ops);
        map.insert("fileio.copy".to_string(), self.fileio.copy_mbps);
        map.insert(
            "fileio.delete".to_string(),
            self.fileio.delete_files_per_sec,
        );
        map
    }
}

fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("qluelessbench").join("references.json"))
}

fn find_config_file() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("QLUELESSBENCH_REFS") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    get_default_config_path().filter(|p| p.exists())
}

pub fn get() -> ReferenceValues {
    ReferenceValues::load()
}
