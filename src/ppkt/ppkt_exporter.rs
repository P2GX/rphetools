//! Module to export GA4GH Phenopackets from the information in the template.

use std::collections::HashMap;
use std::process::id;
use std::sync::Arc;

use phenopacket_tools::builders::time_elements::time_element_from_str;
use phenopackets::ga4gh::vrsatile::v1::{Expression, GeneDescriptor, MoleculeContext, VariationDescriptor, VcfRecord};
use phenopackets::schema::v2::core::genomic_interpretation::{Call, InterpretationStatus};
use phenopackets::schema::v2::core::interpretation::ProgressStatus;
use phenopackets::schema::v2::core::{Diagnosis, KaryotypicSex, OntologyClass};
use phenopackets::schema::v2::core::vital_status::Status;
use phenopackets::schema::v2::core::{AcmgPathogenicityClassification, Disease, ExternalReference, GenomicInterpretation, Individual, Interpretation, MetaData, PhenotypicFeature, Sex, TherapeuticActionability, TimeElement, VariantInterpretation, VitalStatus};
use phenopackets::schema::v2::Phenopacket;
use prost_types::value;
use regex::Regex;
use crate::dto::template_dto::GeneVariantBundleDto;
use crate::error::{self, Error, Result};
use crate::hpo::hpo_util;
use crate::template::gene_variant_bundle::GeneVariantBundle;
use crate::variant::hgvs_variant::HgvsVariant;
use crate::variant::structural_variant::StructuralVariant;
use crate::variant::variant_manager::VariantManager;
use crate::variant::variant_util::{self, generate_id};
use phenopacket_tools;
use super::ppkt_row::{self, PpktRow};
use phenopacket_tools::builders::builder::Builder;



const DEFAULT_HGNC_VERSION: &str =  "06/01/25";
const DEFAULT_OMIM_VERSION: &str =  "06/01/25";
const DEFAULT_SEQUENCE_ONTOLOGY_VERSION: &str =  "2024-11-18";
const DEFAULT_GENO_VERSION: &str =  "2023-10-08";

pub struct PpktExporter {
    hpo_version: String,
    so_version: String,
    geno_version: String,
    omim_version: String,
    hgnc_version: String,
    orcid_id: String,
}

impl Error {
    pub fn malformed_time_element(msg: impl Into<String>) -> Self {
        Error::AgeParseError { msg: msg.into() }
    }
}


impl PpktExporter {


    pub fn new(hpo_version: &str, 
                creator_orcid: &str,
    ) -> Self {
        Self::from_versions(
            hpo_version,
            DEFAULT_SEQUENCE_ONTOLOGY_VERSION,
            DEFAULT_GENO_VERSION,
            DEFAULT_OMIM_VERSION,
            DEFAULT_HGNC_VERSION,
            creator_orcid)
    }

    pub fn from_versions(
        hpo_version: &str,
        so_version: &str, 
        geno_version: &str,
        omim_version: &str, 
        hgnc_version: &str ,
        creator_orcid: &str,
    ) -> Self {
        Self{ 
            hpo_version: hpo_version.to_string(), 
            so_version: so_version.to_string(), 
            geno_version: geno_version.to_string(),
            omim_version: omim_version.to_string(), 
            hgnc_version: hgnc_version.to_string(),
            orcid_id: creator_orcid.to_string(),
        }
    }


    /// Create a GA4GH Individual message
    pub fn extract_individual(&self, ppkt_row: &PpktRow) -> Result<Individual> {
        let individual_dto = ppkt_row.get_individual_dto();
        let mut idvl = Individual{ 
            id: individual_dto.individual_id, 
            alternate_ids: vec![], 
            date_of_birth: None, 
            time_at_last_encounter: None, 
            vital_status: None, 
            sex: Sex::UnknownSex.into(), 
            karyotypic_sex: KaryotypicSex::UnknownKaryotype.into(), 
            gender: None, 
            taxonomy: None };
        match individual_dto.sex.as_ref() {
            "M" => idvl.sex = Sex::Male.into(),
            "F" => idvl.sex = Sex::Female.into(),
            "O" => idvl.sex = Sex::OtherSex.into(),
            "U" => idvl.sex = Sex::UnknownSex.into(),
            _ => { return Err(Error::TemplateError { msg: format!("Did not recognize sex string '{}'", idvl.sex) });
            }
        };
        let last_enc = individual_dto.age_at_last_encounter;
        if last_enc != "na" {
            let age = time_element_from_str(&last_enc)
                .map_err(|e| Error::malformed_time_element(e.to_string()))?;
            idvl.time_at_last_encounter = Some(age);
        }
        if individual_dto.deceased == "yes" {
            idvl.vital_status = Some(VitalStatus{ 
                status: Status::Deceased.into(), 
                time_of_death: None, 
                cause_of_death: None, 
                survival_time_in_days: 0 
            });
        } 
        Ok(idvl)

    }

