use crate::rphetools_traits::TableCell;

///! Deceased
/// This module represents data in the deceased table column
/// Valid values are "yes", "no", "na" (NotAvailable)

#[derive(Debug, PartialEq)]
pub enum Deceased {
    Yes,
    No,
    NotAvailable,
}

pub struct DeceasedTableCell {
    deceased: Deceased,
}

impl DeceasedTableCell {
    pub fn new<S: Into<String> >(value: S) -> Result<Self, String> {
        match value.into().as_str() {
            "yes" =>  Ok(DeceasedTableCell{deceased: Deceased::Yes}),
            "no" =>  Ok(DeceasedTableCell{deceased:  Deceased::No}),
            "na" =>  Ok(DeceasedTableCell{deceased:  Deceased::NotAvailable}),
            other => Err(format!("Unrecognized symbol in deceased column: '{}'", other))
        }
    }

    pub fn is_deceased(&self) -> bool {
        self.deceased  == Deceased::Yes
    }

    pub fn is_alive(&self) -> bool {
        self.deceased  == Deceased::No
    }

}

impl TableCell for DeceasedTableCell {
    fn value(&self) -> String {
        match self.deceased {
            Deceased::Yes => "yes".to_string(),
            Deceased::No => "no".to_string(),
            Deceased::NotAvailable => "na".to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
  
    #[test]
    fn test_construct() {
        let tests = vec![("yes", Deceased::Yes),
            ("no", Deceased::No),
            ("na", Deceased::NotAvailable)];
        for test in tests {
            let deceased = DeceasedTableCell::new(test.0);
            assert!(deceased.is_ok());
            let deceased = deceased.unwrap();
            assert_eq!(test.1, deceased.deceased);
        }
    }

    #[test]
    fn test_invalid() {
        let tests = vec![" yes", "n", ""];
        for test in tests {
            let deceased = DeceasedTableCell::new(test);
            assert!(deceased.is_err())
        }
    }

}