//! PpktExporter -- One row together with Header information, all that is needed to export a GA4GH phenopacket
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! If a PpktExporter instance has no error, then we are ready to create a phenopacket.

use std::collections::HashMap;
use std::fmt::{self};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use ontolius::ontology::csr::FullCsrOntology;
use ontolius::term::simple::SimpleTerm;
use ontolius::TermId;
use polars::prelude::default_arrays;

use crate::dto::case_dto::CaseDto;
use crate::dto::hpo_term_dto::HpoTermDto;
use crate::dto::template_dto::{CellDto, DiseaseDto, GeneVariantBundleDto, IndividualBundleDto, RowDto, TemplateDto};
use crate::dto::validation_errors::ValidationErrors;
use crate::header::individual_header::IndividualHeader;
use crate::hpo::age_util::{self, check_hpo_table_cell};
use crate::hpo::hpo_util;
use crate::template::curie::Curie;
use crate::error::{self, Error, Result};
use crate::template::disease_bundle::{self, DiseaseBundle};
use crate::template::gene_variant_bundle::{self, GeneVariantBundle};
use crate::template::individual_bundle::IndividualBundle;
use crate::template::pt_template::TemplateType;
use crate::template::simple_label::SimpleLabel;
use crate::template::header_duplet_row::{self, HeaderDupletRow};

/// The index where the Mendelian demographic part sars
const DEMOGRAPHIC_IDX:usize = 12;


#[derive(Clone, Debug)]
pub struct PpktRow {
    header: Arc<HeaderDupletRow>,
    individual_bundle: IndividualBundle,
    disease_bundle_list: Vec<DiseaseBundle>,
    gene_var_bundle_list: Vec<GeneVariantBundle>,
    hpo_content: Vec<String>
}



impl PpktRow {
    pub fn from_row(
        header: Arc<HeaderDupletRow>,
        content: Vec<String>,
    ) -> std::result::Result<Self, ValidationErrors> {
        match header.template_type() {
            crate::template::pt_template::TemplateType::Mendelian => Self::from_mendelian_row(header, content),
            crate::template::pt_template::TemplateType::Melded => todo!(),
        }
    }

    pub fn from_mendelian_row(
        header: Arc<HeaderDupletRow>,
        content: Vec<String>
    ) -> std::result::Result<Self, ValidationErrors> {
        let ibundle = IndividualBundle::from_row(&content, DEMOGRAPHIC_IDX)?;
        let disease_bundle = DiseaseBundle::from_row(&content, 4)?; // todo -- put index contents in same place
        let gene_variant_bundle = GeneVariantBundle::from_row(&content, 6)?;
        let mut verrs = ValidationErrors::new();
        let mut hpo_content: Vec<String> = Vec::new();
        for item in content.iter().skip(17) {
            let cell = if item.trim().is_empty() { "na" } else { item }; // TODO -- remove once old templates have been restructured
            verrs.push_result(age_util::check_hpo_table_cell(&item));
            hpo_content.push(item.clone());
        }
        if verrs.has_error() {
            return Err(verrs);
        }
        Ok(Self { header: header.clone(), 
            individual_bundle: ibundle, 
            disease_bundle_list: vec![disease_bundle], 
            gene_var_bundle_list: vec![gene_variant_bundle],
            hpo_content 
        })
    }

    /// Create a new PpktRow. This is used when we create a row (phenopacket) with terms that
    /// may not be included in the previous phenopackets and which may not have values for all of the
    /// terms in the previous phenopackets. 
    ///  # Arguments
    ///
    /// * `header` - Header with all HPO terms in previous cohort and new phenopacket, ordered by DFS
    /// * `individual_dto` - DTO with demographic information about the new individual
    /// * `gene_variant_list` - genotypes
    /// * `tid_to_value_map` - this has values (e.g., observed, na, P32Y2M) for which we have information in the new phenopacket
    /// * `cohort_dto`- DTO for the entire previous cohort (TODO probably we need a better DTO with the new DiseaseBundle!)
    pub fn from_tid_to_value_map(
        header: Arc<HeaderDupletRow>, 
        individual_dto: IndividualBundleDto,
        gene_variant_list: Vec<GeneVariantBundleDto>,
        tid_to_value_map: HashMap<TermId, String>, 
        cohort_dto: TemplateDto) -> std::result::Result<Self, String> {
        if cohort_dto.cohort_type != TemplateType::Mendelian {
            panic!("from_map: Melded not supported");
        }
        let mut items = Vec::with_capacity(header.hpo_count());
        for hduplet in header.hpo_duplets() {
            let tid = hduplet.to_term_id()?;
            let value: String =  tid_to_value_map.get(&tid).map_or("na", |v| v).to_string();
            items.push(value);
        }
        let ibundle = IndividualBundle::from_dto(individual_dto);
        let disease_bundle_list = DiseaseBundle::from_cohort_dto(&cohort_dto)?;
        let gvb_list = GeneVariantBundle::from_dto_list(gene_variant_list);
        Ok(Self { header, 
            individual_bundle: ibundle, 
            disease_bundle_list, 
            gene_var_bundle_list: gvb_list, 
            hpo_content: items
        })
    }


