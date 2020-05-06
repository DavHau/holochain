#![forbid(missing_docs)]
//! DnaFile Utilities

use holochain_serialized_bytes::prelude::*;
use holochain_types::dna::{wasm::DnaWasm, zome::Zome, DnaDef, DnaFile};
use std::collections::BTreeMap;

/// DnaUtilError type.
#[derive(Debug, thiserror::Error)]
pub enum DnaUtilError {
    /// std::io::Error
    #[error("std::io::Error: {0}")]
    StdIoError(#[from] std::io::Error),

    /// DnaError
    #[error("DnaError: {0}")]
    DnaError(#[from] holochain_types::dna::DnaError),

    /// SerializedBytesError
    #[error("SerializedBytesError: {0}")]
    SerializedBytesError(#[from] SerializedBytesError),

    /// InvalidInput
    #[error("InvalidInput: {0}")]
    InvalidInput(String),

    /// serde_json::Error
    #[error("serde_json::Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

/// DnaUtil Result type.
pub type DnaUtilResult<T> = Result<T, DnaUtilError>;

/// internal convert between dna_file_path and dna_work_dir
fn dna_file_path_convert(
    dna_file_path: &impl AsRef<std::path::Path>,
    to_work_dir: bool,
) -> DnaUtilResult<std::path::PathBuf> {
    let dna_file_path = dna_file_path.as_ref();

    let tmp_lossy = dna_file_path.to_string_lossy();
    if to_work_dir {
        if !tmp_lossy.ends_with(".dna.gz") {
            return Err(DnaUtilError::InvalidInput(format!(
                "bad extract path, dna files must end with '.dna.gz': {}",
                dna_file_path.display()
            )));
        }
    } else {
        if !tmp_lossy.ends_with(".dna.workdir") {
            return Err(DnaUtilError::InvalidInput(format!(
                "bad compile path, work dirs must end with '.dna_work_dir': {}",
                dna_file_path.display()
            )));
        }
    }

    let filename = dna_file_path
        .file_name()
        .ok_or_else(|| {
            DnaUtilError::InvalidInput(format!(
                "could not extract filename from: {}",
                dna_file_path.display()
            ))
        })?
        .to_string_lossy();

    let new_name = if to_work_dir {
        let filename_base = &filename[..filename.len() - 7];
        format!("{}.dna.workdir", filename_base)
    } else {
        let filename_base = &filename[..filename.len() - 12];
        format!("{}.dna.gz", filename_base)
    };

    let mut dir = std::path::PathBuf::new();
    dir.push(dna_file_path);
    dir.set_file_name(new_name);

    Ok(dir)
}

/// Expand a DnaFile into a Dna Working Directory
pub async fn expand(dna_file_path: &impl AsRef<std::path::Path>) -> DnaUtilResult<()> {
    let dna_file_path = dna_file_path.as_ref().canonicalize()?;
    let dir = dna_file_path_convert(&dna_file_path, true)?;
    tokio::fs::create_dir_all(&dir).await?;

    let dna_file = DnaFile::from_file_content(&tokio::fs::read(dna_file_path).await?).await?;

    for (zome_name, zome) in &dna_file.dna().zomes {
        let wasm_hash = &zome.wasm_hash;
        let wasm = dna_file.code().get(wasm_hash).expect("dna_file corrupted");
        let mut wasm_filename = dir.clone();
        wasm_filename.push(format!("{}.wasm", zome_name));
        tokio::fs::write(wasm_filename, &*wasm.code()).await?;
    }

    // Might be more efficient to extract the DnaDef / Wasm from the DnaFile
    // then pass by value here.
    let dna_json = DnaDefJson::from_dna_def(dna_file.dna().clone())?;
    let dna_json = serde_json::to_string_pretty(&dna_json)?;

    let mut json_filename = dir.clone();
    json_filename.push("dna.json");
    tokio::fs::write(json_filename, dna_json.as_bytes()).await?;

    Ok(())
}

/// Compress a Dna Working Directory into a DnaFile
pub async fn compress(dna_work_dir: &impl AsRef<std::path::Path>) -> DnaUtilResult<()> {
    let dna_work_dir = dna_work_dir.as_ref().canonicalize()?;
    let dna_file_path = dna_file_path_convert(&dna_work_dir, false)?;

    let mut json_filename = dna_work_dir.clone();
    json_filename.push("dna.json");

    let json_data = tokio::fs::read(json_filename).await?;

    let json_file: DnaDefJson = serde_json::from_slice(&json_data)?;

    let dna_file_content = json_file.compile_dna_file(&dna_work_dir).await?;
    let dna_file_content = dna_file_content.to_file_content().await?;

    tokio::fs::write(dna_file_path, &dna_file_content).await?;

    Ok(())
}

/// See `holochain_types::dna::zome::Zome`.
/// This is a helper to convert to json.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ZomeJson {
    pub wasm_path: String,
}

/// Special Json Value Decode Helper
#[derive(Debug, serde::Serialize, serde::Deserialize, SerializedBytes)]
struct JsonValueDecodeHelper(pub serde_json::Value);

/// See `holochain_types::dna::DnaDef`.
/// This is a helper to convert to json.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DnaDefJson {
    pub name: String,
    pub uuid: String,
    pub properties: serde_json::Value,
    pub zomes: BTreeMap<String, ZomeJson>,
}

impl DnaDefJson {
    pub fn from_dna_def(dna: DnaDef) -> DnaUtilResult<DnaDefJson> {
        let properties: JsonValueDecodeHelper = dna.properties.try_into()?;
        let mut zomes = BTreeMap::new();
        for zome_name in dna.zomes.keys() {
            let zome_file = format!("./{}.wasm", zome_name);
            zomes.insert(
                zome_name.clone(),
                ZomeJson {
                    wasm_path: zome_file,
                },
            );
        }
        Ok(Self {
            name: dna.name,
            uuid: dna.uuid,
            properties: properties.0,
            zomes,
        })
    }

    pub async fn compile_dna_file(
        &self,
        work_dir: impl Into<std::path::PathBuf>,
    ) -> DnaUtilResult<DnaFile> {
        let work_dir = work_dir.into();

        let properties: SerializedBytes =
            JsonValueDecodeHelper(self.properties.clone()).try_into()?;

        let mut zomes = BTreeMap::new();
        let mut wasm_list = Vec::new();

        for (zome_name, zome) in self.zomes.iter() {
            let mut zome_file_path = work_dir.clone();
            zome_file_path.push(&zome.wasm_path);

            let zome_content = tokio::fs::read(zome_file_path).await?;

            let wasm: DnaWasm = zome_content.into();
            let wasm_hash = holo_hash::WasmHash::with_data(&wasm.code()).await;
            let wasm_hash: holo_hash_core::WasmHash = wasm_hash.into();
            zomes.insert(zome_name.clone(), Zome { wasm_hash });
            wasm_list.push(wasm);
        }

        let dna = DnaDef {
            name: self.name.clone(),
            uuid: self.uuid.clone(),
            properties,
            zomes,
        };

        Ok(DnaFile::new(dna, wasm_list).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(threaded_scheduler)]
    async fn test_extract_then_compile() {
        let tmp_dir = tempdir::TempDir::new("dna_util_test").unwrap();

        let dna_file = holochain_types::test_utils::fake_dna_zomes(
            "bla",
            vec![
                ("test-zome-1".into(), vec![1, 2, 3, 4].into()),
                ("test-zome-2".into(), vec![5, 6, 7, 8].into()),
            ],
        );
        let properties = JsonValueDecodeHelper(serde_json::json!({
            "test_prop_1": ["a", 42],
            "test_prop_2": {
                "bool": true,
            }
        }));
        let dna_file = dna_file
            .with_properties(SerializedBytes::try_from(properties).unwrap())
            .await
            .unwrap();

        let dna_filename = tmp_dir.path().join("test-dna.dna.gz");

        tokio::fs::write(&dna_filename, dna_file.to_file_content().await.unwrap())
            .await
            .unwrap();

        expand(&dna_filename).await.unwrap();

        tokio::fs::remove_file(&dna_filename).await.unwrap();

        compress(&tmp_dir.path().join("test-dna.dna.workdir"))
            .await
            .unwrap();

        let dna_file_content = tokio::fs::read(&dna_filename).await.unwrap();
        let dna_file2 = DnaFile::from_file_content(&dna_file_content).await.unwrap();

        assert_eq!(dna_file, dna_file2);
    }
}
