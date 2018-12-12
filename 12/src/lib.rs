#[macro_use]
extern crate nom;


/// A pot can either be empty or have a plant.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PotState {
    Plant,
    Empty,
}

/// A rule is 5 pot states to match, and the result of the match.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Rule {
    pattern: [PotState; 5],
    result: PotState,
}

/// A ruleset is an optimized structure to match the rules. Since the rules are exhaustive, we can
/// encode each rule as an integer between 0 and 31 with each bit being the state of a pot, in
/// order. The value is then the result of the rule.
struct RuleSet {
    patterns: [PotState; 32],
}

impl RuleSet {
    /// Create the set from a list of rules. It will encode the pattern of each rule into an
    /// integer, and write the value of the rule in the corresponding cell.
    fn new(rules: &[Rule]) -> Self {
        let mut patterns = [PotState::Empty; 32];
        #[cfg(not(test))]
        {
            // A real input should be exactly 32 rules long.
            if rules.len() != 32 {
                panic!("Not enough rules! Only {}", rules.len());
            }
        }
        for r in rules {
            patterns[Self::pattern_to_index(&r.pattern)] = r.result;
        }
        assert!(
            patterns[0] == PotState::Empty,
            "Generating plants from nothing!"
        );
        Self { patterns }
    }

    /// Convert a rule pattern to an index.
    fn pattern_to_index(pots: &[PotState]) -> usize {
        assert!(pots.len() == 5);
        let mut res = 0;
        for p in pots {
            res <<= 1;
            if *p == PotState::Plant {
                res += 1;
            }
        }
        res
    }

    /// Match the pots (a slice of size 5) against the rules, return the result.
    fn matches(&self, pots: &[PotState]) -> PotState {
        assert!(pots.len() == 5);
        self.patterns[Self::pattern_to_index(pots)]
    }
}

/// The state of the board.
#[derive(Debug)]
struct State {
    /// The list of pots we're considering, from index -`position`.
    pots: Vec<PotState>,
    /// The position of the pot 0 in `pots`.
    position: usize,
}

impl State {
    /// Construct a state from a list of pots, padding it left and right.
    fn new(pots: Vec<PotState>) -> Self {
        let mut state = Self { pots, position: 5 };
        for _ in 0..5 {
            state.pots.insert(0, PotState::Empty);
            state.pots.push(PotState::Empty);
        }
        state
    }

    /// Return a blank state of the size needed to fit the next generation.
    fn next_gen_blank_state(&self) -> Self {
        // How many pots should we add on the left to have 5 empty pots before the first plant.
        let to_add_first = 5 -
            self.pots
                .iter()
                .enumerate()
                .take(5)
                .find(|(_, &p)| p == PotState::Plant)
                .map_or(5, |(i, _)| i);
        // Same on the right.
        let to_add_last = 5 -
            self.pots
                .iter()
                .rev()
                .enumerate()
                .take(5)
                .find(|(_, &p)| p == PotState::Plant)
                .map_or(5, |(i, _)| i);
        let new_len = self.pots.len() + to_add_first + to_add_last;
        let pots = std::iter::repeat(PotState::Empty)
            .take(new_len)
            .collect::<Vec<_>>();
        Self {
            pots,
            position: self.position + to_add_first,
        }
    }
}

impl std::fmt::Display for State {
    /// Print a state
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} -- {}",
            self.pots
                .iter()
                .skip(self.position - 3)
                .map(|s| match s {
                    PotState::Plant => '#',
                    PotState::Empty => '.',
                })
                .collect::<String>(),
            self.position
        )
    }
}

/// Parse a single pot state.
named!(state <&str, PotState>,
        map!(alt!(char!('#') | char!('.')), |c| if c == '#' {
            PotState::Plant
        } else {
            PotState::Empty
        })
);

/// Parse the first line of the input, with the inital state.
named!(initial_state <&str, Vec<PotState>>,
       preceded!(tag_s!("initial state: "), many1!(complete!(state)))
);

/// Parse a single rule, e.g. "##..# => .\n". The newline at the end is needed.
named!(rule <&str, Rule>,
       do_parse!(
           pattern: count_fixed!(PotState, state, 5) >>
           tag_s!(" => ") >>
           result: state >>
           opt!(char!('\n')) >>
           (Rule { pattern, result })
));

/// Parse the initial state and the list of rules.
named!(parse_rules <&str, (Vec<PotState>, Vec<Rule>)>,
       pair!(ws!(initial_state),
             many1!(complete!(rule)))
);

