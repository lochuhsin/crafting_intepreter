use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::constants;
use crate::errors::runtime_error;
use crate::values::GenericValue;
use crate::values::GenericValueType;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RuntimeError {
    UnsupportedOperation(String, String),
    InvalidOperation(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UnsupportedOperation(type1, type2) => {
                write!(f, "Operation not supported for {} and {}", type1, type2)
            }
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRunTimeError,
}

#[derive(Default)]
pub struct VirtualMachine {
    pub chunk: Chunk,
    pub ip: usize, // instruction pointer, the index currently pointing to the instruction in chunk
    pub vm_stack: VirtualMachineStack,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> VirtualMachine {
        VirtualMachine {
            ip: 0,
            chunk,
            vm_stack: VirtualMachineStack::default(),
        }
    }
    pub fn update_chunk(&mut self, chunk: Chunk) {
        self.chunk = chunk;
    }
}

pub fn run(vm: &mut VirtualMachine) -> InterpretResult {
    loop {
        #[cfg(debug_assertions)]
        {
            for i in 0..vm.vm_stack.top {
                print!("[ {} ]", vm.vm_stack.values[i])
            }
            println!();
            disassemble_instruction(&vm.chunk, vm.ip);
        }
        let op_code = read_op(vm);
        match op_code {
            OpCode::OpReturn => {
                println!("{}", vm.vm_stack.pop());
                return InterpretResult::InterpretOk;
            }
            OpCode::OpConstant => {
                let val = read_constant(vm);
                vm.vm_stack.push(val);
            }
            OpCode::OpNegate => {
                vm.vm_stack.negate_peek();
            }
            OpCode::OpAdd => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop(); // Handle empty value stack

                let v = v1 + v2;
                match v {
                    // TODO: put the actual line, not 0
                    Ok(v) => vm.vm_stack.push(v),
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                }
            }
            OpCode::OpSubtract => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop(); // Handle empty value stack
                let v = v1 - v2;
                match v {
                    Ok(v) => vm.vm_stack.push(v),
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                }
            }
            OpCode::OpMultiply => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop(); // Handle empty value stack
                let v = v1 * v2;
                match v {
                    // TODO: put the actual line, not 0
                    Ok(v) => vm.vm_stack.push(v),
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                }
            }
            OpCode::OpDivide => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop(); // Handle empty value stack
                let v = v1 / v2;
                match v {
                    // TODO: put the actual line, not 0
                    Ok(v) => vm.vm_stack.push(v),
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                }
            }
            OpCode::OpNil => vm.vm_stack.push(GenericValue::from_none()),
            OpCode::OpFalse => vm.vm_stack.push(GenericValue::from_bool(true)),
            OpCode::OpTrue => vm.vm_stack.push(GenericValue::from_bool(false)),
            OpCode::OpNot => {
                let val = vm.vm_stack.pop();

                // TODO: move this to value, operator overloading (trait ~~~)
                fn is_false(v: &GenericValue) -> Result<bool, RuntimeError> {
                    match v {
                        GenericValueType::Nil => Ok(true),
                        GenericValueType::Bool(b) => Ok(!b),
                        _ => Err(RuntimeError::InvalidOperation("unary only support boolean and None, should the error be implemented in this phase ?".to_string())),
                    }
                }
                match is_false(&val) {
                    Ok(v) => vm.vm_stack.push(GenericValue::from_bool(v)),
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                }
            }
            OpCode::OpEqual => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop();

                // TODO: move this to value, operator overloading (trait ~~~)
                fn is_equal(v1: &GenericValue, v2: &GenericValue) -> bool {
                    match (v1, v2) {
                        (GenericValueType::Nil, GenericValueType::Nil) => true,
                        (GenericValueType::Bool(b1), GenericValueType::Bool(b2)) => (*b1) == (*b2),
                        (GenericValueType::Number(n1), GenericValueType::Number(n2)) => n1 == n2,
                        _ => false,
                    }
                }

                vm.vm_stack.push(GenericValueType::Bool(is_equal(&v1, &v2)))
            }
            OpCode::OpGreater => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop();

                // TODO: move this to value, operator overloading (trait ~~~)
                fn is_greater(v1: GenericValue, v2: GenericValue) -> Result<bool, RuntimeError> {
                    match (v1, v2) {
                        (GenericValueType::Number(n1), GenericValueType::Number(n2)) => Ok(n1 > n2),
                        _ => Err(RuntimeError::InvalidOperation(
                            " > not supported ".to_string(),
                        )),
                    }
                }
                match is_greater(v1, v2) {
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                    Ok(v) => vm.vm_stack.push(GenericValueType::Bool(v)),
                }
            }
            OpCode::OpLess => {
                let v1 = vm.vm_stack.pop();
                let v2 = vm.vm_stack.pop();

                // TODO: move this to value, operator overloading (trait ~~~)
                fn is_less(v1: GenericValue, v2: GenericValue) -> Result<bool, RuntimeError> {
                    match (v1, v2) {
                        (GenericValueType::Number(n1), GenericValueType::Number(n2)) => Ok(n1 > n2),
                        _ => Err(RuntimeError::InvalidOperation(
                            " < not supported ".to_string(),
                        )),
                    }
                }
                match is_less(v1, v2) {
                    Err(e) => runtime_error(0, e.to_string().as_str()),
                    Ok(v) => vm.vm_stack.push(GenericValueType::Bool(v)),
                }
            }
        };
    }
}

