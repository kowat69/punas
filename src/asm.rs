use nom::{AsBytes, IResult};
use nom::character::complete::{
    space1,
    multispace0,
    alphanumeric0,
    one_of,
    digit1,
};
use nom::multi::many0_count;
use nom::bytes::complete::{
    tag,
    take_until,
    take,
};
use nom::branch::alt;
use nom::character::{
    is_newline,
    is_space,
    is_alphabetic,
    is_alphanumeric,
};
use std::fs::File;
use std::io::prelude::*;
use std::io::{
    self,
    BufWriter,
    Write,
    BufReader,
    Read,
};
use std::cell::RefCell;
use std::{iter, mem};
mod reg;// load const registers
use reg as r;
use reg::Value;
mod headers;
use headers::*;
#[derive(Default)]
pub struct Label<'a>{
    name: & 'a str,
    pos: usize,
    section_number: usize
}
impl<'a> Label<'a>{
    pub fn new(_name: & 'a str, _pos: usize, _section_number: usize) -> Self{
        Self{name: _name, pos: _pos, section_number: _section_number}
    }
}
#[derive(Default)]
pub struct Section<'a>{
    name : & 'a str,
    data : Vec<u8>,
    number_of_relocations: usize,
}
impl<'a> Section<'a>{
    pub fn new(_name :& 'a str) ->Self{
        let mut ret = Self::default();
        ret.name = _name;
        ret
    }
    pub fn to_section_header(&self) -> SECTION_HEADER{
        let mut sh = SECTION_HEADER::default();
        sh.Name[..self.name.len()].copy_from_slice(self.name.as_bytes());
        sh.SizeOfRawData = self.data.len() as u32;
        sh.NumberOfRelocations = self.number_of_relocations as u16;
        match self.name {
            ".data" =>{
                sh.Characteristics = 0xC0300040;
            },
            ".text" =>{
                sh.Characteristics = 0x60500020;
            },
            ".bss" =>{
                sh.Characteristics = 0xC0300080;
            },
            _ => {panic!("Can't use this section name: {}.", self.name);}
        }
        sh
    }
}
#[derive(Default)]
pub struct Asm<'a>{
    m_file: & 'a str,
    m_contents: & 'a str,
    sections: RefCell<Vec<Section<'a>>>,
    labels: RefCell<Vec<Label<'a>>>,
}

