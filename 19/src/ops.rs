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

pub enum OpCode {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

impl From<&str> for OpCode {
    fn from(input: &str) -> Self {
        match input {
            "addr" => OpCode::Addr,
            "addi" => OpCode::Addi,
            "mulr" => OpCode::Mulr,
            "muli" => OpCode::Muli,
            "banr" => OpCode::Banr,
            "bani" => OpCode::Bani,
            "borr" => OpCode::Borr,
            "bori" => OpCode::Bori,
            "setr" => OpCode::Setr,
            "seti" => OpCode::Seti,
            "gtir" => OpCode::Gtir,
            "gtri" => OpCode::Gtri,
            "gtrr" => OpCode::Gtrr,
            "eqir" => OpCode::Eqir,
            "eqri" => OpCode::Eqri,
            "eqrr" => OpCode::Eqrr,
            _ => panic!("Invalid opcode"),
        }
    }
}

impl OpCode {
    pub fn run(&self, i: [usize; 2], o: usize, r: &mut Registers) {
        let op = match self {
            OpCode::Addr => addr,
            OpCode::Addi => addi,
            OpCode::Mulr => mulr,
            OpCode::Muli => muli,
            OpCode::Banr => banr,
            OpCode::Bani => bani,
            OpCode::Borr => borr,
            OpCode::Bori => bori,
            OpCode::Setr => setr,
            OpCode::Seti => seti,
            OpCode::Gtir => gtir,
            OpCode::Gtri => gtri,
            OpCode::Gtrr => gtrr,
            OpCode::Eqir => eqir,
            OpCode::Eqri => eqri,
            OpCode::Eqrr => eqrr,
        };
        op(i, o, r);
    }
}
