use std::mem;
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Default)]
#[repr(packed)]
pub struct FILE_HEADER{
    pub Machine: u16,
    pub NumberOfSections: u16,
    pub TimeDataStamp: u32,
    pub PointerToSymbolTable: u32,
    pub NumberOfSymbols: u32,
    pub SizeOfOptionalHeader: u16,
    pub Characteristics: u16,
}
impl FILE_HEADER{
    pub fn new() ->Self{
        Self{Machine:0,NumberOfSections:0,TimeDataStamp:0, PointerToSymbolTable:0,
            NumberOfSymbols:0, SizeOfOptionalHeader:0, Characteristics:0}
    }
}
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Default)]
#[repr(packed)]
pub struct SECTION_HEADER{
    pub Name: [u8;8],
    pub VirtualSize: u32,
    pub VirtualAddress: u32,
    pub SizeOfRawData: u32,
    pub PointerToRawData: u32,
    pub PointerToRelocations: u32,
    pub PointerToLinenumbers: u32,
    pub NumberOfRelocations: u16,
    pub NumberOfLinenumbers: u16,
    pub Characteristics: u32,
}
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Default)]
#[repr(packed)]
pub struct RELOCATION{
    pub VirtualAddress: u32,
    pub SymbolTableIndex: u32,
    pub Type: u16,
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Default)]
#[repr(packed)]
pub struct SYMBOL_TABLE{
    pub Name: [u8;8],
    pub Value: u32,
    pub SectionNumber: u16,
    pub Type: u16,
    pub StorageClass: u8,
    pub NumberOfAuxSymbols: u8,
}
impl SYMBOL_TABLE{
    pub fn new_dot_file() -> SYMBOL_TABLE{
        let mut ret = Self::default();
        ret.Name[0..5].copy_from_slice(&".file".as_bytes());
        ret.Value = 0;
        ret.SectionNumber = 0xFFFE;
        ret.Type = 0;
        ret.StorageClass = 0x67;
        ret.NumberOfAuxSymbols = 1;
        
        ret
    }
}
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Default)]
#[repr(packed)]
pub struct SYMBOL_U8{
    pub data: [u8;0x12]
}
impl SYMBOL_U8{
    pub fn set<T: Sized + Copy + PartialOrd>(&mut self, p: &T, offset: usize){
        let tsize = mem::size_of::<T>();
        self.data[offset..offset+tsize].copy_from_slice(as_u8_slice(p));
    }
}
pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8]{
    unsafe{
    std::slice::from_raw_parts(
        (p as * const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
    }
}

pub fn as_u8_slice_size<T: Sized>(p: &T,size :usize)-> &[u8]{
    unsafe{
    std::slice::from_raw_parts(
        (p as * const T) as *const u8,
        size,
    )
    }
}