impl<'a> Asm<'a>{
    pub fn new(file: & 'a str, contents: & 'a str) -> Self{
        let mut ret = Self::default();
        ret.m_file = file;
        ret.m_contents = contents;
        ret
    }
    pub fn debug(&self){
        let sections = self.sections.borrow();
        let mut sections = sections.iter();
        while let &Some(section) = &sections.next(){
            println!("#####{}#####", section.name);
            for op in &section.data{
                print!("{:0>2X} ", op);
            }
            println!();
        }
    }
    pub fn start(&self){
        let mut input;
        let mut lines = self.m_contents.lines();
        while let Some(s) = lines.next(){
            // 一行ずつ読み込んでいる
            input = s;
            println!("{}", s);
            loop {
                if input == ""{
                    break;
                }
                if let Ok((s, _)) = ignore_space(input){
                    input = s;
                    continue;
                }
                if is_ignore_comment(input){
                    break;
                }
                input = self.label_or_instruction(input);
                
            }
        }
    }
    pub fn write(&self, filename: & 'a str){
        let file = File::create(filename).unwrap();
        let mut writer = BufWriter::new(file);

        let sections = self.sections.borrow_mut();

        let data_sumsize = |x: &Vec<Section>| -> usize {
            let mut size = 0usize;
            for s in x{
                size += s.data.len();
            }
            size
        };

        let mut p_section: usize = mem::size_of::<FILE_HEADER>();
        let mut p_data: usize = p_section + sections.len() * mem::size_of::<SECTION_HEADER>();

        let mut file_headers = FILE_HEADER::new();
        let mut section_headers = Vec::<SECTION_HEADER>::new();
        let mut symbol_tables = Vec::<u8>::new();
        // FILE_HEADER
        file_headers.Machine = 0x8664;
        file_headers.NumberOfSections = sections.len() as u16; // set later
        file_headers.TimeDataStamp = chrono::Local::now().timestamp() as u32;
        file_headers.PointerToSymbolTable = 0; // set later
        file_headers.NumberOfSymbols = 0;// set later
        file_headers.SizeOfOptionalHeader = 0;
        file_headers.Characteristics = 0;
        // SECTION
        for i in 0..sections.len(){
            let sec = &sections[i];
            let mut section_header = sec.to_section_header();
            
            // set section
            section_header.PointerToRawData = p_data as u32;
            p_data += sec.data.len();
            section_header.PointerToRelocations = p_data as u32;
            p_data += sec.number_of_relocations * mem::size_of::<RELOCATION>();
            
            section_headers.push(section_header);
        }
        //relocations
        // symbol
        /*
        * file symbol
         */
        let mut _sbl = SYMBOL_TABLE::new_dot_file();
        symbol_tables.append(& mut as_u8_slice(&_sbl).to_vec());
        let mut _sbl = [0u8; 0x12];
        _sbl[..self.m_file.len()].copy_from_slice(self.m_file.as_bytes());
        symbol_tables.append(& mut as_u8_slice(&_sbl).to_vec());
        
        for i in 0..sections.len(){
            let sec = &sections[i];
            //set symbols
            let mut symbol = SYMBOL_TABLE::default();
            symbol.Name[..sec.name.len()].copy_from_slice(sec.name.as_bytes());
            symbol.SectionNumber = 1u16 + i as u16;
            symbol.StorageClass = 3;
            symbol.NumberOfAuxSymbols = 1;
            let mut symbol_define_section = SYMBOL_U8::default();
            symbol_define_section.set(&sec.data.len(), 0);
            symbol_define_section.set(&sec.number_of_relocations, 4);
            
            symbol_tables.append(&mut as_u8_slice(&symbol).to_vec());
            symbol_tables.append(&mut symbol_define_section.data.to_vec());
        }
        // * label
        for label in &*self.labels.borrow(){
            let mut symbol = SYMBOL_TABLE::default();
            symbol.Name[..label.name.len()].copy_from_slice(label.name.as_bytes());
            symbol.Value = label.pos as u32;
            symbol.SectionNumber = label.section_number as u16;
            symbol.StorageClass = 2;
            symbol_tables.append(&mut as_u8_slice(&symbol).to_vec());
        }
        symbol_tables.append(&mut as_u8_slice(&4u32).to_vec());
        // * set later
        file_headers.NumberOfSymbols = (symbol_tables.len() / 0x12) as u32;
        file_headers.PointerToSymbolTable = p_data as u32;
        // * Writing
        // file header
        let pfh = as_u8_slice(&file_headers);
        writer.write_all(pfh).expect("");
        // sections
        for sh_one in &section_headers{
            writer.write_all(as_u8_slice(sh_one)).expect("");
        }
        // data
        let mut seciter = sections.iter();
        while let Some(sec) = seciter.next(){
            writer.write_all(&sec.data).expect("");
        }
        // symbols
        writer.write_all(symbol_tables.as_bytes()).expect("");
        writer.flush().expect("");
    }