    pub fn from_dto(dto: &RowDto, header: Arc<HeaderDupletRow>) -> Self {
        let hpo_content = dto.hpo_data.iter()
            .map(|c|c.value.clone())
            .collect();
        Self { 
            header, 
            individual_bundle: IndividualBundle::from_dto(dto.individual_dto.clone()), 
            disease_bundle_list: DiseaseBundle::from_dto_list(dto.disease_dto_list.clone()), 
            gene_var_bundle_list: GeneVariantBundle::from_dto_list(dto.gene_var_dto_list.clone()), 
            hpo_content
        }
    }

    pub fn get_individual_dto(&self) -> IndividualBundleDto {
        let ibdl = &self.individual_bundle;
        IndividualBundleDto::new(ibdl.pmid(), ibdl.title(), ibdl.individual_id(), ibdl.comment(),
            ibdl.age_of_onset(), ibdl.age_at_last_encounter(), ibdl.deceased(), ibdl.sex())
    }

    pub fn get_disease_dto_list(&self) -> Vec<DiseaseDto> {
        let mut dto_list: Vec<DiseaseDto> = Vec::new();
        for disease in &self.disease_bundle_list {
            dto_list.push(disease.to_dto())
        }
        dto_list
    }

    pub fn get_gene_var_dto_list(&self) -> Vec<GeneVariantBundleDto> {
        let mut gbdto_list: Vec<GeneVariantBundleDto> = Vec::new();
        for gvb in &self.gene_var_bundle_list{
            gbdto_list.push(gvb.to_dto());
        }
        gbdto_list
    }

    pub fn get_hpo_value_list(&self) -> Vec<CellDto> {
        let mut cell_dto_list: Vec<CellDto> = Vec::new();
        for hpo_val in &self.hpo_content {
            cell_dto_list.push(CellDto::new(hpo_val));
        }
        cell_dto_list
    }

    pub fn get_hpo_term_dto_list(&self) -> std::result::Result<Vec<HpoTermDto>, String> {
        self.header.get_hpo_term_dto_list(&self.hpo_content).map_err(|e| e.to_string())
    }


    pub fn mendelian_from_dto(
        &self,
        header_duplet_row: Arc<HeaderDupletRow>,
        individual_dto: IndividualBundleDto,
        annotations: Vec<HpoTermDto>,
        existing_annotation_map:HashMap<TermId, String>) 
    -> std::result::Result<Self, ValidationErrors> 
    {
        let verrs = ValidationErrors::new();
        //let existing_header = self.header:
        
       /*  Ok(Self {
            header_duplet_row: header_duplet_row,
            content: values
        })*/
        if verrs.has_error() {
            Err(verrs)
        } else {
            Ok(Self{ 
                header: header_duplet_row, 
                individual_bundle: todo!(), 
                disease_bundle_list: todo!(), 
                gene_var_bundle_list: todo!(), 
                hpo_content: todo!() })
        }
    }

    /// This function checks the current PpktRow for syntactical errors
    pub fn check_for_errors(&self) -> std::result::Result<(), ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        verrs.push_verr_result(self.individual_bundle.do_qc());
        for db in &self.disease_bundle_list {
            verrs.push_verr_result(db.do_qc());
        }
        for gvb in &self.gene_var_bundle_list {
            verrs.push_verr_result(gvb.do_qc());
        }
        for item in &self.hpo_content {
            verrs.push_result(check_hpo_table_cell(item));
        }

