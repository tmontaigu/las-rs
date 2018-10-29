use num::{Num, NumCast, cast};
use error;
use std::io::{Read, Cursor, SeekFrom, Seek};
use byteorder::{ReadBytesExt, LittleEndian};
use point::Error;
use Result;
use vlr::Vlr;

use std::collections::HashMap;


#[allow(missing_docs)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ExtraDimTypes {
    // Unsigned integer Types
    U8,
    U16,
    U32,
    U64,
    
    // Signed integer types
    I8,
    I16,
    I32,
    I64,

    // Floating point types
    F32,
    F64,
}


#[allow(missing_docs)]
enum DimensionValue {
    // Unsigned integer Types
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    
    // Signed integer types
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    // Floating point types
    F32(f32),
    F64(f64),
}

impl ExtraDimTypes {
    pub fn size(&self) -> usize {
        match self {
            ExtraDimTypes::U8 => 1,
            ExtraDimTypes::I8 => 1,
            ExtraDimTypes::U16 => 2,
            ExtraDimTypes::I16 => 2,
            ExtraDimTypes::U32 => 4,
            ExtraDimTypes::I32 => 4,
            ExtraDimTypes::U64 => 8,
            ExtraDimTypes::I64 => 8,
            ExtraDimTypes::F32 => 8,
            ExtraDimTypes::F64 => 8,
        }
    }
}

fn value_to_type(value_type: u8) -> ExtraDimTypes {
    match value_type {
        1 => ExtraDimTypes::U8,
        2 => ExtraDimTypes::I8,
        3 => ExtraDimTypes::U16,
        4 => ExtraDimTypes::I16,
        5 => ExtraDimTypes::U32,
        6 => ExtraDimTypes::I32,
        7 => ExtraDimTypes::U64,
        8 => ExtraDimTypes::I64,
        9 => ExtraDimTypes::F32,
        10 => ExtraDimTypes::F64,
        _ => ExtraDimTypes::F64,
    }
}


#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct ExtraBytes {
    bytes: Vec<u8>
}

fn read_extra(rdr: &mut Cursor<Vec<u8>>, t: ExtraDimTypes) -> std::io::Result<DimensionValue> {
    match t {
        // No little endian for u8 & i8 ?¿
        ExtraDimTypes::U8 => Ok(DimensionValue::U8(rdr.read_u8()?)),
        ExtraDimTypes::U16 => Ok(DimensionValue::U16(rdr.read_u16::<LittleEndian>()?)),
        ExtraDimTypes::U32 => Ok(DimensionValue::U32(rdr.read_u32::<LittleEndian>()?)),
        ExtraDimTypes::U64 => Ok(DimensionValue::U64(rdr.read_u64::<LittleEndian>()?)),

        ExtraDimTypes::I8 => Ok(DimensionValue::I8(rdr.read_i8()?)),
        ExtraDimTypes::I16 => Ok(DimensionValue::I16(rdr.read_i16::<LittleEndian>()?)),
        ExtraDimTypes::I32 => Ok(DimensionValue::I32(rdr.read_i32::<LittleEndian>()?)),
        ExtraDimTypes::I64 => Ok(DimensionValue::I64(rdr.read_i64::<LittleEndian>()?)),

        ExtraDimTypes::F32 => Ok(DimensionValue::F32(rdr.read_f32::<LittleEndian>()?)),
        ExtraDimTypes::F64 => Ok(DimensionValue::F64(rdr.read_f64::<LittleEndian>()?)),
    }
}

fn cast_extra<T: Num + NumCast>(value: DimensionValue) -> Option<T> {
    match value {
        DimensionValue::U8(v) => cast::<u8, T>(v),
        DimensionValue::U16(v) => cast::<u16, T>(v),
        DimensionValue::U32(v) => cast::<u32, T>(v),
        DimensionValue::U64(v) => cast::<u64, T>(v),

        DimensionValue::I8(v) => cast::<i8, T>(v),
        DimensionValue::I16(v) => cast::<i16, T>(v),
        DimensionValue::I32(v) => cast::<i32, T>(v),
        DimensionValue::I64(v) => cast::<i64, T>(v),

        DimensionValue::F32(v) => cast::<f32, T>(v),
        DimensionValue::F64(v) => cast::<f64, T>(v),
    }
}

impl ExtraBytes {
    #[allow(missing_docs)]
    pub fn new(bytes_: Vec<u8>) -> Self {
        ExtraBytes{bytes: bytes_}
    }