    pub fn hpo_version(&self) -> &str {
        &self.hpo_version
    } 

    pub fn so_version(&self) -> &str {
        &self.so_version
    } 

    pub fn geno_version(&self) -> &str {
        &self.geno_version
    } 

    pub fn omim_version(&self) -> &str {
        &self.omim_version
    } 

    pub fn hgnc_version(&self) -> &str {
        &self.hgnc_version
    } 

    /// TODO possibly the PpktExporter has state (created, etc, also dynamically get the time string)
    pub fn get_meta_data(&self, ppkt_row: &PpktRow) -> Result<MetaData> {

        let created_by = "Earnest B. Biocurator";
        let mut meta_data = Builder::meta_data_now(created_by);
        let hpo = phenopacket_tools::builders::resources::Resources::hpo_version(self.hpo_version());
        let geno = phenopacket_tools::builders::resources::Resources::geno_version(self.geno_version());
        let so = phenopacket_tools::builders::resources::Resources::geno_version(self.so_version());
        let omim = phenopacket_tools::builders::resources::Resources::omim_version(self.omim_version());
        let indvl_dto = ppkt_row.get_individual_dto();
        // TODO add HGNC
        //let hgnc =  phenopacket_tools::builders::resources::Resources::hgnc_version(self.omim_version());
        let ext_res = ExternalReference{ 
            id: indvl_dto.pmid, 
            reference: String::default(), 
            description: indvl_dto.title 
        };
        meta_data.resources.push(hpo);
        meta_data.resources.push(geno);
        meta_data.resources.push(so);
        meta_data.resources.push(omim);
        meta_data.external_references.push(ext_res);
        Ok(meta_data)
    }


    /// Generate the phenopacket identifier from the PMID and the individual identifier
    pub fn get_phenopacket_id(&self, ppkt_row: &PpktRow) -> String {
        let individual_dto = ppkt_row.get_individual_dto();
        let pmid = individual_dto.pmid.replace(":", "_");
        let individual_id = individual_dto.individual_id.replace(" ", "_");
        let ppkt_id = format!("{}_{}", pmid, individual_id);
        let ppkt_id = ppkt_id.replace("__", "_");
        // Replace any non-ASCII characters with _, but remove trailing "_" if it exists.
        let mut sanitized: String = ppkt_id.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .clone().collect();
         // Collapse multiple underscores, if any
        let re = Regex::new(r"_+").unwrap();
        sanitized = re.replace_all(&sanitized, "_").to_string();
        if let Some(stripped) = sanitized.strip_suffix('_') {
            sanitized = stripped.to_string();
        }
        sanitized
    }

    /// TODO extend for multiple diseases
    pub fn get_disease(&self, ppkt_row: &PpktRow) -> Result<Disease> {
        let disease_list = ppkt_row.get_disease_dto_list();
        if disease_list.is_empty() {
            return Err(Error::TemplateError { msg: format!("todo empty disease") });
        }
        let dto = disease_list[0].clone();
        let dx_id = Builder::ontology_class(dto.disease_id, dto.disease_label)
            .map_err(|e| Error::DiseaseIdError{msg:format!("malformed disease id")})?;
        let mut disease = Disease{ 
            term: Some(dx_id), 
            excluded: false, 
            onset: None, 
            resolution: None, 
            disease_stage: vec![], 
            clinical_tnm_finding: vec![], 
            primary_site: None, 
            laterality: None 
        };
        let idl_dto = ppkt_row.get_individual_dto();
        let onset = idl_dto.age_of_onset;
        if onset != "na" {
            let age = time_element_from_str(&onset)
                .map_err(|e| Error::malformed_time_element(e.to_string()))?;
            disease.onset = Some(age);
        };
        Ok(disease)
    }

    fn allele_not_contained(allele: &str) -> String {
        format!("'{allele}' must be validated before exporting to Phenopacket Schema")
    }