    fn label_or_instruction(&self, mut input: & 'a str) -> & 'a str{
// label or instruction
        let Ok((s, first_word)) = get_word(input) else{
            return input;
        };
        input = s;
        let c = get_str_first(input);
        // label
        if c == b':'{
            if let Ok((s, _)) = read_chars(input, 1){
                let mut labels = self.labels.borrow_mut();
                let sections = self.sections.borrow();
                let section = &sections[sections.len() - 1];
                labels.push(Label::new(first_word,
                    section.data.len(), sections.len()));
                input = s;
                return input;
            }else{
                let ae = AsmError::new(self.m_contents);
                ae.panic_from_word(input, "colon error");
            };
        }
        // instruction
        input = self.ignore_space(input);
        input = self._instruction(input, first_word);
        input
    }
    fn _instruction(&self, mut input: & 'a str, instruction: & 'a str) -> & 'a str{
        let first_word_lower = instruction.to_lowercase();
        let instruction_lower = first_word_lower.as_str();
        // dx or resx
        let c = get_str_first(instruction_lower);
        // Declaring Uninitialized or Initialized Data
        match c {
            //dx
            b'd' =>{
                if instruction_lower.len() == 2{
                    let c = instruction_lower.as_bytes()[1];
                    let size = self.dx_to_size(c);
                    input = self.dx(input, size);
                    return input;
                }
            },
            //resx
            b'r' =>{
                if instruction_lower.len() == 4 && &instruction_lower[0..3] == "res" {
                    let c = instruction_lower.as_bytes()[3];
                    let size = self.dx_to_size(c);
                    input = self.resx(input, size);
                    return input;
                }
            }
            _ => {}
        };
        match instruction_lower{
            "section" =>{
                input = self.section(input);
            },
            "mov" =>{
                input = self.mov(input);
            },
            "ret" =>{
                input = self.ret(input);
            },
            "add" =>{
                input = self.add(input);
            },
            "sub" => {
                input = self.sub(input);
            }
            _ =>{
                let ae = AsmError::new(self.m_contents);
                ae.panic_from_word(input, "Syntax Error.");
            }
        };
        input
    }
    fn dx_to_size(&self, c: u8) -> u8{
        match c{
            b'b'=>1,
            b'w'=>2,
            b'd'=>4,
            b'q'=>8,
            b't'=>16,
            b'o'=>32,
            b'y'=>64,
            b'z'=>128,
            _ => {
                0
            }
        }
    }
    fn resx(&self, mut input: & 'a str, size: u8 ) -> & 'a str{
        if input == ""{
            return input;
        }
        if let Ok((s, _)) = ignore_space(input){
            input = s;
        }
        if is_ignore_comment(input) {
            return input;
        }
        let c = get_str_first(input);
        if let Ok((s, fig)) = get_figure(input){
            input = s;
            if let Ok(fig) = fig.parse::<u64>(){
                let mut sections = self.sections.borrow_mut();
                let section = sections.last_mut().expect("");
                let mut data = iter::repeat(0u8)
                    .take(size as usize * fig as usize)
                    .collect::<Vec<u8>>();
                section.data.append(&mut data);
            }
        }else{
            let ae = AsmError::new(self.m_contents);
            let mes = format!("Require Figure.");
            ae.panic_from_word_idx(input, 0, mes.as_str());
        }
        input
    }
    fn dx(&self, mut input: & 'a str, size: u8) -> & 'a str{
        if input == ""{
            return input;
        }
        if let Ok((s, _)) = ignore_space(input){
            input = s;
        }
        if is_ignore_comment(input){
            return input;
        }
        let c = get_str_first(input);
        // string
        if c == b'\'' || c ==b'\"' {
            if let Ok((s, first)) = get_string(input){
                input = s;
                let len = first.len() % size as usize;

                let mut sections = self.sections.borrow_mut();
                let section = sections.last_mut().expect("");

                section.data.append(& mut first.as_bytes().to_vec());
                let mut zeros = vec![0u8; len];
                section.data.append(& mut zeros);

            }else {
                let ae = AsmError::new(self.m_contents);
                let mes = format!("Require \'or\".");
                ae.panic_from_word_idx(input, 0, mes.as_str());
            }
            // figure
        }else if let Ok((s, first)) = get_figure(input){
            input = s;
            if let Ok(figure) = first.parse::<u64>(){

                let mut sections = self.sections.borrow_mut();
                let section = sections.last_mut().expect("");

                section.data.append(& mut as_u8_slice_size(&figure, size as usize).to_vec());
            }else{}// not occur
        }else{
            let ae = AsmError::new(self.m_contents);
            let mes = format!("Require {}.", input);
            ae.panic_from_word_idx(input, 0, mes.as_str());
        }
        input = self.ignore_space(input);
        let c = get_str_first(input);
        if c == b','{
            input = self.read_comma(input);
            input = self.dx(input, size);
        }
        input
    }
    fn section(&self, mut input: & 'a str) -> & 'a str{
        input = self.ignore_space(input);
        if let Ok((s, section_name)) = get_word(input){
            let mut sections = self.sections.borrow_mut();
            sections.push(Section::new(section_name));
            s
        }else{
            let ae = AsmError::new(self.m_contents);
            ae.panic_from_word(input, "Syntax Error.");
            panic!();
        }
    }
    fn ret(&self, input: & 'a str)-> & 'a str {
        let mut sections = self.sections.borrow_mut();
        let section = sections.last_mut().expect("");
        section.data.push(0xC3);
        input
    }
    fn add(&self, mut input: & 'a str) -> & 'a str{
        let ae = AsmError::new(self.m_contents);
        let ((value1, value1str, value2, value2str), s) = self.read_2args(input);
        input = s;

        let mut data = Vec::<u8>::new();
        match value1{
            Value::Reg(reg1, size1) => {
                match value2{
                    Value::Figure(v2) =>{
                        if size1 == 8{
                            let rexb = (reg1 & 0b1000) >> 3 << r::REX_B;
                            let v2 = v2.parse::<u64>().unwrap();
                            let v2size = if v2 < 0x80 || v2 >= 0xffffffff_ffffff80 {1}
                                         else if v2 < 0x80000000 || v2 >= 0xffffffff_80000000 {4}
                                         else {8};
                            if v2size == 8{
                                ae.panic_from_word(value2str, "Expect signed 32bit");
                            }
                            let v2 = v2.to_le_bytes().to_vec();

                            let rexw: u8 = 0x48 | rexb;
                            data.push(rexw);
                            let reg = reg1 & 0b0111;
                            if v2size == 1{
                                let op: u8 = 0x83;
                                let modf = 0b11;
                                let regf = 0b000;
                                let rmf = reg1 & 0b0111;

                                let regrm = r::create_modrm(modf, regf, rmf);

                                data.append(&mut vec![op, regrm]);
                                data.push(v2[0]);
                            }else if reg == reg::RAX{
                                let op:u8 = 0x05;
                                data.push(op);
                                data.extend(&v2[0..v2size]);
                            }else{
                                let op = 0x81;
                                let modf = 0b11;
                                let regf = 0b000;
                                let rmf = reg1 & 0b0111;

                                let regrm = r::create_modrm(modf, regf, rmf);

                                data.extend([op, regrm]);
                                data.extend(&v2[0..v2size]);
                            }
                        }
                    },
                    Value::Reg(reg2, size2) =>{
                        // r/m reg1
                        let rexb: u8 = (reg1 & 0b1000) >> 3;
                        // reg reg2
                        let rexr: u8 = (reg2 & 0b1000) >> 3;
                        
                        if size1== 8 {
                            let rex = r::create_rex(1, rexr, 0, rexb);
                            data.push(rex);
                        }else if rexb == (0b1 << r::REX_B) || rexr == (0b1 << r::REX_R) {
                            let rex = r::create_rex(0, rexr, 0, rexb);
                            data.push(rex);
                        }
                        let op = 0x01;
                        let reg1 = reg1 & 0b0111;
                        let reg2 = reg2 & 0b0111;
                        let modrm = r::create_modrm(0b11, reg2, reg1);
                        data.push(op);
                        data.push( modrm);
                    },
                    _ =>{ae.panic_from_word(value2str, "Not")}
                }
            },
            _ =>{ae.panic_from_word(value1str, "Not")}
        }
        let mut sections = self.sections.borrow_mut();
        let section = sections.last_mut().expect("");
        // data
        section.data.append(&mut data);
        input
    }
    fn sub(&self, mut input: & 'a str) -> & 'a str{
        let ae = AsmError::new(self.m_contents);
        let ((value1, value1str, value2, value2str), s) = self.read_2args(input);
        input = s;
        let mut data = Vec::<u8>::new();
        match value1{
            Value::Reg(reg1, size1) => {
                match value2{
                    Value::Figure(v2) => {
                        if size1 == 8{
                            let rexb = (reg1 & 0b1000) >> 3 << r::REX_B;
                            let v2 = v2.parse::<u64>().unwrap();
                            let v2size = if v2 < 0x80 || v2 >= 0xffffffff_ffffff80 {1}
                                         else if v2 < 0x80000000 || v2 >= 0xffffffff_80000000 {4}
                                         else {8};
                            if v2size == 8{
                                ae.panic_from_word(value2str, "Expect signed 32bit");
                            }
                            let v2 = v2.to_le_bytes().to_vec();

                            let rexw: u8 = 0x48 | rexb;
                            data.push(rexw);
                            let reg = reg1 & 0b0111;
                            if v2size == 1{
                                let op: u8 = 0x83;
                                let modf = 0b11;
                                let regf = 0b101;// /5
                                let rmf = reg1 & 0b0111;

                                let regrm = r::create_modrm(modf, regf, rmf);

                                data.append(&mut vec![op, regrm]);
                                data.push(v2[0]);
                            }else if reg == reg::RAX{
                                let op:u8 = 0x2d;
                                data.push(op);
                                data.extend(&v2[0..v2size]);
                            }else{
                                let op = 0x81;
                                let modf = 0b11;
                                let regf = 0b101;// /5
                                let rmf = reg1 & 0b0111;

                                let regrm = r::create_modrm(modf, regf, rmf);

                                data.extend([op, regrm]);
                                data.extend(&v2[0..v2size]);
                            }
                        }
                    },
                    Value::Reg(reg2, size2) => {
                        // r/m reg1
                        let rexb: u8 = (reg1 & 0b1000) >> 3;
                        // reg reg2
                        let rexr: u8 = (reg2 & 0b1000) >> 3;
                        
                        if size1== 8 {
                            let rex = r::create_rex(1, rexr, 0, rexb);
                            data.push(rex);
                        }else if rexb == (0b1 << r::REX_B) || rexr == (0b1 << r::REX_R) {
                            let rex = r::create_rex(0, rexr, 0, rexb);
                            data.push(rex);
                        }
                        let op = 0x29;
                        let reg1 = reg1 & 0b0111;
                        let reg2 = reg2 & 0b0111;
                        let modrm = r::create_modrm(0b11, reg2, reg1);
                        data.push(op);
                        data.push( modrm);
                    },
                    _ => {ae.panic_from_word(value2str, "Not");}
                }
            },
            _ => {ae.panic_from_word(value1str, "Not");}
        }
        //########################################################################
        let mut sections = self.sections.borrow_mut();
        let section = sections.last_mut().expect("");
        // data
        section.data.append(&mut data);
        input
    }
    fn mov(&self, mut input: & 'a str) -> & 'a str{
        let ((value1, _, value2, _), s) = self.read_2args(input);
        input = s;
        let mut data = Vec::<u8>::new();
        match value1{
            Value::Reg(reg1, size) => {
                match value2{
                    Value::Figure(v2) =>{
                        let rexb = (reg1 & 0b1000) >> 3;
                        let v2 = v2.parse::<u64>().unwrap();
                        // b8 + rd id
                        let (op, v2size)= if v2 <= 0xffffffff {
                            (0xb8, 4)
                        // 48 c7 /0 id
                        }else if v2 >= 0xffffffff_00000000{
                            (0xc7, 4)
                        // rex.w b8 + rd io
                        }else{
                            (0xb8, 8)
                        };
                        let v2 = v2.to_le_bytes().to_vec();
                        if size == 8 && (op == 0xc7 || (op == 0xb8 && v2size == 8)) {
                            let rexw: u8 = 0x48 | rexb;
                            data.push(rexw);
                        }else if rexb == 1{
                            let rex:u8 = 0x40 | rexb;
                            data.push(rex);
                        }
                        if op == 0xb8{
                            let op = 0xB8 | (reg1 & 0b111);// opcode
                            data.push(op);

                            data.extend(&v2[0..v2size as usize]);
                        }else if op == 0xc7{
                            let op = 0xc7;
                            let modf = 0b11;
                            let regf = 0;
                            let rmf = reg1 & 0b111;
                            let modrm = modf << 6 | regf << 3 | rmf;
                            data.push(op);
                            data.push(modrm);
                            data.extend(&v2[0..v2size as usize]);
                        }else {panic!();}
                        
                    },
                    Value::Reg(reg2, size) =>{
                        //reg
                        let rexr = (reg2 & 0b1000) >> 3 << r::REX_R;
                        //rm
                        let rexb = (reg1 & 0b1000) >> 3 << r::REX_B;
                        if size == 8{
                            let rex = 0x48 | rexr | rexb;
                            data.push(rex);
                        }else if rexr == 1 << r::REX_R || rexb == 1 << r::REX_B{
                            let rex = 0x40 | rexr | rexb;
                            data.push(rex);
                        }
                        let op = 0x89;
                        let modf = 0b11;
                        let reg = reg2 & 0b111;
                        let rm = reg1 & 0b111;

                        let r = r::create_modrm(modf, reg, rm);

                        data.extend([op, r]);

                    },
                    _ => {panic!()}
                }
                
            },
            _ =>{}
        };
        //########################################################################
        let mut sections = self.sections.borrow_mut();
        let section = sections.last_mut().expect("");
        // data
        section.data.append(&mut data);
        input
    }
    
    fn read_2args(&self, mut input: & 'a str) -> ((Value, & 'a str, Value, & 'a str), & 'a str){
        let value1 :Value;
        let value2 :Value;
        //########################################################################
        let value1str = input;
        let (s, value) = self.read_value_unwrap(input, "mov: Expect Register or Memory");
        input = s;
        value1 = value;
        //########################################################################
        input = self.read_comma(input);
        //########################################################################
        input = self.ignore_space(input);
        let value2str = input;
        let (s, value) = self.read_value_unwrap(input, "mov: Expect Register, Memory,or Memory");
        input = s;
        value2 = value;
        //########################################################################

        ((value1, value1str, value2, value2str), input)
    }
    fn read_value(&self, mut input: & 'a str) -> Result<(&'a str, Value), &'a str>{
        let value;
        if let Ok((s, t)) = get_word(input){
            input = s;
            let reg = r::reg(t);
            if let Ok(reg) = reg{
                value = reg;
            }else{
                return Err(input);
            }
        }else if let Ok((s, t)) = get_figure(input){
            input = s;
            value = Value::Figure(t);
        }else {
            return Err(input);
        }
        Ok((input, value))
    }
    fn read_value_unwrap(&self, input: & 'a str, message: & str) -> (& 'a str, Value){
        if let Ok((s, value)) = self.read_value(input){
            return (s, value);
        }else{
            let ae = AsmError::new(self.m_contents);
            ae.panic_from_word(input, message);
            panic!();
        }
    }
    fn read_comma(&self, mut input: & 'a str)-> & 'a str{
        // ignore space
        input = self.ignore_space(input);
        // read comma
        let Ok((s, comma)) = get_others(input) else {
            let ae = AsmError::new(self.m_contents);
            ae.panic_from_word(input, "Require Comma.");
            panic!();
        };
        input = s;
        if comma == ","{}
        else{
            let ae = AsmError::new(self.m_contents);
            ae.panic_from_word(input, "Require \',\' .");
        }
        input
    }
    fn ignore_space(&self, input: & 'a str) -> &'a str{
        if let Ok((s, _)) = ignore_space(input){
            return s;
        }else{
            return input;
        }
    }
    
}
fn read_chars<'a>(input: & 'a str, cnt: usize) -> IResult<&str, &str>{
    take(cnt)(input)
}
fn get_figure<'a>(input: & 'a str)->IResult<&str, &str>{
    let (input, figure) = digit1(input)?;
    Ok((input, figure))
}
fn get_string<'a>(input: & 'a str) -> IResult<&str, &str>{
    let(input, first) = alt((tag("'"), tag("\"")))(input)?;
    let (input, m_string) = take_until(first)(input)?;
    let (input, _) = tag(first)(input)?;
    Ok((input, m_string))
}
fn get_others<'a>(input: & 'a str) -> IResult<&str, &str>{
    let first = input;
    let result =
        one_of::<_, _, (&str, nom::error::ErrorKind)>(",:()[]")(input);
    if let Ok((input, c)) = result{
        let c = get_str_back(first, input);
        return Ok((input, c));
    }
    let error : IResult<&str, &str> =
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::AlphaNumeric)));
    return error;
}
fn get_word<'a >(input: & 'a str)->IResult<&str, & str> {
    let first = input;
    let mut input = input;
    let c = get_str_first(input);
    if is_alphabetic(c) || c == b'_' || c == b'.'{}
    else {
        let error : IResult<&str, &str> =
        Err(nom::Err::Error(nom::error::Error::new(first, nom::error::ErrorKind::AlphaNumeric)));
        return error;
    }
    loop{
        // get under vars or alphanum
        let (_input, word) = get_word_backward(input)?;
        input = _input;
        if word == ""{break}
    }
    //wordの計算
    let word = get_str_back(first, input);
    Ok((input, word))
}