    #[allow(missing_docs)]
    pub fn get_field_as<T: Num + NumCast>(&self, name: &str) -> Result<T> {
        let mut rdr = Cursor::new(self.bytes.clone());


        // let value: u16 = rdr.read_u16::<LittleEndian>()?;
        let value = read_extra(&mut rdr, ExtraDimTypes::U16);


        // let value = cast::<u16, T>(value);
        let value = cast_extra::<T>(value.unwrap());

        match value {
            Some(v) => Ok(v),
            None => Err(error::Error::NotAscii("lol".to_string()).into())
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct RawExtraByteStruct {
    reserved: [u8; 2],
    data_type: u8,
    options: u8,
    name: [u8; 32],
    unused: [u8; 4], 
    no_data: [u8; 24],
    min: [u8; 24],
    max: [u8; 24],
    scale: [u8; 24],
    offset: [u8; 24],
    description: [u8; 32]
}

impl Default for RawExtraByteStruct {
    fn default() -> RawExtraByteStruct {
        RawExtraByteStruct {
            reserved: [0; 2],
            data_type: 0,
            options: 0,
            name: [0; 32],
            unused: [0; 4], 
            no_data: [0; 24],
            min: [0; 24],
            max: [0; 24],
            scale: [0; 24],
            offset: [0; 24],
            description: [0; 32]
        }
    }
}

const RawExtraByteStructSize: usize = 192;

impl RawExtraByteStruct {
    pub fn read_from<R: Read>(source: &mut R) -> std::io::Result<Self> {
        let mut ebs = RawExtraByteStruct::default();

        source.read_exact(&mut ebs.reserved)?;
        ebs.data_type = source.read_u8()?;
        ebs.options = source.read_u8()?;
        source.read_exact(&mut ebs.name)?;
        source.read_exact(&mut ebs.unused)?;
        source.read_exact(&mut ebs.no_data)?;
        source.read_exact(&mut ebs.min)?;
        source.read_exact(&mut ebs.max)?;
        source.read_exact(&mut ebs.scale)?;
        source.read_exact(&mut ebs.offset)?;
        source.read_exact(&mut ebs.description)?;
        
        Ok(ebs)
    }

    pub fn name(&self) -> String {
        let mut first_zero = 0;
        for c in &self.name {
            if *c == 0 {
                break;
            }
            first_zero += 1;
        }
        let mut name = String::from_utf8_lossy(&self.name).to_string();
        name.truncate(first_zero);
        name
    }

    pub fn data_type(&self) -> ExtraDimTypes {
        value_to_type(self.data_type)
    }
}

#[derive(Clone, Debug)]
pub struct ExtraBytesParser {
    //FIXME HashMap if it preserve insert order ?
    pub ebss: Vec<RawExtraByteStruct>,
}

fn find_extra_bytes_vlr(vlrs: &Vec<Vlr>) -> Option<&Vlr> {
    let mut eb_vlr: Option<&Vlr> = None;
    for vlr in vlrs {
        if vlr.record_id == 4 {
            eb_vlr = Some(vlr);
        }
    }
    eb_vlr
}

impl ExtraBytesParser {
    pub fn from_vlrs(vlrs: &Vec<Vlr>) -> Option<ExtraBytesParser> {

        let eb_vlr: &Vlr;
        if let Some(vlr) = find_extra_bytes_vlr(vlrs) {
            eb_vlr = vlr;
        } else {
            return None;
        }

        // TODO check size % 192 == 0
        let num_ebs = eb_vlr.data.len() / RawExtraByteStructSize;

        // let mut ebs_map = HashMap::<String, RawExtraByteStruct>::new();
        let mut ebs_vec = Vec::<RawExtraByteStruct>::new();
        let mut source = Cursor::new(eb_vlr.data.clone());
        for i in 0..num_ebs {
            // FIXME Bad Unwrap
            let e = RawExtraByteStruct::read_from(&mut source).unwrap();
            // let e_name = String::from_utf8_lossy(&e.name);
            // ebs_map.insert(e_name.to_string(), e);
            ebs_vec.push(e);
        }

        Some(ExtraBytesParser{ebss: ebs_vec})
    }

    pub fn get_field_as<T: Num + NumCast>(&self, bytes: &Vec<u8>, name: &str) -> Result<T> {
        let mut rdr = Cursor::new(bytes.clone());

        let mut offset: u64 = 0;
        let mut corresponding_eb: Option<&RawExtraByteStruct> = None;
        for ebs in &self.ebss {
            println!("'{}' '{}' {} => {}", ebs.name(), name, ebs.name() == name,ebs.data_type().size());
            if ebs.name() == name {
                corresponding_eb = Some(ebs);
                break;
            }
            offset += ebs.data_type().size() as u64;
        }
        println!("Seeking to {} while len is {}", offset, bytes.len());
        rdr.seek(SeekFrom::Start(offset))?;

        if !corresponding_eb.is_some() {
            return Err(error::Error::NotAscii("lol".to_string()).into());
        }
        

        let value = read_extra(&mut rdr, corresponding_eb.unwrap().data_type());
        let value = cast_extra::<T>(value.unwrap());

        match value {
            Some(v) => Ok(v),
            None => Err(error::Error::NotAscii("lol".to_string()).into())
        }
    }
}