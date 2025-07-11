
use std::{collections::HashMap, fs::{self, File, OpenOptions}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use crate::{dto::variant_dto::{VariantDto, VariantListDto}, variant::{hgvs_variant::HgvsVariant, structural_variant::StructuralVariant, variant_manager::VariantManager, variant_validator::VariantValidator}};

use crate::variant::structural_variant::DELETION as DEL;
use crate::variant::structural_variant::DUPLICATION as DUP;
use crate::variant::structural_variant::INVSERSION as INV;
use crate::variant::structural_variant::TRANSLOCATION as TRANSL;

type VariantCache = HashMap<String, HgvsVariant>;
type StructuralCache = HashMap<String, StructuralVariant>;

pub struct DirManager {
    /// Path to the directory where we store the various files for the project (e.g., FBN1 for the FBN1 cohort)
    cache_dir_path: PathBuf,
    variant_manager: VariantManager
}


impl DirManager {
    /// Open the directory at the indicated location; if it does not exist, create it.
    /// Once we have opened the directory, open or create the HGVS cache.
    pub fn new<P: AsRef<Path>>(dir_path: P) -> Result<Self, String> {
        let path_buf = dir_path.as_ref().to_path_buf();
        if !path_buf.exists() {
            fs::create_dir_all(&path_buf).map_err(|e| e.to_string())?;
        }
        if !path_buf.is_dir() {
            return Err(format!("Path exists but is not a directory: {:?}", path_buf));
        }
        let vmanager = VariantManager::new(&path_buf);
        Ok(Self {
            cache_dir_path: path_buf,
            variant_manager: vmanager
        })
    }

    /// Check an HGVS or structural variant.
    /// If we validate, we return the same DTO (except that the validated flag is set to true)
    /// The cause of any error is returned as a string.
    pub fn validate_variant(
        &mut self, 
        variant: &VariantDto) 
    -> Result<VariantDto, String> {
        self.variant_manager.validate_variant(variant)
    }

    pub fn validate_variant_dto_list(&mut self, variant_dto_list: Vec<VariantDto>) -> Vec<VariantDto> {
        self.variant_manager.validate_variant_dto_list(variant_dto_list)
    }

    pub fn get_cohort_dir(&self) -> PathBuf {
        self.cache_dir_path.clone()
    }

    pub fn variant_manager(&mut self) -> &VariantManager {
        &self.variant_manager
    }

     pub fn get_hgvs_dict(&self) -> &HashMap<String, HgvsVariant> {
        self.variant_manager.get_hgvs_dict()
    }

    pub fn get_structural_dict(&self) -> &HashMap<String, StructuralVariant> {
        self.variant_manager.get_structural_dict()
    }
}



// region:    --- Tests

#[cfg(test)]
mod tests {
     

    use super::*;


    #[test]
    #[ignore]
    pub fn test_var() {
        let dirpat = "/Users/robin/TMP/CMPK2";
        let result = DirManager::new(dirpat);
        assert!(result.is_ok());
        let mut dirman = result.unwrap();
        let var_dto = VariantDto::new_hgvs("c.1A>C", "NM_207315.4",   "HGNC:27015","CMPK2");
        assert_eq!(false, var_dto.validated());
        let updated_result = dirman.validate_variant(&var_dto);
        assert!(updated_result.is_ok());
        let updated_dto = updated_result.unwrap();
        assert!(updated_dto.validated());
    }
    
}

// endregion: --- Tests