fn get_word_backward<'a>( input: & 'a str)->IResult<& 'a str, & 'a str>{
    let ch = get_str_first(input);
    if is_alphanumeric(ch){
        let (input, word) = alphanumeric0(input)?;
        Ok((input, word))
    }else if ch == b'_'{
        let (input, _) = many0_count(tag("_"))(input)?;
        // このあと計算イラン get_wordでする get_word で計算している
        Ok((input, "_"))
    }else if ch == b'.'{
        let (input, _) = many0_count(tag("."))(input)?;
        // このあと計算イラン get_wordでする get_word で計算している
        Ok((input, "."))
    }else {
        Ok((input, ""))
    }
}
fn get_str_back<'a >(p: & 'a str, pe: &'a str)-> & 'a str{
    let word = unsafe{std::slice::from_raw_parts(
        p.as_ptr(),
        pe.as_ptr().offset_from(p.as_ptr()) as usize
    )};
    std::str::from_utf8(word).unwrap()
}


fn ignore_comment<'a>(input: & 'a str) -> IResult<&str, &str>{
    let (input, first) = (tag(";"))(input)?;
    // multiple comments
    let (input, _) = take_until("\n")(input)?;
    let (input, _) = tag("\n")(input)?;
    Ok((input, ""))
}
fn is_ignore_comment<'a>(input: & 'a str) -> bool{
    if get_str_first(input) == b';' {true} else {false}
}
fn ignore_space<'a >(input : & 'a str) -> IResult<& 'a str, & 'a str>{
    space1(input)
}
fn ignore_space_and_return<'a>(input: & 'a str) -> IResult<& 'a str, & 'a str>{
    let (input, _) = multispace0(input)?;
    Ok((input, ""))
}
pub struct AsmError<'a >{
    m_str: & 'a str,
}

