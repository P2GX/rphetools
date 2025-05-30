mod common;

use std::sync::Arc;

use ontolius::ontology::csr::FullCsrOntology;
use rphetools::PheTools;
use rstest::rstest;
use common::hpo;
use common::matrix;
use zip::result;

/// Make sure that our test matrix is valid before we start changing fields to check if we pick up errors
#[rstest]
fn test_valid_input(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
}


/// Make sure we do not inadvertently change anything by loading the matrix
#[rstest]
fn check_round_trip(matrix: Vec<Vec<String>>, hpo: Arc<FullCsrOntology>) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let loaded_matrix = phetools.get_string_matrix().unwrap();
    assert_eq!(original_matrix, loaded_matrix);
}


/// Check whether trying to set an invalid value leads to an Error
#[rstest]
#[case(2,17, "+", "Malformed entry for Failure to thrive (HP:0001508): '+'")]
fn check_setting_invalid_value_single_test(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str,
    #[case] error_msg: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(error_msg, err);
}



/// Check whether trying to set an invalid value leads to an Error
#[rstest]
#[case(2,0, "PMD123", "Invalid CURIE with no colon: 'PMD123'")]
#[case(2,1, "A Variant in ZSWIM6 ", "Trailing whitespace in 'A Variant in ZSWIM6 '")]
#[case(2,1, "A Variant  in ZSWIM6", "Consecutive whitespace in 'A Variant  in ZSWIM6'")]
#[case(2,2, "Individual/A", "Forbidden character '/' found in label 'Individual/A'")]
#[case(2,2, "Individual) A", "Forbidden character ')' found in label 'Individual) A'")]
#[case(2,2, "Individual(A)", "Forbidden character '(' found in label 'Individual(A)'")]
#[case(2,4, "OMIM617865", "Invalid CURIE with no colon: 'OMIM617865'")]
#[case(2,4, "OMIM:17865", "OMIM identifiers must have 6 digits: 'OMIM:17865'")]
#[case(2,4, "MONDO76617865", "Invalid CURIE with no colon: 'MONDO76617865'")]
#[case(2,5, "Neurodevelopmental disorder ", "Trailing whitespace in 'Neurodevelopmental disorder '")]
#[case(2,5, "Neurodevelopmental  disorder", "Consecutive whitespace in 'Neurodevelopmental  disorder'")]
#[case(2,5, "", "Value must not be empty")]
#[case(2,6, "", "Empty CURIE")]
#[case(2,6, "HGNC:29316 ", "Contains stray whitespace: 'HGNC:29316 '")]
#[case(2,6, "HGNY:29316", "HGNC id has invalid prefix: 'HGNY:29316'")]
#[case(2,7, "", "Value must not be empty")]
#[case(2,7, "ZSWIM6 ", "Trailing whitespace in 'ZSWIM6 '")]
#[case(2,8, "", "Value must not be empty")]
#[case(2,8, "NM_020928", "Transcript 'NM_020928' is missing a version")]
#[case(2,9, "", "Value must not be empty")]
#[case(2,9, "c.2737C >T", "Malformed allele 'c.2737C >T'")]
#[case(2,9, "c.2737C > T", "Malformed allele 'c.2737C > T'")]
#[case(2,9, "2737C>T", "Malformed allele '2737C>T'")]
#[case(2,9, "c2737C>T", "Malformed allele 'c2737C>T'")]
#[case(2,9, "c.2737CT", "Malformed allele 'c.2737CT'")]
#[case(2,10, "", "Value must not be empty")]
#[case(2,10, "nan", "Malformed allele_2 field: 'nan'")]
#[case(2,10, "2737C>T", "Malformed allele_2 field: '2737C>T'")]
#[case(2,12, "Infantileonset", "Malformed age_of_onset 'Infantileonset'")]
#[case(2,12, "", "age_of_onset must not be empty")]
#[case(2,13, "Infantileonset", "Malformed age_at_last_encounter 'Infantileonset'")]
#[case(2,13, "", "age_at_last_encounter must not be empty")]
#[case(2,14, "", "deceased must not be empty")]
#[case(2,14, "Yes", "Malformed deceased entry: 'Yes'")]
#[case(2,15, "", "sex must not be empty")]
#[case(2,15, "male", "Malformed entry in sex field: 'male'")]
#[case(2,16, "", "HPO (separator) must not be empty")]
#[case(2,16, "nan", "Malformed HPO (separator) entry: 'nan'")]
#[case(2,17, "+", "Malformed entry for Failure to thrive (HP:0001508): '+'")]
#[case(2,17, "Observed", "Malformed entry for Failure to thrive (HP:0001508): 'Observed'")]
#[case(2,17, "-", "Malformed entry for Failure to thrive (HP:0001508): '-'")]
#[case(2,17, "exc", "Malformed entry for Failure to thrive (HP:0001508): 'exc'")]
fn check_setting_invalid_value(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str,
    #[case] error_msg: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(error_msg, err);
}


