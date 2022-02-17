//! This module represents a run of a Cairo program as a consecution of execution
//! steps, each of which define the execution logic of Cairo instructions

use crate::definitions::*;
use crate::memory::CairoMemory;
use crate::word::{CairoWord, Decomposition};
use ark_ff::FftField;

/// A structure to store program counter, allocation pointer and frame pointer
#[derive(Clone, Copy)]
pub struct CairoPointers<F: FftField> {
    /// Program counter: points to address in memory
    pub pc: F,
    /// Allocation pointer: points to first free space in memory
    pub ap: F,
    /// Frame pointer: points to the beginning of the stack in memory (for arguments)
    pub fp: F,
}

impl<F: FftField> CairoPointers<F> {
    /// Creates a new triple of pointers
    pub fn new(pc: F, ap: F, fp: F) -> CairoPointers<F> {
        CairoPointers { pc, ap, fp }
    }
}

/// A structure to store auxiliary variables throughout computation
pub struct CairoVariables<F: FftField> {
    /// Destination
    dst: Option<F>,
    /// First operand
    op0: Option<F>,
    /// Second operand
    op1: Option<F>,
    /// Result
    res: Option<F>,
    /// Destination address
    dst_addr: F,
    /// First operand address
    op0_addr: F,
    /// Second operand address
    op1_addr: F,
    /// Size of the instruction
    size: F,
}

impl<F: FftField> CairoVariables<F> {
    /// This function creates an instance of a default CairoVariables struct
    pub fn new() -> CairoVariables<F> {
        CairoVariables {
            dst: None,
            op0: None,
            op1: None,
            res: None,
            dst_addr: F::zero(),
            op0_addr: F::zero(),
            op1_addr: F::zero(),
            size: F::zero(),
        }
    }
}
impl<F: FftField> Default for CairoVariables<F> {
    fn default() -> Self {
        Self::new()
    }
}

/// A data structure to store a current step of Cairo computation
pub struct CairoStep<'a, F: FftField> {
    /// current word of the program
    pub mem: &'a mut CairoMemory<F>,
    // comment instr for efficiency
    /// current pointers
    curr: CairoPointers<F>,
    /// (if any) next pointers
    next: Option<CairoPointers<F>>,
    /// state auxiliary variables
    vars: CairoVariables<F>,
}

impl<'a, F: FftField> CairoStep<'a, F> {
    /// Creates a new Cairo execution step from a step index, a Cairo word, and current pointers
    pub fn new(mem: &mut CairoMemory<F>, ptrs: CairoPointers<F>) -> CairoStep<F> {
        CairoStep {
            mem,
            curr: ptrs,
            next: None,
            vars: CairoVariables::new(),
        }
    }

    /// Executes a Cairo step from the current registers
    pub fn execute(&mut self) {
        // This order is important in order to allocate the memory in time
        self.set_op0();
        self.set_op1();
        self.set_res();
        self.set_dst();
        // If the Option<> thing is not a guarantee for continuation of the program, we may be removing this
        let next_pc = self.next_pc();
        let (next_ap, next_fp) = self.next_apfp();
        self.next = Some(CairoPointers::new(
            next_pc.unwrap(),
            next_ap.unwrap(),
            next_fp.unwrap(),
        ));
    }

    /// This function returns the current word instruction being executed
    pub fn instr(&mut self) -> CairoWord<F> {
        CairoWord::new(self.mem.read(self.curr.pc).unwrap())
    }

    /// This function computes the first operand address
    pub fn set_op0(&mut self) {
        let reg = {
            match self.instr().op0_reg() {
                OP0_AP => self.curr.ap, // reads first word from allocated memory
                _ => self.curr.fp,      // reads first word from input stack
            } // no more values than 0 and 1 because op0_reg is one bit
        };
        self.vars.op0_addr = reg + self.instr().off_op0();
        self.vars.op0 = self.mem.read(self.vars.op0_addr);
    }

    /// This function computes the second operand address and content and the instruction size
    /// Panics if the flagset OP1_SRC has more than 1 nonzero bit
    pub fn set_op1(&mut self) {
        let (reg, size) = {
            match self.instr().op1_src() {
                OP1_DBL => (self.vars.op0.unwrap(), F::one()), // double indexing, op0 should be positive for address
                OP1_VAL => (self.curr.pc, F::from(2u32)), // off_op1 will be 1 and then op1 contains an immediate value
                OP1_FP => (self.curr.fp, F::one()),
                OP1_AP => (self.curr.ap, F::one()),
                _ => panic!("Invalid instruction"),
            }
        };
        self.vars.size = size;
        self.vars.op1_addr = reg + self.instr().off_op1(); // apply second offset to corresponding register
        self.vars.op1 = self.mem.read(self.vars.op1_addr);
    }

