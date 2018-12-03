use nom::digit;
use std::str::FromStr;

/// Represents a claim made by an elf.
#[derive(Debug, PartialEq)]
pub struct Claim {
    pub id: usize,
    pub coordinates: (usize, usize),
    pub size: (usize, usize),
}

/// Parse an int (unsigned).
named!(usize <&str, usize>,
   map!(map!(digit, FromStr::from_str), Result::unwrap)
);

/// Parse the identifier of the claim.
named!(id <&str, usize>,
    preceded!(tag_s!("#"), usize)
);

/// Parse the coordinates of the top-left corner (format: "1,3").
named!(coordinates <&str, (usize, usize)>,
  separated_pair!(usize, char!(','), usize)
);

/// Parse the size of the claim (format: "1x3").
named!(size <&str, (usize, usize)>,
  separated_pair!(usize, char!('x'), usize)
);

/// Parse the whole claim.
/// e.g. "#1 @ 1,3: 4x4 "
/// Note: the string must end with a space or EOF.
named!(claim <&str, Claim>,
    do_parse!(
        id: id >>
        ws!(char!('@')) >>
        coordinates: coordinates >>
        ws!(char!(':')) >>
        size: size >>
        (Claim {id: id, coordinates: coordinates, size: size})
    )
);

/// Convenience function to parse a claim from a string, adding a space at the end.
pub fn parse_claim(input: &std::string::String) -> Claim {
    // Add a space at the end to "complete" the string.
    claim(&format!("{} ", input)).unwrap().1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_usize() {
        assert_eq!(usize("123 ").unwrap().1, 123);
    }

    #[test]
    fn test_claim() {
        assert_eq!(
            claim("#1 @ 1,3: 4x4 ").unwrap().1,
            Claim {
                id: 1,
                coordinates: (1, 3),
                size: (4, 4),
            }
        );
    }
}
