use std::fs::read_to_string;
use std::path::Path;

use translator::Translator;

#[derive(Debug, PartialEq)]
pub enum MemorySegment {
    Local,
    Argument,
    This,
    That,
    Constant,
    Static,
    Pointer,
    Temp,
}

impl MemorySegment {
    pub fn seg_ptr(&self) -> &str {
        match *self {
            MemorySegment::Local => "LCL",
            MemorySegment::Argument => "ARG",
            MemorySegment::This => "THIS",
            MemorySegment::That => "THAT",
            _ => panic!("No segment pointer for {:?}", self),
        }
    }
}

mod parser {
    // Takes a VM instruction and parses it into the type of instruction it is
    // as well as its individual components if necessary
    use super::MemorySegment;

    #[derive(Debug, PartialEq)]
    pub enum ParsedVMInstruction {
        Add,
        Sub,
        Neg,
        Eq,
        Gt,
        Lt,
        And,
        Or,
        Not,
        Pop { segment: MemorySegment, idx: u16 },
        Push { segment: MemorySegment, idx: u16 },
        Label { label: String },
        Goto { label: String },
        IfGoto { label: String },
        Function { name: String, num_local_vars: u16 },
        Call { name: String, num_args: u16 },
        Return,
    }

    pub fn parse_instruction(instruction: &str) -> ParsedVMInstruction {
        let split_instr: Vec<&str> = instruction.split(" ").collect();
        match split_instr[0] {
            "add" => ParsedVMInstruction::Add,
            "sub" => ParsedVMInstruction::Sub,
            "neg" => ParsedVMInstruction::Neg,
            "eq" => ParsedVMInstruction::Eq,
            "gt" => ParsedVMInstruction::Gt,
            "lt" => ParsedVMInstruction::Lt,
            "and" => ParsedVMInstruction::And,
            "or" => ParsedVMInstruction::Or,
            "not" => ParsedVMInstruction::Not,
            "pop" => match split_instr[1] {
                "local" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::Local,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "argument" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::Argument,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "this" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::This,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "that" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::That,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "static" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::Static,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "pointer" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::Pointer,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "temp" => ParsedVMInstruction::Pop {
                    segment: MemorySegment::Temp,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                _ => panic!("Invalid pop memory segment: {}", split_instr[1]),
            },
            "push" => match split_instr[1] {
                "local" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Local,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "argument" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Argument,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "this" => ParsedVMInstruction::Push {
                    segment: MemorySegment::This,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "that" => ParsedVMInstruction::Push {
                    segment: MemorySegment::That,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "constant" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Constant,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "static" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Static,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "pointer" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Pointer,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                "temp" => ParsedVMInstruction::Push {
                    segment: MemorySegment::Temp,
                    idx: split_instr[2].parse::<u16>().unwrap(),
                },
                _ => panic!("Invalid push memory segment: {}", split_instr[1]),
            },
            "label" => ParsedVMInstruction::Label {
                label: split_instr[1].to_owned(),
            },
            "goto" => ParsedVMInstruction::Goto {
                label: split_instr[1].to_owned(),
            },
            "if-goto" => ParsedVMInstruction::IfGoto {
                label: split_instr[1].to_owned(),
            },
            "function" => ParsedVMInstruction::Function {
                name: split_instr[1].to_owned(),
                num_local_vars: split_instr[2].parse::<u16>().unwrap(),
            },
            "call" => ParsedVMInstruction::Call {
                name: split_instr[1].to_owned(),
                num_args: split_instr[2].parse::<u16>().unwrap(),
            },
            "return" => ParsedVMInstruction::Return,
            _ => panic!("Invalid instruction type: {}", split_instr[0]),
        }
    }
}

mod translator {
    // Given a parsed VM instruction, translates the instruction into its
    // valid Hack assembly code
    use super::parser::ParsedVMInstruction;
    use super::MemorySegment;

    const ADD: &'static [&str] = &["@SP", "AM=M-1", "D=M", "A=A-1", "M=M+D"];
    const SUBTRACT: &'static [&str] = &["@SP", "AM=M-1", "D=M", "A=A-1", "M=M-D"];
    const NEG: &'static [&str] = &["@SP", "A=M-1", "M=-M"];
    const AND: &'static [&str] = &["@SP", "AM=M-1", "D=M", "A=A-1", "M=D&M"];
    const OR: &'static [&str] = &["@SP", "AM=M-1", "D=M", "A=A-1", "M=D|M"];
    const NOT: &'static [&str] = &["@SP", "A=M-1", "M=!M"];
    const RETURN: &'static [&str] = &[
        "@LCL", "D=M", "@7", "M=D", "@5", "D=A", "@7", "A=M-D", "D=M", "@8", "M=D", "@SP", "A=M-1",
        "D=M", "@ARG", "A=M", "M=D", "@ARG", "D=M+1", "@SP", "M=D", "@7", "AM=M-1", "D=M", "@THAT",
        "M=D", "@7", "AM=M-1", "D=M", "@THIS", "M=D", "@7", "AM=M-1", "D=M", "@ARG", "M=D", "@7",
        "AM=M-1", "D=M", "@LCL", "M=D", "@8", "A=M", "0;JMP",
    ];

    const TEMP_OFFSET: u16 = 5;

    pub struct Translator {
        pub static_base: String,
        pub asm: Vec<String>,
        next_instr: u16,
        call_counter: u16,
    }

    impl Translator {
        pub fn new(static_base: String) -> Self {
            Self {
                next_instr: 0,
                call_counter: 0,
                static_base: static_base,
                asm: vec![],
            }
        }

        fn add_instr(&mut self, instr: &str) {
            self.asm.push(instr.to_owned());
            if instr.chars().next().unwrap() != '(' {
                self.next_instr += 1;
            }
        }

        fn const_instr_to_vec(&mut self, const_instr: &'static [&str]) {
            for &instr in const_instr {
                self.add_instr(instr)
            }
        }

        pub fn translate(&mut self, instruction: &ParsedVMInstruction) {
            match instruction {
                ParsedVMInstruction::Add => self.const_instr_to_vec(ADD),
                ParsedVMInstruction::Sub => self.const_instr_to_vec(SUBTRACT),
                ParsedVMInstruction::Neg => self.const_instr_to_vec(NEG),
                ParsedVMInstruction::Eq => self.logical_comp("JEQ"),
                ParsedVMInstruction::Gt => self.logical_comp("JGT"),
                ParsedVMInstruction::Lt => self.logical_comp("JLT"),
                ParsedVMInstruction::And => self.const_instr_to_vec(AND),
                ParsedVMInstruction::Or => self.const_instr_to_vec(OR),
                ParsedVMInstruction::Not => self.const_instr_to_vec(NOT),
                ParsedVMInstruction::Pop { segment, idx } => match segment {
                    MemorySegment::Local => self.basic_pop(segment, idx),
                    MemorySegment::Argument => self.basic_pop(segment, idx),
                    MemorySegment::This => self.basic_pop(segment, idx),
                    MemorySegment::That => self.basic_pop(segment, idx),
                    MemorySegment::Constant => panic!("Invalid instruction: pop constant"),
                    MemorySegment::Static => self.pop_static(idx),
                    MemorySegment::Pointer => self.pop_ptr(idx),
                    MemorySegment::Temp => self.pop_temp(idx),
                },
                ParsedVMInstruction::Push { segment, idx } => match segment {
                    MemorySegment::Local => self.basic_push(segment, idx),
                    MemorySegment::Argument => self.basic_push(segment, idx),
                    MemorySegment::This => self.basic_push(segment, idx),
                    MemorySegment::That => self.basic_push(segment, idx),
                    MemorySegment::Constant => self.push_const(idx),
                    MemorySegment::Static => self.push_static(idx),
                    MemorySegment::Pointer => self.push_ptr(idx),
                    MemorySegment::Temp => self.push_temp(idx),
                },
                ParsedVMInstruction::Label { label } => self.label_fn(&label),
                ParsedVMInstruction::Goto { label } => self.goto(&label),
                ParsedVMInstruction::IfGoto { label } => self.if_goto(&label),
                ParsedVMInstruction::Function {
                    name,
                    num_local_vars,
                } => self.function(&name, *num_local_vars),
                ParsedVMInstruction::Call { name, num_args } => self.call(&name, *num_args),
                ParsedVMInstruction::Return => self.const_instr_to_vec(RETURN),
            }
        }

        fn logical_comp(&mut self, jmp_instr: &str) {
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=M");
            self.add_instr("A=A-1");
            self.add_instr("D=M-D");
            self.add_instr("M=-1");
            // next_instr + 5 is how many instructions until the end of the current asm block
            self.add_instr(&format!("@{}", self.next_instr + 5));
            self.add_instr(&format!("D;{}", jmp_instr));
            self.add_instr("@SP");
            self.add_instr("A=M-1");
            self.add_instr("M=0");
        }

        fn basic_pop(&mut self, segment: &MemorySegment, idx: &u16) {
            let seg_ptr = segment.seg_ptr();
            self.add_instr(&format!("@{idx}"));
            self.add_instr("D=A");
            self.add_instr(&format!("@{seg_ptr}"));
            self.add_instr("D=D+M");
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=D+M");
            self.add_instr("A=D-M");
            self.add_instr("M=D-A");
        }
        fn pop_temp(&mut self, idx: &u16) {
            let mem_addr = TEMP_OFFSET + idx;
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=M");
            self.add_instr(&format!("@{mem_addr}"));
            self.add_instr("M=D");
        }

        fn pop_ptr(&mut self, idx: &u16) {
            let seg_ptr = match idx {
                0 => MemorySegment::This.seg_ptr(),
                1 => MemorySegment::That.seg_ptr(),
                _ => panic!("pop pointer instruction must have index 0 or 1"),
            };
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=M");
            self.add_instr(&format!("@{seg_ptr}"));
            self.add_instr("M=D");
        }

        fn pop_static(&mut self, idx: &u16) {
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=M");
            self.add_instr(&format!("@{}.{}", self.static_base, idx));
            self.add_instr("M=D");
        }

        fn push_const(&mut self, idx: &u16) {
            self.add_instr(&format!("@{idx}"));
            self.add_instr("D=A");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
        }

        fn basic_push(&mut self, segment: &MemorySegment, idx: &u16) {
            let seg_ptr = segment.seg_ptr();
            self.add_instr(&format!("@{idx}"));
            self.add_instr("D=A");
            self.add_instr(&format!("@{seg_ptr}"));
            self.add_instr("A=D+M");
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
        }

        fn push_temp(&mut self, idx: &u16) {
            let mem_addr = TEMP_OFFSET + idx;
            self.add_instr(&format!("@{mem_addr}"));
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
        }

        fn push_ptr(&mut self, idx: &u16) {
            let seg_ptr = match idx {
                0 => MemorySegment::This.seg_ptr(),
                1 => MemorySegment::That.seg_ptr(),
                _ => panic!("push pointer instruction must have index 0 or 1"),
            };
            self.add_instr(&format!("@{seg_ptr}"));
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
        }

        fn push_static(&mut self, idx: &u16) {
            self.add_instr(&format!("@{}.{}", self.static_base, idx));
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
        }

        fn label_fn(&mut self, label: &str) {
            self.add_instr(&format!("({label})"));
        }

        fn goto(&mut self, label: &str) {
            self.add_instr(&format!("@{label}"));
            self.add_instr("0;JMP");
        }

        fn if_goto(&mut self, label: &str) {
            self.add_instr("@SP");
            self.add_instr("AM=M-1");
            self.add_instr("D=M");
            self.add_instr(&format!("@{label}"));
            self.add_instr("D;JNE");
        }

        fn function(&mut self, name: &str, num_local_vars: u16) {
            self.add_instr(&format!("({name})"));
            for _ in 0..num_local_vars {
                self.add_instr("@SP");
                self.add_instr("M=M+1");
                self.add_instr("A=M-1");
                self.add_instr("M=0");
            }
        }

        fn call(&mut self, name: &str, num_args: u16) {
            let return_addr_label = format!("{}$ret.{}", name, self.call_counter);
            let arg_offset = 5 + num_args;
            self.add_instr(&format!("@{return_addr_label}"));
            self.add_instr("D=A");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
            self.add_instr("@LCL");
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
            self.add_instr("@ARG");
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
            self.add_instr("@THIS");
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
            self.add_instr("@THAT");
            self.add_instr("D=M");
            self.add_instr("@SP");
            self.add_instr("M=M+1");
            self.add_instr("A=M-1");
            self.add_instr("M=D");
            self.add_instr(&format!("@{arg_offset}"));
            self.add_instr("D=A");
            self.add_instr("@SP");
            self.add_instr("D=M-D");
            self.add_instr("@ARG");
            self.add_instr("M=D");
            self.add_instr("@SP");
            self.add_instr("D=M");
            self.add_instr("@LCL");
            self.add_instr("M=D");
            self.add_instr(&format!("@{name}"));
            self.add_instr("0;JMP");
            self.add_instr(&format!("({return_addr_label})"));
            self.call_counter += 1;
        }

        pub fn set_bootstrap(&mut self) {
            self.add_instr("@256");
            self.add_instr("D=A");
            self.add_instr("@SP");
            self.add_instr("M=D");
            self.call("Sys.init", 0);
        }
    }
}

fn read_lines(infile: &Path) -> Vec<String> {
    // Reads the lines of the infile, while ignoring comments and whitespace.
    read_to_string(infile)
        .unwrap()
        .lines()
        .filter_map(|line| strip_comment_and_whitespace(line))
        .collect()
}

fn strip_comment_and_whitespace(line: &str) -> Option<String> {
    let line = line.split("//").next().unwrap().trim();
    if line.is_empty() {
        return None;
    } else {
        return Some(line.to_owned());
    }
}

fn get_static_base(file: &Path) -> String {
    let static_base = file.file_stem().unwrap().to_str().unwrap();
    static_base.to_owned()
}

pub fn translate_file(infile: &Path) -> Vec<String> {
    let static_base = get_static_base(infile);
    let mut translator = Translator::new(static_base);
    let lines = read_lines(infile);
    for line in lines {
        let instruction = parser::parse_instruction(&line);
        translator.translate(&instruction);
    }
    translator.asm
}

pub fn translate_directory(directory: &Path) -> Vec<String> {
    let mut vm_files = vec![];
    for entry in directory.read_dir().unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().unwrap() == "vm" {
                vm_files.push(path);
            }
        }
    }
    let mut translator = Translator::new(String::from(""));
    translator.set_bootstrap();
    for file in vm_files {
        let static_base = get_static_base(&file);
        translator.static_base = static_base;
        let lines = read_lines(&file);
        for line in lines {
            let instruction = parser::parse_instruction(&line);
            translator.translate(&instruction);
        }
    }
    translator.asm
}

#[cfg(test)]
mod tests {
    use super::parser::{parse_instruction, ParsedVMInstruction};
    use super::MemorySegment;

    #[test]
    fn test_parse_valid_instruction() {
        let test_cases = vec![
            (
                "push constant 10",
                ParsedVMInstruction::Push {
                    segment: MemorySegment::Constant,
                    idx: 10,
                },
            ),
            (
                "pop argument 2",
                ParsedVMInstruction::Pop {
                    segment: MemorySegment::Argument,
                    idx: 2,
                },
            ),
            ("add", ParsedVMInstruction::Add),
            ("sub", ParsedVMInstruction::Sub),
        ];

        for test in test_cases {
            let parsed_instruction = parse_instruction(test.0);
            assert_eq!(parsed_instruction, test.1);
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_instruction() {
        let _parsed_instruction = parse_instruction("gte");
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_push_instruction() {
        let _parsed_instruction = parse_instruction("push constant");
    }
}