/// Parse the input.
fn parse_input_rules(input: &str) -> Result<(Vec<PotState>, Vec<Rule>), nom::Err<&str>> {
    parse_rules(input).map(|(_, t)| t)
}

/// Given a state and the set of rules, return the state corresponding to the next generation.
fn advance_state(state: &State, rules: &RuleSet) -> State {
    assert!(state.pots.len() > 10);
    let mut new_state: State = state.next_gen_blank_state();
    // Don't match the very edges.
    (2..(state.pots.len() - 2)).for_each(|i| {
        // Account for the change of padding.
        new_state.pots[i + new_state.position - state.position] =
            rules.matches(&state.pots[(i - 2)..(i + 3)])
    });
    new_state
}

/// Return the sum of the position of the pots with plants.
fn count_pots(state: &State) -> i64 {
    state
        .pots
        .iter()
        .enumerate()
        .map(|(i, &p)| if p == PotState::Plant {
            (i as i64) - (state.position as i64)
        } else {
            0
        })
        .sum()
}

/// Print the state during tests, for debugging.
fn maybe_print_state(_state: &State) {
    #[cfg(test)]
    {
        println!("{}", _state);
    }
}

/// Given the input, advance for `num_generations` and return the pot count.
/// This method implements a short-circuit: if the difference of pot count between 2 generations is
/// the same for 25 generations in a row, it will assume that it is always going to be the same,
/// and returns the linear projection of the pot count.
pub fn count_pots_from_input(input: &str, num_generations: usize) -> i64 {
    let (initial_state, rules) = parse_input_rules(input).expect("Error parsing input: ");
    let ruleset = RuleSet::new(&rules);
    let mut state = State::new(initial_state);
    let mut diff = 0;
    let mut same_count = 0;
    let mut previous_count = count_pots(&state);
    maybe_print_state(&state);
    for i in 0..num_generations {
        // Compute the next generation.
        state = advance_state(&state, &ruleset);
        maybe_print_state(&state);
        // New count.
        let count = count_pots(&state);
        if count - previous_count == diff {
            // The diff is the same as before, we count it.
            same_count += 1;
            if same_count == 25 {
                println!("Found pattern, stopping at iteration {}", i);
                // We did i + 1 iterations, we have num_generations - (i + 1) iterations left.
                return count + (num_generations - i - 1) as i64 * diff;
            }
        } else {
            // The diff was not the same as before, reset the counter.
            same_count = 0;
            diff = count - previous_count;
        }
        previous_count = count;
    }
    count_pots(&state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::PotState::*;

    const TEST_INPUT: &str = "
initial state: #..#.#..##......###...###

...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #
";

    #[test]
    fn initial_state_test() {
        assert_eq!(
            initial_state("initial state: #..#").unwrap().1,
            vec![Plant, Empty, Empty, Plant]
        );
    }

    #[test]
    fn rule_test() {
        assert_eq!(
            rule("##.#. => #\n").unwrap().1,
            Rule {
                pattern: [Plant, Plant, Empty, Plant, Empty],
                result: Plant,
            }
        );
    }

    #[test]
    fn rules_test() {
        let (_, rules) = parse_input_rules(TEST_INPUT).unwrap();
        assert_eq!(rules.len(), 14);
    }

    #[test]
    fn ruleset_matches_test() {
        {
            let rule = Rule {
                pattern: [Plant, Empty, Empty, Empty, Plant],
                result: Plant,
            };
            let ruleset = RuleSet::new(&[rule]);
            let state = vec![Empty, Plant, Empty, Empty, Empty, Plant, Plant];
            assert_eq!(ruleset.matches(&state[0..5]), Empty);
            assert_eq!(ruleset.matches(&state[1..6]), Plant);
            assert_eq!(ruleset.matches(&state[2..7]), Empty);
        }
        {
            let rule = Rule {
                pattern: [Empty, Plant, Empty, Plant, Empty],
                result: Plant,
            };
            let ruleset = RuleSet::new(&[rule]);
            let state = vec![Empty, Plant, Empty, Empty, Plant, Empty, Plant, Empty, Empty, Empty];
            for i in 0..state.len() - 5 {
                assert_eq!(ruleset.matches(&state[i..(i + 5)]), if i == 3 { Plant } else { Empty });
            }
        }
    }

    #[test]
    fn count_pots_from_input_test() {
        assert_eq!(count_pots_from_input(TEST_INPUT, 20), 325);
    }
}
