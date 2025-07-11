
use ga4ghphetools::dto::{case_dto::CaseDto, hpo_term_dto::HpoTermDto};
use rstest::fixture;
use ontolius::{io::OntologyLoaderBuilder, ontology::csr::FullCsrOntology};
use std::{fs::File, io::BufReader, sync::Arc};
use flate2::bufread::GzDecoder;

#[fixture]
pub fn hpo() -> Arc<FullCsrOntology> {
    let path = "resources/hp.v2025-03-03.json.gz";
    let reader = GzDecoder::new(BufReader::new(File::open(path).unwrap()));
    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let hpo = loader.load_from_read(reader).unwrap();
    Arc::new(hpo)
}

/// A matrix representing a PheTools template for OMIM:617865
#[fixture]
pub fn matrix() -> Vec<Vec<String>> {
    let row1: Vec<String> = vec![ 
        "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO", "Failure to thrive", "Tongue thrusting", "Ataxia", "Hypertonia", "Loss of ambulation", "Happy demeanor", "Seizure"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row2: Vec<String> = vec![
        "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na", "HP:0001508", "HP:0100703", "HP:0001251", "HP:0001276", "HP:0002505", "HP:0040082", "HP:0001250" 
    ].into_iter().map(|s| s.to_owned()).collect();
    let row3: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 1", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P16Y", "na", "M", "na", "observed", "observed", "excluded", "observed", "observed", "observed", "observed"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row4: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 2", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P7Y", "yes", "F", "na", "excluded", "observed", "observed", "excluded", "excluded", "observed", "excluded"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row5: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 3", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P4Y", "no", "F", "na", "excluded", "observed", "excluded", "observed", "excluded", "observed", "na"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row6: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 4", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P5Y", "no", "F", "na", "excluded", "excluded", "observed", "excluded", "excluded", "na", "excluded"
    ].into_iter().map(|s| s.to_owned()).collect();
    vec![row1, row2, row3, row4, row5, row6]
}


#[fixture]
pub fn one_case_matrix() -> Vec<Vec<String>> {
      let row1: Vec<String> = vec![ 
        "PMID", "title", "individual_id", "comment", "disease_id", "disease_label", "HGNC_id", "gene_symbol", "transcript", "allele_1", "allele_2", "variant.comment", "age_of_onset", "age_at_last_encounter", "deceased", "sex", "HPO", "Failure to thrive", "Seizure"
    ].into_iter().map(|s| s.to_owned()).collect();
    let row2: Vec<String> = vec![
        "CURIE", "str", "str", "optional", "CURIE", "str", "CURIE", "str", "str", "str", "str", "optional", "age", "age", "yes/no/na", "M:F:O:U", "na", "HP:0001508",  "HP:0001250" 
    ].into_iter().map(|s| s.to_owned()).collect();
    let row3: Vec<String> = vec![
        "PMID:29198722", "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", "p.Arg913Ter Affected Individual 1", "", "OMIM:617865", "Neurodevelopmental disorder with movement abnormalities, abnormal gait, and autistic features", "HGNC:29316", "ZSWIM6", "NM_020928.2", "c.2737C>T", "na", "", "Infantile onset", "P16Y", "na", "M", "na", "observed", "observed"
    ].into_iter().map(|s| s.to_owned()).collect();
     vec![row1, row2, row3]
}

#[fixture]
pub fn case_5_dto() -> CaseDto {
    CaseDto::new(
        "PMID:29198722", //PMID
        "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", //title 
        "p.Arg913Ter Affected Individual 5", // individual_id
         "",  // comment
        "c.2737C>T",  // allele_1
        "na", // allele_2
        "",  // variant.comment
        "Infantile onset", // age_at_onset
        "P3Y", //  age_at_last_encounter
        "na", // deceased
         "F" //sex
    )
}

#[fixture]
pub fn case_6_dto() -> CaseDto {
    CaseDto::new(
        "PMID:29198722", //PMID
        "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", //title 
        "p.Arg913Ter Affected Individual 6", // individual_id
        "",  // comment
        "c.2737C>T",  // allele_1
        "na", // allele_2
        "",  // variant.comment
        "Infantile onset", // age_at_onset
        "P29Y", //  age_at_last_encounter
        "na", // deceased
        "M" //sex
    )
}

#[fixture]
pub fn case_7_dto() -> CaseDto {
    CaseDto::new(
        "PMID:29198722", //PMID
        "A Recurrent De Novo Nonsense Variant in ZSWIM6 Results in Severe Intellectual Disability without Frontonasal or Limb Malformations", //title 
        "p.Arg913Ter Affected Individual 7", // individual_id
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
pub fn thick_eye_brow_excluded_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0000574", "Thick eyebrow", "excluded")
}

#[fixture]
pub fn thick_eye_brow_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0000574", "Thick eyebrow", "observed")
}

#[fixture]
pub fn thick_eye_brow_na_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0000574", "Thick eyebrow", "na")
}


#[fixture]
pub fn flat_occiput_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0005469", "Flat occiput", "observed")
}
#[fixture]
pub fn flat_occiput_excluded_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0005469", "Flat occiput", "excluded")
}
#[fixture]
pub fn flat_occiput_na_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0005469", "Flat occiput", "na")
}
#[fixture]
pub fn join_hypermobility_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0001382", "Joint hypermobility", "observed")
}

#[fixture]
pub fn joint_hypermobility_excluded_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0001382", "Joint hypermobility", "excluded")
}
#[fixture]
pub fn joint_hypermobility_na_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0001382", "Joint hypermobility", "na")
}


#[fixture]
pub fn grand_mal_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0002069", "Bilateral tonic-clonic seizure", "observed")
}
#[fixture]
pub fn grand_mal_excluded_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0002069", "Bilateral tonic-clonic seizure", "excluded")
}

#[fixture]
pub fn strabismus_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0000486", "Strabismus", "observed")
}

#[fixture]
pub fn esotropia_observed_dto() -> HpoTermDto {
    HpoTermDto::new("HP:0000565", "Esotropia", "observed")
}

#[fixture]
pub fn hpo_dto_list_1(
    thick_eye_brow_excluded_dto: HpoTermDto,
    grand_mal_observed_dto: HpoTermDto,
    strabismus_observed_dto: HpoTermDto,
    esotropia_observed_dto: HpoTermDto) -> Vec<HpoTermDto>
{
    vec![thick_eye_brow_excluded_dto, grand_mal_observed_dto, strabismus_observed_dto, esotropia_observed_dto]
}

#[fixture]
pub fn hpo_dto_list_2(
    thick_eye_brow_excluded_dto: HpoTermDto,
    joint_hypermobility_na_dto: HpoTermDto,
    grand_mal_excluded_dto: HpoTermDto,
    esotropia_observed_dto: HpoTermDto,
    flat_occiput_excluded_dto: HpoTermDto) -> Vec<HpoTermDto>
{
    vec![thick_eye_brow_excluded_dto, grand_mal_excluded_dto,  esotropia_observed_dto, joint_hypermobility_na_dto, flat_occiput_excluded_dto]
}