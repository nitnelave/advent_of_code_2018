#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;

use boolinator::Boolinator;
use nom::digit;
use std::collections::HashSet;

named!(usize <&str, usize>,
       map!(complete!(digit), |d| d.parse::<usize>().unwrap())
);

#[derive(Debug, From, Copy, Clone, PartialEq, Eq, Default)]
pub struct Registers([usize; 4]);

impl std::ops::Index<usize> for Registers {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

named!(registers <&str, Registers>,
       map!(delimited!(char!('['),
                  count_fixed!(usize, terminated!(usize, opt!(tag_s!(", "))), 4),
                  char!(']')),
            Registers::from)
);

pub struct Operation {
    opcode: usize,
    inputs: [usize; 2],
    output: usize,
}

impl From<[usize; 4]> for Operation {
    fn from(array: [usize; 4]) -> Self {
        Self {
            opcode: array[0],
            inputs: [array[1], array[2]],
            output: array[3],
        }
    }
}

named!(operation <&str, Operation>,
       map!(count_fixed!(usize, terminated!(usize, opt!(char!(' '))), 4), Operation::from)
);

/// Inputs, operation, outputs.
type SampleOperation = (Registers, Operation, Registers);

named!(sample_operation <&str, SampleOperation>,
       do_parse!(
           tag_s!("Before: ") >>
           input: registers >>
           char!('\n') >>
           op: operation >>
           char!('\n') >>
           tag_s!("After:  ") >>
           output: registers >>
           char!('\n') >>
           ((input, op, output))
        )
);

named!(sample_operations <&str, Vec<SampleOperation>>,
       separated_list!(char!('\n'), sample_operation)
);

named!(program <&str, Vec<Operation>>,
       many1!(terminated!(operation, opt!(char!('\n'))))
);

named!(parse_all <&str, (Vec<SampleOperation>, Vec<Operation>)>,
       separated_pair!(sample_operations, tag_s!("\n\n\n"), program)
);

pub fn parse_input(input: &str) -> Result<(Vec<SampleOperation>, Vec<Operation>), nom::Err<&str>> {
    parse_all(input).map(|t| t.1)
}

mod ops {
    use super::Registers;
    fn to_usize(b: bool) -> usize {
        if b {
            1
        } else {
            0
        }
    }

    pub fn addr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] + r[i[1]];
    }
    pub fn addi(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] + i[1];
    }
    pub fn mulr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] * r[i[1]];
    }
    pub fn muli(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] * i[1];
    }
    pub fn banr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] & r[i[1]];
    }
    pub fn bani(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] & i[1];
    }
    pub fn borr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] | r[i[1]];
    }
    pub fn bori(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]] | i[1];
    }
    pub fn setr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = r[i[0]]
    }
    pub fn seti(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = i[0]
    }
    pub fn gtir(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(i[0] > r[i[1]]);
    }
    pub fn gtrr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(r[i[0]] > r[i[1]]);
    }
    pub fn gtri(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(r[i[0]] > i[1]);
    }
    pub fn eqir(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(i[0] == r[i[1]]);
    }
    pub fn eqrr(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(r[i[0]] == r[i[1]]);
    }
    pub fn eqri(i: [usize; 2], o: usize, r: &mut Registers) {
        r[o] = to_usize(r[i[0]] == i[1]);
    }
}

type OpType = &'static Fn([usize; 2], usize, &mut Registers) -> ();
const OP_LIST: [OpType; 16] = [
    &ops::addr,
    &ops::addi,
    &ops::mulr,
    &ops::muli,
    &ops::banr,
    &ops::bani,
    &ops::borr,
    &ops::bori,
    &ops::setr,
    &ops::seti,
    &ops::gtir,
    &ops::gtri,
    &ops::gtrr,
    &ops::eqir,
    &ops::eqri,
    &ops::eqrr,
];

/// Returns whether `op` matches the `sample`.
fn op_matches(op: OpType, sample: &SampleOperation) -> bool {
    let mut r = sample.0;
    op(sample.1.inputs, sample.1.output, &mut r);
    r == sample.2
}

/// Returns the number of possible operations that match this sample.
fn num_op_matches(sample: &SampleOperation) -> usize {
    OP_LIST.iter().filter(|&o| op_matches(o, sample)).count()
}

/// Returns the number of samples that match at least 3 operations.
pub fn num_very_ambiguous_ops(samples: &[SampleOperation]) -> usize {
    samples.iter().filter(|s| num_op_matches(s) >= 3).count()
}

/// Initialize a fixed-sized array from a function.
fn initialize_array_16<T>(f: &Fn(usize) -> T) -> [T; 16] {
    unsafe {
        // Create an uninitialized array.
        let mut array: [T; 16] = std::mem::uninitialized();

        for (i, element) in array.iter_mut().enumerate() {
            // Overwrite `element` without running the destructor of the old value.
            // Since Foo does not implement Copy, it is moved.
            std::ptr::write(element, f(i))
        }

        array
    }
}

/// Back-tracking algorithm to find a possible assignment, given a list of possibilities and the
/// set of elements already assigned.
fn find_match_rec(
    possibilities: &[HashSet<usize>],
    seen: &mut HashSet<usize>,
) -> Option<Vec<usize>> {
    if possibilities.is_empty() {
        return Some(Vec::new());
    }
    // Possibilities for this slot that haven't been assigned yet.
    for &p in possibilities[0].difference(&seen.clone()) {
        seen.insert(p);
        // Try to pick `p`, recurse.
        let res = find_match_rec(&possibilities[1..], seen);
        seen.remove(&p);
        // If it worked, return the result, otherwise keep trying.
        // It will only work once all the slots have been assigned.
        if let Some(mut v) = res {
            v.insert(0, p);
            return Some(v);
        }
    }
    None
}

/// Find a match for the list of possibilities, then return the corresponding list of operations.
fn find_match(possibilities: &[HashSet<usize>; 16]) -> [OpType; 16] {
    let mut seen = HashSet::new();
    let res = find_match_rec(possibilities, &mut seen).expect("No matching found");
    initialize_array_16(&|i| OP_LIST[res[i]])
}

/// Match the opcodes to the operations using the list of `samples`.
fn match_ops(samples: &[SampleOperation]) -> [OpType; 16] {
    let mut ops_possibilities = initialize_array_16(&|_| (0..16).collect::<HashSet<usize>>());
    // (opcode -> HashSet<OP_LIST index>)
    samples
        .iter()
        .map(|s| {
            (
                s.1.opcode,
                OP_LIST
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &op)| op_matches(op, s).as_some(i))
                    .collect::<HashSet<_>>(),
            )
        })
        .for_each(|(opcode, set)| {
            ops_possibilities[opcode] = ops_possibilities[opcode]
                .intersection(&set)
                .cloned()
                .collect()
        });
    find_match(&ops_possibilities)
}

/// Using the `samples`, figure out the opcodes, then run the `program` and return register 0.
pub fn execute_program(samples: &[SampleOperation], program: &[Operation]) -> usize {
    let op_table = match_ops(samples);
    let mut registers = Registers::default();
    program
        .iter()
        .for_each(|op| op_table[op.opcode](op.inputs, op.output, &mut registers));
    registers[0]
}