/// Check that setting a valid value does not lead to an Error
#[rstest]
#[case(2,17, "")]
fn check_setting_valid_value_single_test(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_ok());

}



/// Check that setting a valid value does not lead to an Error
#[rstest]
#[case(2,0, "PMID:123")]
#[case(2,1, "A Variant in ZSWIM6")]
#[case(2,2, "Individual:A")]
#[case(2,2, "Individual A")]
#[case(2,2, "Individual_A")]
#[case(2,4, "OMIM:617865")]
#[case(2,4, "MONDO:76617865")]
#[case(2,5, "Neurodevelopmental disorder")]
#[case(2,6, "HGNC:29316")]
#[case(2,7, "ZSWIM6")]
#[case(2,8, "NM_020928.42")]
#[case(2,8, "NM_020928.1")]
#[case(2,9, "c.2737C>T")]
#[case(2,9, "DEL: deletion exon 5")]
#[case(2,10, "INV: inversion whole gene")]
#[case(2,10, "TRANSL: trans(chr2q1, chr4p2")]
#[case(2,10, "na")]
#[case(2,12, "Infantile onset")]
#[case(2,13, "Infantile onset")]
#[case(2,14, "yes")]
#[case(2,14, "no")]
#[case(2,15, "U")]
#[case(2,15, "F")]
#[case(2,16, "na")]
#[case(2,17, "")]
#[case(2,17, "observed")]
#[case(2,17, "excluded")]
#[case(2,17, "P24Y")]
#[case(2,17, "")]
fn check_setting_valid_value(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] value: &str) {
    let mut phetools = PheTools::new(hpo);
    let original_matrix = matrix.clone();
    let res = phetools.load_matrix(matrix);
    assert!(res.is_ok());
    let result = phetools.set_value(i, j, value);
    assert!(result.is_ok());
}





/// The headers (rows 0/1) cannot be edited unless it is an HPO column (17 or later)
#[rstest]
#[case(0,0,vec!["not editable".to_string()])]
#[case(1,0,vec!["not editable".to_string()])]
#[case(0,1,vec!["not editable".to_string()])]
#[case(1,1,vec!["not editable".to_string()])]
#[case(0,3,vec!["not editable".to_string()])]
#[case(1,3,vec!["not editable".to_string()])]
#[case(0,10,vec!["not editable".to_string()])]
#[case(1,10,vec!["not editable".to_string()])]
fn test_get_options_header(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] expected_options: Vec<String>) 
{
        let mut phetools = PheTools::new(hpo);
        let res = phetools.load_matrix(matrix);
        assert!(res.is_ok());
        let empty_addtl = vec![];
        let result = phetools.get_edit_options_for_table_cell(i, j, empty_addtl);
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(expected_options.len(), options.len());
        assert_eq!(expected_options, options);
}


#[rstest]
#[case(2,1,vec!["edit".to_string(), "trim".to_string()])]
fn test_get_options_single(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] expected_options: Vec<String>) {
        let mut phetools = PheTools::new(hpo);
        let res = phetools.load_matrix(matrix);
        assert!(res.is_ok());
        let addtl = vec!["additional".to_string()];
        let result = phetools.get_edit_options_for_table_cell(i, j, addtl);
        assert!(result.is_ok());
        let options = result.unwrap();
        let mut expected = expected_options.clone();
        expected.push("additional".to_string());
        if expected != options {
            println!("Expected: {:?}", expected_options);
            println!("Got:      {:?}", options);
        }
        assert_eq!(expected.len(), options.len());
        assert_eq!(expected, options);
}


