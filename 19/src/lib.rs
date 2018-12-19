#[macro_use]
extern crate nom;

mod ops;

use nom::digit;

named!(usize <&str, usize>,
       map!(complete!(digit), |d| d.parse::<usize>().unwrap())
);

#[derive(Default)]
pub struct Registers([usize; 6]);

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

struct Operation {
    opcode: ops::OpCode,
    inputs: [usize; 2],
    output: usize,
}

impl Operation {
    /// Apply the operation on the registers.
    fn run(&self, registers: &mut Registers) {
        self.opcode.run(self.inputs, self.output, registers);
    }
}

pub struct Program {
    /// Index of the register that maps to the instruction pointer.
    ip: usize,
    ops: Vec<Operation>,
}

named!(op_code <&str, ops::OpCode>,
       map!(alt!(
           tag_s!("addr") |
           tag_s!("addi") |
           tag_s!("mulr") |
           tag_s!("muli") |
           tag_s!("banr") |
           tag_s!("bani") |
           tag_s!("borr") |
           tag_s!("bori") |
           tag_s!("setr") |
           tag_s!("seti") |
           tag_s!("gtir") |
           tag_s!("gtri") |
           tag_s!("gtrr") |
           tag_s!("eqir") |
           tag_s!("eqri") |
           tag_s!("eqrr")), |o| ops::OpCode::from(o))
);

named!(operation <&str, Operation>,
       do_parse!(
           op: op_code >>
           char!(' ') >>
           in1: usize >>
           char!(' ') >>
           in2: usize >>
           char!(' ') >>
           out: usize >>
           (Operation { opcode: op, inputs: [in1, in2], output: out })
           )
);

named!(program <&str, Program>,
       do_parse!(
           tag_s!("#ip ") >>
           ip: usize >>
           char!('\n') >>
           ops: many1!(complete!(terminated!(operation, char!('\n')))) >>
           (Program { ip, ops })
        )
);

pub fn parse_input(input: &str) -> Result<Program, nom::Err<&str>> {
    program(input).map(|r| r.1)
}

fn run_instruction(program: &Program, registers: &mut Registers) -> bool {
    let op = program.ops.get(registers[program.ip]);
    if op.is_none() {
        return true;
    }
    op.unwrap().run(registers);
    // Increase the program counter.
    registers[program.ip] += 1;
    false
}

/// This value is where the reference number (the one we factor) is stored. It depends on the user.
/// You can detect it by checking which register never changes after the first few instructions.
const TARGET_REGISTER: usize = 3;

/// The program given as input computes the sum of the factors of target register.
pub fn run_program(program: &Program, init_value: usize) -> usize {
    let mut reg = Registers::default();
    reg[0] = init_value;
    loop {
        if reg[program.ip] == 1 {
            return sum_factors(reg[TARGET_REGISTER]);
        }
        if run_instruction(program, &mut reg) {
            break;
        }
    }
    // Technically, the register holding the program counter is not increased until the beginning
    // of the next operation.
    reg[program.ip] -= 1;
    reg[0]
}

/// Returns the sum of the factors of `value`.
fn sum_factors(value: usize) -> usize {
    println!("Finding factors of {}", value);
    (1..=value).filter(|&i| value % i == 0).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_program_test() {
        let program = parse_input(include_str!("../input")).expect("Failed to parse");
        assert_eq!(run_program(&program, 0), 2160);
    }
    #[test]
    fn sum_factors_test() {
        assert_eq!(sum_factors(12), 28);
        assert_eq!(sum_factors(920), 2160);
    }
}