    fn get_sv_variant_interpretation(
        gvb: &GeneVariantBundleDto, 
        allele: &str,
        sv: &StructuralVariant
    ) -> VariantInterpretation {
        let gene_ctxt = GeneDescriptor{ 
            value_id: gvb.hgnc_id.clone(), 
            symbol: gvb.gene_symbol.clone(), 
            description: String::default(), 
            alternate_ids: vec![] , 
            alternate_symbols: vec![] , 
            xrefs: vec![] 
            };
        let sv_type = OntologyClass{ 
            id: sv.so_id().to_string(), 
            label: sv.so_label().to_string() 
        };
        let vdesc = VariationDescriptor {
            id: variant_util::generate_id(),
            variation: None,
            label: sv.label().to_string(),
            description: String::default(),
            gene_context: Some(gene_ctxt),
            expressions: vec![],
            vcf_record: None,
            xrefs: vec![],
            alternate_labels: vec![],
            extensions: vec![],
            molecule_context: MoleculeContext::Genomic.into(),
            structural_type: Some(sv_type),
            vrs_ref_allele_seq: String::default(),
            allelic_state: None,
        };
        let vi = VariantInterpretation{ 
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(), 
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(), 
            variation_descriptor: Some(vdesc) 
        };
        vi
    }
    
    fn get_hgvs_variant_interpretation(
            gvb: &GeneVariantBundleDto, 
            allele: &str,
            hgvs: &HgvsVariant) -> VariantInterpretation {
        let gene_ctxt = GeneDescriptor{ 
            value_id: gvb.hgnc_id.clone(), 
            symbol: gvb.gene_symbol.clone(), 
            description: String::default(), 
            alternate_ids: vec![] , 
            alternate_symbols: vec![] , 
            xrefs: vec![] 
            };
        let vcf_record = VcfRecord{ 
            genome_assembly: hgvs.assembly().to_string(), 
            chrom: hgvs.chr().to_string(), 
            pos: hgvs.position() as u64, 
            id: String::default(), 
            r#ref: hgvs.ref_allele().to_string(), 
            alt: hgvs.alt_allele().to_string(), 
            qual: String::default(), 
            filter: String::default(), 
            info: String::default(), 
        };

        let hgvs_c = Expression{ 
            syntax: "hgvs.c".to_string(),
            value: format!("{}:{}", gvb.transcript, allele), 
            version: String::default() 
        };
        let mut expression_list = vec![hgvs_c];
        if let Some(hgvs_g) = hgvs.g_hgvs() {
            let hgvs_g = Expression{
                        syntax: "hgvs.g".to_string(),
                        value: hgvs_g.to_string(),
                        version: String::default(),
                    };
            expression_list.push(hgvs_g);
        };
        

        let vdesc = VariationDescriptor{ 
            id: variant_util::generate_id(), 
            variation: None, 
            label: String::default(), 
            description: String::default(), 
            gene_context: Some(gene_ctxt), 
            expressions: expression_list, 
            vcf_record: Some(vcf_record), 
            xrefs: vec![], 
            alternate_labels: vec![], 
            extensions: vec![], 
            molecule_context: MoleculeContext::Genomic.into(), 
            structural_type: None, 
            vrs_ref_allele_seq: String::default(), 
            allelic_state: None 
        };
        let vi = VariantInterpretation{ 
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(), 
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(), 
            variation_descriptor: Some(vdesc) 
        };
        vi
    }

    fn get_variant_interpretation_list(
        gvb: &GeneVariantBundleDto, 
        hgvs_dict: &HashMap<String, HgvsVariant>,
        structural_dict: &HashMap<String, StructuralVariant>) 
    -> Vec<VariantInterpretation> {
        let mut v_interp_list: Vec<VariantInterpretation> = Vec::new();
        if gvb.allele1 == "na" {
            return v_interp_list;
        }
        if hgvs_dict.contains_key(&gvb.allele1) {
            let hgvs = hgvs_dict.get(&gvb.allele1).unwrap();
            let vinterp = Self::get_hgvs_variant_interpretation(gvb, &gvb.allele1, hgvs);
            v_interp_list.push(vinterp);
        } else if structural_dict.contains_key(&gvb.allele1) {
            let sv = structural_dict.get(&gvb.allele1).unwrap();
            let vinterp = Self::get_sv_variant_interpretation(gvb, &gvb.allele1, sv);
        } else {
            // Assume allele2 is not set if allele1 is
            return v_interp_list
        }
        


        v_interp_list
    }
    
    
    