    /// This function computes the value of the result of the arithmetic operation
    /// Panics if a JNZ instruction is used with an invalid format
    ///     or if the flagset RES_LOG has more than 1 nonzero bit
    pub fn set_res(&mut self) {
        if self.instr().pc_up() == PC_JNZ {
            // jnz instruction
            if self.instr().res_log() == RES_ONE
                && self.instr().opcode() == OPC_JMP_INC
                && self.instr().ap_up() != AP_ADD
            {
                self.vars.res = Some(F::zero()); // "unused"
            } else {
                panic!("Invalid instruction");
            }
        } else if self.instr().pc_up() == PC_SIZ
            || self.instr().pc_up() == PC_ABS
            || self.instr().pc_up() == PC_REL
        {
            // rest of types of updates
            // common increase || absolute jump || relative jump
            if self.instr().res_log() == RES_ONE {
                self.vars.res = self.vars.op1; // right part is single operand
            } else if self.instr().res_log() == RES_ADD {
                self.vars.res = Some(self.vars.op0.unwrap() + self.vars.op1.unwrap());
            // right part is addition
            } else if self.instr().res_log() == RES_MUL {
                self.vars.res = Some(self.vars.op0.unwrap() * self.vars.op1.unwrap());
            // right part is multiplication
            } else {
                panic!("Invalid instruction");
            }
        } else {
            // multiple bits take value 1
            panic!("Invalid instruction");
        }
    }

    /// This function computes the destination address
    pub fn set_dst(&mut self) {
        let reg = {
            match self.instr().dst_reg() {
                DST_AP => self.curr.ap, // read from stack
                _ => self.curr.fp,      // read from parameters
            } // no more values than 0 and 1 because op0_reg is one bit
        };
        self.vars.dst_addr = reg + self.instr().off_dst();
        self.vars.dst = self.mem.read(self.vars.dst_addr);
    }

    /// This function computes the next program counter
    /// Panics if the flagset PC_UP has more than 1 nonzero bit
    pub fn next_pc(&mut self) -> Option<F> {
        if self.instr().pc_up() == PC_SIZ {
            // next instruction is right after the current one
            Some(self.curr.pc + self.vars.size) // the common case
        } else if self.instr().pc_up() == PC_ABS {
            // next instruction is in res
            Some(self.vars.res.unwrap()) // absolute jump
        } else if self.instr().pc_up() == PC_REL {
            // relative jump
            Some(self.curr.pc + self.vars.res.unwrap()) // go to some address relative to pc
        } else if self.instr().pc_up() == PC_JNZ {
            // conditional relative jump (jnz)
            if self.vars.dst == Some(F::zero()) {
                Some(self.curr.pc + self.vars.size) // if condition false, common case
            } else {
                // if condition true, relative jump with second operand
                Some(self.curr.pc + self.vars.op1.unwrap())
            }
        } else {
            panic!("Invalid instruction");
        }
    }

    /// This function computes the next values of the allocation and frame pointers
    /// Panics if in a CALL instruction the flagset AP_UP is incorrect
    ///     or if in any other instruction the flagset AP_UP has more than 1 nonzero bit
    ///     or if the flagset OPCODE has more than 1 nonzero bit
    fn next_apfp(&mut self) -> (Option<F>, Option<F>) {
        let (next_ap, next_fp);
        // The following branches don't include the assertions. That is done in the verification.
        if self.instr().opcode() == OPC_CALL {
            // "call" instruction
            self.mem.write(self.curr.ap, self.curr.fp); // Save current fp
            self.mem
                .write(self.curr.ap + F::one(), self.curr.pc + self.vars.size); // Save next instruction
                                                                                // Update fp
            next_fp = Some(self.curr.ap + F::from(2u32)); // pointer for next frame is after current fp and instruction after call
                                                          // Update ap
            if self.instr().ap_up() == AP_Z2 {
                next_ap = Some(self.curr.ap + F::from(2u32)); // two words were written so advance 2 positions
            } else {
                panic!("Invalid instruction"); // ap increments not allowed in call instructions
            }
        } else if self.instr().opcode() == OPC_JMP_INC
            || self.instr().opcode() == OPC_RET
            || self.instr().opcode() == OPC_AEQ
        {
            // rest of types of instruction
            // jumps and increments || return || assert equal
            if self.instr().ap_up() == AP_Z2 {
                next_ap = Some(self.curr.ap) // no modification on ap
            } else if self.instr().ap_up() == AP_ADD {
                next_ap = Some(self.curr.ap + self.vars.res.unwrap()); // ap += <op> // should be larger than current
            } else if self.instr().ap_up() == AP_ONE {
                next_ap = Some(self.curr.ap + F::one()); // ap++
            } else {
                panic!("Invalid instruction");
            }
            if self.instr().opcode() == OPC_JMP_INC {
                next_fp = Some(self.curr.fp); // no modification on fp
            } else if self.instr().opcode() == OPC_RET {
                next_fp = Some(self.vars.dst.unwrap()); // ret sets fp to previous fp that was in [ap-2]
            } else if self.instr().opcode() == OPC_AEQ {
                // The following conditional is a fix that is not explained in the whitepaper
                // The goal is to distinguish two types of ASSERT_EQUAL where one checks that
                // dst = res , but in order for this to be true, one sometimes needs to write
                // the res in mem(dst_addr) and sometimes write dst in mem(res_dir). The only
                // case where res can be None is when res = op1 and thus res_dir = op1_addr
                if self.vars.res.is_none() {
                    self.mem.write(self.vars.op1_addr, self.vars.dst.unwrap()); // res = dst
                } else {
                    self.mem.write(self.vars.dst_addr, self.vars.res.unwrap()); // dst = res
                }
                next_fp = Some(self.curr.fp); // no modification on fp
            } else {
                panic!("Invalid instruction");
            }
        } else {
            panic!("Invalid instruction");
        }
        (next_ap, next_fp)
    }
}

