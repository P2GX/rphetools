//! Pyphetools Template
//! 
//! The struct that contains all data needed to create or edit a cohort of phenopackets
//! in "pyphetools" format, and to export GA4GH Phenopackets.

use std::{collections::HashMap, fmt::format, str::FromStr, vec};

use ontolius::{ontology::{csr::FullCsrOntology, OntologyTerms}, term::{simple::SimpleMinimalTerm, MinimalTerm}, Identified, TermId};


use crate::{disease_gene_bundle::DiseaseGeneBundle, hpo::hpo_term_arranger::HpoTermArranger, phetools_qc::PheToolsQc, pptcolumn::{header_duplet::HeaderDuplet, ppt_column::PptColumn}};
use crate::error::{self, Error, Result};

pub enum TemplateType {
    Mendelian,
    Melded
}

/// All data needed to edit a cohort of phenopackets or export as GA4GH Phenopackets
pub struct PptTemplate<'a> {
    disease_gene_bundle: DiseaseGeneBundle,
    columns: Vec<PptColumn>,
    template_type: TemplateType,
    ptools_qc: PheToolsQc<'a>,
}


impl Error {

    fn empty_template(nlines: usize) -> Self {
        let msg = format!("Valid template must have at least three rows (at least one data row) but template has only {nlines} rows");
        Error::TemplateError { msg }
    }
    fn unequal_row_lengths() -> Self {
        let msg = format!("Not all rows of template have the same number of fields");
        Error::TemplateError { msg }
    }

   
}




impl<'a> PptTemplate<'a> {



    /// Create the initial pyphetools template (Table) with empty values so the curator can start to make
    /// a template with cases for a specific cohort
    /// Todo: Figure out the desired function signature.
    pub fn create_pyphetools_template_mendelian(
        dg_bundle: DiseaseGeneBundle,
        hpo_term_ids: Vec<TermId>,
        hpo: &'a FullCsrOntology,
        ) ->  Result<Self> {
            

            let mut smt_list: Vec<SimpleMinimalTerm> = Vec::new();
            for hpo_id in hpo_term_ids {
                match hpo.term_by_id(&hpo_id) {
                    Some(term) => { 
                        let smt = SimpleMinimalTerm::new(term.identifier().clone(), term.name(), vec![], false);
                        smt_list.push(smt);},
                    None => { return Err(Error::HpIdNotFound { id: hpo_id.to_string() } ); }
                }
            }
            let column_result = Self::get_ppt_columns(&smt_list, hpo);
            match column_result {
                 // nrows is 2 at this point - we have initialized the two header rows
                Ok(columns) => {
                     Ok(Self {
                        disease_gene_bundle: dg_bundle,
                        columns: columns,
                        template_type: TemplateType::Mendelian,
                        ptools_qc: PheToolsQc::new(hpo)
                    })
                },
                Err(e) => Err(e)
            }
        }


    pub fn get_ppt_columns (
        hpo_terms: &Vec<SimpleMinimalTerm>, 
        hpo:&'a FullCsrOntology
    ) -> Result<Vec<PptColumn>> {
        let empty_col: Vec<String> = vec![]; // initialize to empty column
        let mut column_list: Vec<PptColumn> = vec![];
        column_list.push(PptColumn::pmid(&empty_col));
        column_list.push(PptColumn::title(&empty_col));
        column_list.push(PptColumn::individual_id(&empty_col));
        column_list.push(PptColumn::individual_comment(&empty_col));
        column_list.push(PptColumn::disease_id(&empty_col));
        column_list.push(PptColumn::disease_label(&empty_col));
        column_list.push(PptColumn::hgnc(&empty_col));
        column_list.push(PptColumn::gene_symbol(&empty_col));
        column_list.push(PptColumn::transcript(&empty_col));
        column_list.push(PptColumn::allele_1(&empty_col));
        column_list.push(PptColumn::allele_2(&empty_col));
        column_list.push(PptColumn::variant_comment(&empty_col));
        column_list.push(PptColumn::age_of_onset(&empty_col));
        column_list.push(PptColumn::age_at_last_encounter(&empty_col));
        column_list.push(PptColumn::deceased(&empty_col));
        column_list.push(PptColumn::sex(&empty_col));
        column_list.push(PptColumn::separator(&empty_col));

        // Arrange the HPO terms in a sensible order.
        let mut hpo_arranger = HpoTermArranger::new(hpo);
        let term_id_to_label_d: HashMap<TermId, String> = hpo_terms
            .iter()
            .map(|term| (term.identifier().clone(), term.name().to_string()))
            .collect();
        let term_ids: Vec<TermId> = term_id_to_label_d.keys().cloned().collect();
        let arranged_term_ids = hpo_arranger.arrange_terms(&term_ids);

        for tid in arranged_term_ids {
            let result = term_id_to_label_d.get(&tid);
            match result {
                Some(name) => column_list.push(PptColumn::hpo_term(name, &tid)),
                None => return Err(Error::HpIdNotFound { id: tid.to_string() }),
            }
        }
        /* todo QC headers */
        return Ok(column_list);
    }