        verrs.ok()
    }

    /// Update current HPO values according to a new header.
    /// The new header may contain HPO terms that the current PpktRow does not have
    /// in this case, we must add 'na' as the value for these terms.
    pub fn update_header(
        &self, 
        updated_hdr: Arc<HeaderDupletRow>
    ) -> std::result::Result<Self, ValidationErrors> {
        let mut verrs = ValidationErrors::new();
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;
        let previous_header = &self.header;
        let hpo_map = previous_header.get_hpo_content_map(&self.hpo_content);
        let hpo_map = hpo_map.map_err(|e|{
            verrs.push_str(e);
            verrs // only propagated if error occurs
        })?;
        let mut content = Vec::new();
        for tid in updated_hpo_id_list {
            let item: String = hpo_map
                .get(&tid)
                .cloned() // converts Option<&String> to Option<String>
                .unwrap_or_else(|| "na".to_string()); // this will pertain to the new HPO term we are adding
            content.push(item);
        }

        Ok(Self { 
            header: updated_hdr, 
            individual_bundle: self.individual_bundle.clone(), 
            disease_bundle_list: self.disease_bundle_list.clone(), 
            gene_var_bundle_list: self.gene_var_bundle_list.clone(), 
            hpo_content: content 
        })
    }

    /// Given a previous list of `TermId`s and an updated list, this function
    /// returns a vector of indices representing where each element of the
    /// `previous_hpo_list` now appears in the `updated_hpo_list`.
    ///
    /// This is useful for tracking how terms from an earlier template are
    /// rearranged after updating the template (e.g., after inserting or reordering terms).
    /// The returned vector can be used to remap associated data (e.g., column values)
    /// to their new positions.
    ///
    /// # Arguments
    /// - `previous_hpo_list`: The list of HPO term IDs before the update.
    /// - `updated_hpo_list`: The reordered or expanded list of HPO term IDs after the update.
    ///                       It must contain all terms from `previous_hpo_list`.
    ///
    /// # Returns
    /// A `Vec<usize>` where each element `i` gives the index in `updated_hpo_list`
    /// of the `i`-th term in `previous_hpo_list`.
    ///
    /// # Panics
    /// This function will panic if any term from `previous_hpo_list` is not found in `updated_hpo_list`.
    ///
    pub fn get_update_vector(
        previous_hpo_list: &[TermId],
        updated_hpo_list: &[TermId])
    -> Vec<usize> {
        let id_to_new_index: HashMap<TermId, usize> = updated_hpo_list
            .iter()
            .enumerate()
            .map(|(i, tid)| (tid.clone(), i))
            .collect();
        let new_indices: Vec<usize> = previous_hpo_list
            .iter()
            .map(|tid| id_to_new_index[tid])
            .collect();
        new_indices
    }

    /// Given the old values and a mapping from old indices to new indices,
    /// return a new vector of the size of the updated list, where each element
    /// from the original list is moved to its new index, and all other positions
    /// are filled with `"na"`.
    ///
    /// # Arguments
    /// - `old_values`: The values associated with the old HPO list (same order).
    /// - `old_to_new_indices`: A vector where `old_to_new_indices[i]` gives the
    ///                         index in the new list where the `i`th old value should go.
    /// - `new_size`: The size of the new list (typically, `updated_hpo_list.len()`).
    ///
    /// # Returns
    /// A `Vec<String>` of length `new_size` where old values are in their new positions,
    /// and new (missing) entries are `"na"`.
    fn reorder_or_fill_na(
        old_values: &[String],
        old_to_new_indices: &[usize],
        new_size: usize,
    ) -> Vec<String> {
        let mut new_values = vec!["na".to_string(); new_size];

        for (old_idx, &new_idx) in old_to_new_indices.iter().enumerate() {
            new_values[new_idx] = old_values[old_idx].clone();
        }

        new_values
    }


    pub fn update(
        &self, 
        tid_map: &mut HashMap<TermId, String>, 
        updated_hdr: Arc<HeaderDupletRow>) 
    -> std::result::Result<Self, ValidationErrors> {
        // update the tid map with the existing  values
        let mut verr = ValidationErrors::new();
        let previous_hpo_id_list = self.header.get_hpo_id_list()?;
        let hpo_cell_content_list = self.hpo_content.clone();
        if previous_hpo_id_list.len() != hpo_cell_content_list.len() {
            verr.push_str("Mismatched lengths between HPO ID list and HPO content");
            return Err(verr); // not recoverable
        }
        let updated_hpo_id_list = updated_hdr.get_hpo_id_list()?;
        let reordering_indices = Self::get_update_vector(&previous_hpo_id_list, &updated_hpo_id_list);

        let updated_hpo = Self::reorder_or_fill_na(&hpo_cell_content_list, 
        &reordering_indices,
        updated_hpo_id_list.len());
        Ok(Self {
            header: updated_hdr,
            individual_bundle: self.individual_bundle.clone(),
            disease_bundle_list: self.disease_bundle_list.clone(),
            gene_var_bundle_list: self.gene_var_bundle_list.clone(),
            hpo_content: updated_hpo,
        })
    }


}