/// This struct stores the needed information to run a program
pub struct CairoProgram<'a, F: FftField> {
    /// total number of steps
    steps: F,
    /// full execution memory
    mem: &'a mut CairoMemory<F>,
    /// initial computation registers
    ini: CairoPointers<F>,
    /// final computation pointers
    fin: CairoPointers<F>,
}

impl<'a, F: FftField> CairoProgram<'a, F> {
    /// Creates a Cairo execution from the public information (memory and initial pointers)
    pub fn new(mem: &mut CairoMemory<F>, pc: u64, ap: u64) -> CairoProgram<F> {
        let mut prog = CairoProgram {
            steps: F::zero(),
            mem,
            ini: CairoPointers::new(F::from(pc), F::from(ap), F::from(ap)),
            fin: CairoPointers::new(F::zero(), F::zero(), F::zero()),
        };
        prog.execute();
        prog
    }

    /// Outputs the total number of steps of the execution carried out by the runner
    pub fn get_steps(&self) -> F {
        self.steps
    }

    /// Outputs the final value of the pointers after the execution carried out by the runner
    pub fn get_final(&self) -> CairoPointers<F> {
        self.fin
    }

    /// This function simulates an execution of the Cairo program received as input.
    /// It generates the full memory stack and the execution trace
    fn execute(&mut self) {
        // set finishing flag to false, as it just started
        let mut end = false;
        // saves local copy of the initial (claimed) pointers of the program
        let mut curr = self.ini;
        let mut next = self.ini;
        // first timestamp
        let mut n: u64 = 0;
        // keep executing steps until the end is reached
        while !end {
            // create current step of computation
            let mut step = CairoStep::new(self.mem, next);
            // save current value of the pointers
            curr = step.curr;
            // execute current step and increase time counter
            step.execute();
            n += 1;
            match step.next {
                None => end = true, // if find no next pointers, end
                _ => {
                    // if there are next pointers
                    end = false;
                    // update next value of pointers
                    next = step.next.unwrap();
                    if curr.ap <= next.pc {
                        // if reading from unallocated memory, end
                        end = true;
                    }
                }
            }
        }
        self.steps = F::from(n);
        self.fin = CairoPointers::new(curr.pc, curr.ap, curr.fp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helper::CairoFieldHelpers;
    use mina_curves::pasta::fp::Fp as F;

    #[test]
    fn test_cairo_step() {
        // This tests that CairoStep works for a 2 word instruction
        //    tempvar x = 10;
        let instrs = vec![
            F::from(0x480680017fff8000u64),
            F::from(10u64),
            F::from(0x208b7fff7fff7ffeu64),
        ];
        let mut mem = CairoMemory::new(instrs);
        // Need to know how to find out
        // Is it final ap and/or final fp? Will write to starkware guys to learn about this
        mem.write(F::from(4u32), F::from(7u32));
        mem.write(F::from(5u32), F::from(7u32));
        let ptrs = CairoPointers::new(F::from(1u32), F::from(6u32), F::from(6u32));
        let mut step = CairoStep::new(&mut mem, ptrs);

        step.execute();
        assert_eq!(step.next.unwrap().pc, F::from(3u32));
        assert_eq!(step.next.unwrap().ap, F::from(7u32));
        assert_eq!(step.next.unwrap().fp, F::from(6u32));

        println!("{}", step.mem);
    }

    #[test]
    fn test_cairo_program() {
        let instrs = vec![
            F::from(0x480680017fff8000u64),
            F::from(10u64),
            F::from(0x208b7fff7fff7ffeu64),
        ];
        let mut mem = CairoMemory::<F>::new(instrs);
        // Need to know how to find out
        // Is it final ap and/or final fp? Will write to starkware guys to learn about this
        mem.write(F::from(4u32), F::from(7u32)); //beginning of output
        mem.write(F::from(5u32), F::from(7u32)); //end of output
        let prog = CairoProgram::new(&mut mem, 1, 6);
        println!("{}", prog.mem);
    }

    #[test]
    fn test_cairo_output() {
        // This is a test for a longer program, involving builtins, imports and outputs
        /*
        %builtins output
        from starkware.cairo.common.serialize import serialize_word
        func main{output_ptr : felt*}():
            tempvar x = 10
            tempvar y = x + x
            tempvar z = y * y + x
            serialize_word(x)
            serialize_word(y)
            serialize_word(z)
            return ()
        end
        */
        let instrs: Vec<i128> = vec![
            0x400380007ffc7ffd,
            0x482680017ffc8000,
            1,
            0x208b7fff7fff7ffe,
            0x480680017fff8000,
            10,
            0x48307fff7fff8000,
            0x48507fff7fff8000,
            0x48307ffd7fff8000,
            0x480a7ffd7fff8000,
            0x48127ffb7fff8000,
            0x1104800180018000,
            -11,
            0x48127ff87fff8000,
            0x1104800180018000,
            -14,
            0x48127ff67fff8000,
            0x1104800180018000,
            -17,
            0x208b7fff7fff7ffe,
            /*41, // beginning of outputs
            44,   // end of outputs
            44,   // input
            */
        ];

        let mut mem = CairoMemory::<F>::new(F::vec_to_field(instrs));
        // Need to know how to find out
        mem.write(F::from(21u32), F::from(41u32)); // beginning of outputs
        mem.write(F::from(22u32), F::from(44u32)); // end of outputs
        mem.write(F::from(23u32), F::from(44u32)); //end of program
        let prog = CairoProgram::new(&mut mem, 5, 24);
        assert_eq!(prog.get_final().pc, F::from(20u32));
        assert_eq!(prog.get_final().ap, F::from(41u32));
        assert_eq!(prog.get_final().fp, F::from(24u32));
        println!("{}", prog.mem);
        assert_eq!(prog.mem.read(F::from(24u32)).unwrap(), F::from(10u32));
        assert_eq!(prog.mem.read(F::from(25u32)).unwrap(), F::from(20u32));
        assert_eq!(prog.mem.read(F::from(26u32)).unwrap(), F::from(400u32));
        assert_eq!(prog.mem.read(F::from(27u32)).unwrap(), F::from(410u32));
        assert_eq!(prog.mem.read(F::from(28u32)).unwrap(), F::from(41u32));
        assert_eq!(prog.mem.read(F::from(29u32)).unwrap(), F::from(10u32));
        assert_eq!(prog.mem.read(F::from(30u32)).unwrap(), F::from(24u32));
        assert_eq!(prog.mem.read(F::from(31u32)).unwrap(), F::from(14u32));
        assert_eq!(prog.mem.read(F::from(32u32)).unwrap(), F::from(42u32));
        assert_eq!(prog.mem.read(F::from(33u32)).unwrap(), F::from(20u32));
        assert_eq!(prog.mem.read(F::from(34u32)).unwrap(), F::from(24u32));
        assert_eq!(prog.mem.read(F::from(35u32)).unwrap(), F::from(17u32));
        assert_eq!(prog.mem.read(F::from(36u32)).unwrap(), F::from(43u32));
        assert_eq!(prog.mem.read(F::from(37u32)).unwrap(), F::from(410u32));
        assert_eq!(prog.mem.read(F::from(38u32)).unwrap(), F::from(24u32));
        assert_eq!(prog.mem.read(F::from(39u32)).unwrap(), F::from(20u32));
        assert_eq!(prog.mem.read(F::from(40u32)).unwrap(), F::from(44u32));
        assert_eq!(prog.mem.read(F::from(41u32)).unwrap(), F::from(10u32));
        assert_eq!(prog.mem.read(F::from(42u32)).unwrap(), F::from(20u32));
        assert_eq!(prog.mem.read(F::from(43u32)).unwrap(), F::from(410u32));
    }
}