/// Test the get_options method, which returns operations that can be performed via right-click on a GUI
#[rstest]
#[case(2,0,vec!["edit".to_string(), "remove whitespace".to_string()])] // PMID
#[case(2,1,vec!["edit".to_string(), "trim".to_string()])] // title
#[case(2,2,vec!["edit".to_string(), "trim".to_string()])] // individual
#[case(2,3,vec!["edit".to_string(), "trim".to_string(), "clear".to_string()])] // comment
#[case(2,4,vec!["edit".to_string(), "remove whitespace".to_string()])] // disease id
#[case(2,5,vec!["edit".to_string(), "trim".to_string()])] // disease label
#[case(2,6,vec!["edit".to_string(), "remove whitespace".to_string()])] // HNGC_id
#[case(2,7,vec!["edit".to_string(), "remove whitespace".to_string()])] // gene symbol
#[case(2,8,vec!["edit".to_string(), "remove whitespace".to_string()])] // transcript
#[case(2,9,vec!["edit".to_string(), "remove whitespace".to_string()])] // allele 1
#[case(2,10,vec!["edit".to_string(), "remove whitespace".to_string(), "na".to_string()])] // allele 2
#[case(2,11,vec!["edit".to_string(), "trim".to_string(), "clear".to_string()])] // variant.comment
#[case(2,12,vec!["edit".to_string(), "trim".to_string(), "na".to_string()])] // age of onset
#[case(2,13,vec!["edit".to_string(), "trim".to_string(), "na".to_string()])] // age at last encounter
#[case(2,14,vec!["yes".to_string(), "no".to_string(), "na".to_string()])] // deceased
#[case(2,15,vec!["M".to_string(), "F".to_string(), "O".to_string(), "U".to_string()])] // sex
#[case(2,16,vec!["na".to_string()])] // HPO (separator)
#[case(2,17, vec!["observed".to_string(), "excluded".to_string(), "na".to_string(), "edit".to_string()])]
fn test_get_options_non_header(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] expected_options: Vec<String>) {
        let mut phetools = PheTools::new(hpo);
        let res = phetools.load_matrix(matrix);
        assert!(res.is_ok());
        let addtl = vec!["additional".to_string()];
        let result =  phetools.get_edit_options_for_table_cell(i, j, addtl);
        assert!(result.is_ok());
        let options = result.unwrap();
        let mut expected = expected_options;
        expected.push("additional".to_string());
        assert_eq!(expected.len(), options.len());
        assert_eq!(expected, options);
}


#[rstest]
fn test_trim_operation(mut matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,) {
      
        assert_eq!("Infantile onset", &matrix[2][12]);
        matrix[2][12] = "Infantile onset ".to_string();
        assert_eq!("Infantile onset ", &matrix[2][12]);
        let mut phetools = PheTools::new(hpo);
        let res = phetools.load_matrix(matrix);
        assert!(res.is_ok());
        let res = phetools.execute_operation(2, 12, "trim");
        assert!(res.is_ok());
        let trimmed_matrix = phetools.get_string_matrix().unwrap();
        assert_eq!("Infantile onset", &trimmed_matrix[2][12]);
}


#[rstest]
#[case(2,12, "na", "Infantile onset", "na")] // age_of_onset
#[case(2,13, "na", "P16Y", "na")]// age_at_last_encounter
#[case(2,14, "yes", "na", "yes")]// deceased
#[case(2,15, "female", "M", "F")] // sex
#[case(2,15, "other", "M", "O")] // sex
#[case(2,15, "unknown", "M", "U")] // sex
#[case(2,17, "na", "observed", "na")] // HP:0001508
#[case(2,17, "excluded", "observed", "excluded")] // HP:0001508
fn test_execute_operation(
    matrix: Vec<Vec<String>>, 
    hpo: Arc<FullCsrOntology>,
    #[case] i: usize, 
    #[case] j: usize, 
    #[case] operation: &str,
    #[case] original_value: &str,
    #[case] expected_result: &str) {
        // check the original value is as exected -- this is set in common/mod.rs
        assert_eq!(original_value, matrix[i][j]);
        let original_matrx = matrix.clone();
        let mut phetools = PheTools::new(hpo);
        let res = phetools.load_matrix(matrix);
        assert!(res.is_ok());
        let res = phetools.execute_operation(i, j, operation);
        assert!(res.is_ok());
        let modified_matrix = phetools.get_string_matrix().unwrap();
        let data_row = i + 2; // note that the method uses the data index, which is two more (because of the header)
        assert_eq!(expected_result, modified_matrix[i][j]);
    }