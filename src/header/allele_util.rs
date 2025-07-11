
use std::collections::HashSet;

use once_cell::sync::Lazy;
use polars::series::implementations;
use regex::Regex;
use lazy_static::lazy_static;

use crate::error::{self, Error, Result};

impl Error {
    fn hgnc_error<T>(val: &str) -> Self
    {
        Error::AlleleError { msg: format!("{}", val) }
    }

    fn empty_allele() -> Self {
        Error::AlleleError{ msg: format!("HGVS cannot be empty")}
    }
}

pub static SUBSTITUTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(c|n)\.\d+[ACGT]+>[ACGT]+$").unwrap()
});

pub static INSERTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(c|n)\.\d+_\d+ins[ACGT]+$").unwrap()
});

pub static DELINS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(c|n)\.\d+_\d+delins[A-Za-z0-9]+$").unwrap()
});

pub static DEL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(c|n)\.\d+(?:_\d+)?del+$").unwrap()
});

pub static DUPLICATION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(c|n)\.\d+(?:_\d+)?dup$").unwrap()
});


lazy_static! {
    pub static ref ALLOWED_STRUCTURAL_PREFIX: HashSet<String> =  {
        let mut set = HashSet::new();
        set.insert("DEL".to_string());
        set.insert("DUP".to_string());
        set.insert("INV".to_string());
        set.insert("INS".to_string());
        set.insert("TRANSL".to_string());
        set
    };

}

/// We will be sending all HGVS variants to variant validator. Here, we just do 
/// a rough screening to reject some obvious mistakes.
/// - must start with c./n.
/// - must not contain whitespace
/// - must contain at least one digit
/// - If has '>', must have bases before and after
/// - If ins insertion, must have bases after 'ins'
pub fn is_plausible_hgvs(hgvs: &str) -> bool {
    if !(hgvs.starts_with("c.") || hgvs.starts_with("n.")) {
        return false;
    }
    if hgvs.contains(char::is_whitespace) {
        return false;
    }
    if !hgvs.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Some(pos) = hgvs.find('>') {
        // get the characters before and after '>'
        let (before, after) = (&hgvs[..pos], &hgvs[pos + 1..]);
        if !before.chars().rev().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
        if !after.chars().take_while(|c| c.is_ascii_alphabetic()).all(|c| "ACGT".contains(c)) {
            return false;
        }
    }

    if let Some(pos) = hgvs.find("ins") {
        let after = &hgvs[pos + 3..]; // after 'ins'
        if after.is_empty() || !after.chars().all(|c| "ACGT".contains(c)) {
            return false;
        }
    }
    true
}


pub fn check_valid_hgvs(value: &str) -> std::result::Result<(), String> {
    if SUBSTITUTION_RE.is_match(value) {
        return Ok(());
    }
    if INSERTION_RE.is_match(value) {
        return Ok(());
    }
    if DELINS_RE.is_match(value) {
        return Ok(());
    }
    if DUPLICATION_RE.is_match(value) {
        return Ok(());
    } 
    if DEL_RE.is_match(value) {
        return Ok(());
    }
    Err(format!("Malformed HGVS '{value}'"))
}


pub fn check_valid_structural(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    let prefix = parts[0];
    let suffix = parts[1..].join(":"); // in case the original string contains ":"
    let structural_var = suffix.trim();
    return  ALLOWED_STRUCTURAL_PREFIX.contains(prefix)
}



#[cfg(test)]
mod tests {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("c.6231dup", true)]
    #[case("c.6231_6233dup", true)]
    #[case("c.1932T>A", true)]
    #[case("c.417_418insA", true)]
    #[case("c.112_115delinsG", true)]
    #[case("c.76_78del", true)]  // you allow just 'del' in your logic
    #[case("c.76A>G", true)]
    #[case("c.1177del", true)]
    #[case("c.76_78ins", false)] // missing inserted sequence
    #[case("g.123456A>T", false)] // wrong prefix
    #[case("c.", false)]          // incomplete
    #[case("c.-19_*21del", true)]
    fn test_check_valid_hgvs(#[case] input: &str, #[case] should_pass: bool) {
        let result = is_plausible_hgvs(input);
        assert_eq!(result, should_pass, "Failed on input: {}", input);
    }


   
    
}

// endregion: --- Testsq