impl<'a> AsmError<'a>{
    pub fn new(_str: &'a str) -> Self{
        Self{m_str: _str}
    }
    pub fn panic_from_pos(&self, first : usize, message: &str){
        let mut lines = self.m_str.lines();
        let mut loopcnt: usize = 0;
        let mut linecnt = 0;
        let mut rawcnt = 0;
        let mut error_line : &str = "";
        while let Some(line) = lines.next(){
            loopcnt += 1;
            let linefirst = wrapper_pos(self.m_str, line) as usize;

            let lineend = linefirst + line.len();
            if linefirst <= first && first < lineend {
                linecnt = loopcnt;
                // 一応一行目芋締めから始まるため
                rawcnt = first - linefirst + 1;
                error_line = line;
                break;
            }
        }
        let mut spaces = String::from("");
        for _ in 1..rawcnt{
            spaces += " ";
        }
        println!("######ERROR");
        println!("{}, {}", linecnt, rawcnt);
        println!("{}", error_line);
        println!("{}^", spaces);
        println!("{}", message);
        panic!();
    }
    pub fn panic_from_word(&self, word: & 'a str, message: & str){
        self.panic_from_pos(wrapper_pos(self.m_str, word) as usize, message)
    }
    pub fn panic_from_word_idx(&self, word: & 'a str, idx: isize, message: & str){
        self.panic_from_pos((wrapper_pos(self.m_str, word) + idx) as usize,
    message);
    }
}
fn wrapper_pos(origin: &str, branch: &str) -> isize{
    unsafe{branch.as_ptr().offset_from(origin.as_ptr())}
}
fn get_str_first( input: & str) -> u8{
    unsafe{*input.as_ptr()}
}