fn read_op_raw(vm: &mut VirtualMachine) -> usize {
    let code = vm.chunk.bytecode[vm.ip];
    vm.ip += 1;
    code
}

fn read_op(vm: &mut VirtualMachine) -> OpCode {
    let code = vm.chunk.bytecode[vm.ip];
    vm.ip += 1;
    OpCode::from_usize(code)
}

fn read_constant(vm: &mut VirtualMachine) -> GenericValue {
    let code = read_op_raw(vm);
    vm.chunk.const_pool.values[code]
}

////////////////////////////////////////////////////////////////
pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.count {
        offset = disassemble_instruction(chunk, offset)
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04}   ", offset);

    let instruction = OpCode::from_usize(chunk.bytecode[offset]);

    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!(" |     ")
    } else {
        print!("{:04}   ", chunk.lines[offset])
    }

    match instruction {
        OpCode::OpReturn => simple_instruction(instruction, offset),
        OpCode::OpConstant => constant_instruction(instruction, offset, chunk),
        OpCode::OpNegate => simple_instruction(instruction, offset),
        OpCode::OpAdd => simple_instruction(instruction, offset),
        OpCode::OpSubtract => simple_instruction(instruction, offset),
        OpCode::OpMultiply => simple_instruction(instruction, offset),
        OpCode::OpDivide => simple_instruction(instruction, offset),
        OpCode::OpNil => simple_instruction(instruction, offset),
        OpCode::OpFalse => simple_instruction(instruction, offset),
        OpCode::OpTrue => simple_instruction(instruction, offset),
        OpCode::OpNot => simple_instruction(instruction, offset),
        OpCode::OpEqual => simple_instruction(instruction, offset),
        OpCode::OpGreater => simple_instruction(instruction, offset),
        OpCode::OpLess => simple_instruction(instruction, offset),
    }
}

pub fn simple_instruction(op: OpCode, offset: usize) -> usize {
    println!("{}", op);
    offset + 1
}

pub fn constant_instruction(op: OpCode, offset: usize, chunk: &Chunk) -> usize {
    let constant = chunk.bytecode[offset + 1];
    let val = chunk.const_pool.values[constant];

    println!("{}{}'{}'", op, " ".repeat(15), val);
    offset + 2
}

pub struct VirtualMachineStack {
    pub values: [GenericValue; constants::STACK_MAX as usize],
    pub top: usize,
}

impl VirtualMachineStack {
    pub fn push(&mut self, value: GenericValue) {
        if self.top >= self.values.len() {
            panic!("Invalid operation, exceeds stack limit")
        }
        self.values[self.top] = value;
        self.top += 1;
    }

    pub fn pop(&mut self) -> GenericValue {
        if self.top == 0 {
            panic!("Invalid operation, empty stack ")
        }
        self.top -= 1;
        self.values[self.top]
    }

    pub fn peek(&mut self, distance: usize) -> GenericValue {
        /*
           peek value, start from the top of the stack,
           zero means the top value
        */
        if self.top == 0 {
            panic!("Invalid operation, empty stack ")
        }
        self.values[self.top - 1 - distance]
    }

    // Special optimization for OP_NEGATE
    pub fn negate_peek(&mut self) {
        if self.top == 0 {
            panic!("Invalid operation, empty stack ")
        }
        let v = -self.values[self.top - 1];
        match v {
            // TODO: put the actual line, not 0
            Ok(v) => self.values[self.top - 1] = v,
            Err(e) => runtime_error(0, e.to_string().as_str()),
        }
    }
}

impl Default for VirtualMachineStack {
    fn default() -> Self {
        VirtualMachineStack {
            values: [GenericValue::Nil; constants::STACK_MAX as usize], // Initialize as nil
            top: 0,
        }
    }
}