    /// get the total number of rows (which is 2 for the header plus the number of phenopacket rows)
    fn nrows(&self) -> Result<usize> {
        match self.columns.get(0) {
            Some(col0) => Ok(col0.phenopacket_count() + 2 ),
            None => Err(Error::TemplateError { msg: format!("Could not extract column zero") })
        }
    }

    /// A function to export a Vec<Vec<String>> matrix from the data
    /// 
    /// # Returns
    ///     
    /// - `Ok(Vec<Vec<String>>)`: A 2d matrix of owned strings representing the data in the template.
    /// - `Err(std::io::Error)`: If an error occurs while transforming the data into a String matrix.
    pub fn get_string_matrix(&self) -> Result<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let nrows = self.nrows()?;
        for idx in 0..nrows {
            let mut row: Vec<String> = Vec::new();
            for col in &self.columns {
                match col.get(idx) {
                    Ok(data) => row.push(data),
                    Err(e) => {
                        return Err(Error::Custom(format!("Could not retrieve column at index {idx}")));
                    }
                }
            }
            rows.push(row);
        }
        Ok(rows)
    }


    /// Extract the disease and gene information from the String template (e.g., from an Excel file)
    /// 
    /// In most cases, we expect only one disease, but for melded genetic diagnoses we expect two
    /// We inspect the first two header rows to determine if a template has one or two diseases.
    fn extract_disease_gene_bundles(matrix: &Vec<Vec<String>>) -> Vec<DiseaseGeneBundle> {



        vec![]
    }


    pub fn from_string_matrix(matrix: Vec<Vec<String>>, hpo: &'a FullCsrOntology) -> Result<Self> {
        if matrix.len() < 3 {
            return Err(Error::empty_template(matrix.len()));
        }
        // check equal length of all rows
        let row_len = matrix[0].len();
        if ! matrix.iter().all(|v| v.len() == row_len) {
            return Err(Error::unequal_row_lengths());
        }
        let hdup_list = HeaderDuplet::extract_from_string_matrix(&matrix)?;
        /// TODO separate for Mendelian and Melded here
        let ptools_qc = PheToolsQc::new(hpo);
        ptools_qc.is_valid_mendelian_header(&hdup_list)?;
        // transpose the String matrix so we can create PptColumns
        let mut columns = vec![Vec::with_capacity(matrix.len()); row_len];
        for row in matrix {
            for (col_idx, value) in row.into_iter().enumerate() {
                columns[col_idx].push(value);
            }
        }
        let mut column_list: Vec<PptColumn> = vec![];
        let disease_id_col = PptColumn::disease_id(&columns[4]);
        let disease_id_str = disease_id_col.get_unique()?;
        let disease_id_tid = TermId::from_str(&disease_id_str).map_err(|e| Error::termid_parse_error(&disease_id_str))?;
        let disease_label_col = PptColumn::disease_label(&columns[5]);
        let disease_label_str = disease_label_col.get_unique()?;
        let hgnc_col = PptColumn::hgnc(&columns[6]);
        let hgnc_str = hgnc_col.get_unique()?;
        let hgnc_tid = TermId::from_str(&hgnc_str).map_err(|e| Error::termid_parse_error(&hgnc_str))?;
        let gene_symbol_col = PptColumn::gene_symbol(&columns[7]);
        let gene_symbol_str = gene_symbol_col.get_unique()?;
        let transcript_col = PptColumn::transcript(&columns[8]);
        let transcript_str = transcript_col.get_unique()?;
        let dg_bundle = DiseaseGeneBundle::new(&disease_id_tid, disease_label_str, &hgnc_tid, gene_symbol_str, transcript_str)?;
        column_list.push(PptColumn::pmid(&columns[0]));
        column_list.push(PptColumn::title(&columns[1]));
        column_list.push(PptColumn::individual_id(&columns[2]));
        column_list.push(PptColumn::individual_comment(&columns[3]));
        column_list.push(disease_id_col);
        column_list.push(disease_label_col);
        column_list.push(hgnc_col);
        column_list.push(gene_symbol_col);
        column_list.push(transcript_col);
        column_list.push(PptColumn::allele_1(&columns[9]));
        column_list.push(PptColumn::allele_2(&columns[10]));
        column_list.push(PptColumn::variant_comment(&columns[11]));
        column_list.push(PptColumn::age_of_onset(&columns[12]));
        column_list.push(PptColumn::age_at_last_encounter(&columns[13]));
        column_list.push(PptColumn::deceased(&columns[14]));
        column_list.push(PptColumn::sex(&columns[15]));
        column_list.push(PptColumn::separator(&columns[16]));
        // Every column after this must be an HPO column
        // We must have at least one HPO column for the template to be valid
        if row_len < 18 {
            return Err(Error::TemplateError { msg: format!("No HPO column found (number of columns: {})", row_len) });
        }
        for i in 17..row_len {
            let hp_column = PptColumn::hpo_term_from_column(&columns[i]);
            column_list.push(hp_column);
        }
        Ok(Self {
            disease_gene_bundle: dg_bundle,
            columns: column_list,
            template_type: TemplateType::Mendelian,
            ptools_qc: ptools_qc
        })

    }
    
}