    pub fn get_interpretation_list(
        &self, 
        ppkt_row: &PpktRow,
        hgvs_dict: &HashMap<String, HgvsVariant>,
        structural_dict: &HashMap<String, StructuralVariant>) 
    -> std::result::Result<Vec<Interpretation>, String> {
        let mut dx_list = ppkt_row.get_disease_dto_list();
        let gdb_list = ppkt_row.get_gene_var_dto_list();
        //TODO for now we just support Mendelian. Need to extend for digenic and Melded
        if dx_list.len() != 1 || gdb_list.len() != 1 {
            return Err("Only mendelian supported TODO".to_ascii_lowercase());
        }
        println!("{}{}-", file!(), line!());
        let gdb_dto = gdb_list.first().unwrap();
        let dx_dto = dx_list.first().unwrap();
        let a1 = &gdb_dto.allele1;
        let a2 = &gdb_dto.allele2;
        if a1 != "na" && ! hgvs_dict.contains_key(a1) && !structural_dict.contains_key(a1) {
            return Err(Self::allele_not_contained(a1));
        }
        if a2 != "na" && ! hgvs_dict.contains_key(a2) && !structural_dict.contains_key(a2) {
            return Err(Self::allele_not_contained(a2));
        }
        let v_interpretations = Self::get_variant_interpretation_list(gdb_dto, hgvs_dict, structural_dict);
        let disease_clz = OntologyClass{
            id: dx_dto.disease_id.clone(),
            label: dx_dto.disease_label.clone(),
        };
        let mut g_interpretations: Vec<GenomicInterpretation> = Vec::new();
        for vi in v_interpretations {
            let gi = GenomicInterpretation{
                subject_or_biosample_id: ppkt_row.get_individual_dto().individual_id.clone(),
                interpretation_status: InterpretationStatus::Causative.into(),
                call: Some(Call::VariantInterpretation(vi))
            };
            g_interpretations.push(gi);
        }
        let diagnosis = Diagnosis{
            disease: Some(disease_clz),
            genomic_interpretations: g_interpretations,
        };
        let i = Interpretation{
            id: generate_id(),
            progress_status: ProgressStatus::Solved.into(),
            diagnosis: Some(diagnosis),
            summary: String::default(),
        };
        let interpretation_list: Vec<Interpretation> = vec![i];
        Ok(interpretation_list)
    }

    


    pub fn get_phenopacket_features(&self, ppkt_row: &PpktRow) -> Result<Vec<PhenotypicFeature>> {
        let dto_list = ppkt_row.get_hpo_term_dto_list()?;
        let mut ppkt_feature_list: Vec<PhenotypicFeature> = Vec::with_capacity(dto_list.len());
        for dto in dto_list {
            if ! dto.is_ascertained() {
                continue;
            }
            let hpo_term = Builder::ontology_class(dto.term_id(), dto.label())
                .map_err(|e| Error::termid_parse_error(dto.term_id()))?;
            let mut pf = PhenotypicFeature{ 
                description: String::default(), 
                r#type: Some(hpo_term), 
                excluded: dto.is_excluded(), 
                severity: None, 
                modifiers: vec![], 
                onset: None,
                resolution: None, 
                evidence: vec![]
            };
            if dto.has_onset() {
                let value = dto.onset()?;
                let ost = time_element_from_str(&value)
                    .map_err(|e| Error::malformed_time_element(value))?;
                pf.onset = Some(ost);
            }
            ppkt_feature_list.push(pf);
        }
        Ok(ppkt_feature_list)
    }


    pub fn extract_phenopacket(
        &self, 
        ppkt_row: &PpktRow, 
        hgvs_dict: &HashMap<String, HgvsVariant>,
        structural_dict: &HashMap<String, StructuralVariant>) 
    -> Result<Phenopacket> {
        if ppkt_row.get_gene_var_dto_list().len() != 1 {
            panic!("NEED TO EXTEND MODEL TO NON MEND. NEED TO EXTEND CACHE KEY FOR GENE-TRANSCRIPT-NAME");
        }
        let interpretation_list = self.get_interpretation_list(ppkt_row, hgvs_dict, structural_dict)?;
        let gv_dto = ppkt_row.get_gene_var_dto_list()[0].clone();
        let allele1 = gv_dto.allele1;
        let allele2= gv_dto.allele2;
        let ppkt = Phenopacket{ 
            id: self.get_phenopacket_id(ppkt_row), 
            subject:  Some(self.extract_individual(ppkt_row)?), 
            phenotypic_features: self.get_phenopacket_features(ppkt_row)?, 
            measurements: vec![], 
            biosamples: vec![], 
            interpretations: interpretation_list, 
            diseases: vec![self.get_disease(ppkt_row)?], 
            medical_actions: vec![], 
            files: vec![], 
            meta_data: Some(self.get_meta_data(ppkt_row)?) 
        };
    
        Ok(ppkt)
    }


}