#[cfg(test)]
mod test {
    use crate::{error::Error, header::{hpo_term_duplet::HpoTermDuplet}, hpo::hpo_util::{self, HpoUtil}};
    use lazy_static::lazy_static;
    use ontolius::{io::OntologyLoaderBuilder, ontology::{csr::MinimalCsrOntology, OntologyTerms}, term};
    use polars::io::SerReader;
    use super::*;
    use std::{fs::File, io::BufReader, str::FromStr};
    use rstest::{fixture, rstest};
    use flate2::bufread::GzDecoder;

    #[fixture]
    fn hpo() -> Arc<FullCsrOntology> {
        let path = "resources/hp.v2025-03-03.json.gz";
        let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();
        let hpo = loader.load_from_read(reader).unwrap();
        Arc::new(hpo)
    }



    #[fixture]
    fn row1() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id",	"gene_symbol", 
            "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", 
            "deceased", "sex", "HPO",	"Clinodactyly of the 5th finger", "Hallux valgus",	"Short 1st metacarpal", 
            "Ectopic ossification in muscle tissue", "Long hallux", "Pain", "Short thumb"
        ];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row2() -> Vec<String> 
    {
        let row: Vec<&str> = vec![
            "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na",
            "HP:0004209", "HP:0001822", "HP:0010034", "HP:0011987", "HP:0001847", "HP:0012531", "HP:0009778"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }

    #[fixture]
    fn row3() -> Vec<String> {
        let row: Vec<&str> =  vec![
            "PMID:29482508", "Difficult diagnosis and genetic analysis of fibrodysplasia ossificans progressiva: a case report", "current case", "", 
            "OMIM:135100", "Fibrodysplasia ossificans progressiva", "HGNC:171", "ACVR1", 
            "NM_001111067.4", "c.617G>A", "na", "NP_001104537.1:p.(Arg206His)", 
            "P9Y", "P16Y", "no", "M", "na", "na", "P16Y", "na", "P16Y", "P16Y", "P16Y", "na"];
        row.into_iter().map(|s| s.to_owned()).collect()
    }


    #[fixture]
    fn original_matrix(row1: Vec<String>, row2: Vec<String>, row3: Vec<String>)  -> Vec<Vec<String>> {
        vec![row1, row2, row3]
    }


    #[fixture]
    pub fn case_a_dto() -> CaseDto {
        CaseDto::new(
            "PMID:123", 
            "A Recurrent De Novo Nonsense Variant", 
            "Individual 7", 
            "",  // comment
            "c.2737C>T",  // allele_1
            "na", // allele_2
            "",  // variant.comment
            "Infantile onset", // age_at_onset
            "P32Y", //  age_at_last_encounter
            "na", // deceased
            "M" //sex
        )
    }

    #[fixture]
    pub fn hpo_dtos() -> Vec<HpoTermDto> {
        vec![HpoTermDto::new("HP:0001382", "Joint hypermobility", "observed"),
        HpoTermDto::new("HP:0000574", "Thick eyebrow", "observed")]
    }
    
    #[rstest]
    fn test_rearrange_vector() {
        let tid1 = TermId::from_str("HP:0000001").unwrap();
        let tid2 = TermId::from_str("HP:0000002").unwrap();
        let tid3 = TermId::from_str("HP:0000003").unwrap();
        let tid4 = TermId::from_str("HP:0000004").unwrap();
        let tid5 = TermId::from_str("HP:0000005").unwrap();
        let tid42 = TermId::from_str("HP:00000042").unwrap();
        let tid43 = TermId::from_str("HP:00000043").unwrap();
        let v1 = vec![tid1.clone(), tid2.clone(), tid3.clone(), tid4.clone(), tid5.clone()];
        let v2 = vec![tid1.clone(), tid2.clone(), tid42.clone(), tid3.clone(), tid43.clone(),tid4.clone(), tid5.clone()];
        // order of the original TIDs (v1) in the rearranged vector v2
        let expected_order = vec![0,1,3,5,6];
        let observed_order = PpktRow::get_update_vector(&v1, &v2);
        assert_eq!(expected_order, observed_order);
        // Now check we fill in with na
        let hpo_values = vec!["observed".to_string(),"observed".to_string(),"observed".to_string(),"observed".to_string(),"observed".to_string()];
        let expected = vec!["observed".to_string(),"observed".to_string(),"na".to_string(), "observed".to_string(),"na".to_string(), "observed".to_string(),"observed".to_string()];
        let observed = PpktRow::reorder_or_fill_na(&hpo_values, &expected_order, expected.len());
        assert_eq!(expected_order, observed_order);


    }


}

