const U8: u32 = 0;
const U16: u32 = 1;
const U32: u32 = 2;
const I8: u32 = 3;
const I16: u32 = 4;
const I32: u32 = 5;
const BOOLEAN: u32 = 6;
const REF: u32 = 7;
const REF_MUT: u32 = 8;
const FUNCTION: u32 = 9;
const VOID: u32 = 10;
const F32: u32 = 11;
const F64: u32 = 12;
const OPTION: u32 = 13;
const EXTERNREF: u32 = 14;
const STRING: u32 = 15;
const SLICE: u32 = 16;
const VECTOR: u32 = 17;

#[derive(Debug, Clone)]
pub enum Describe {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    Boolean,
    Void,
    F32,
    F64,
    ExternRef,
    String,
    Vector {
        ty: Box<Describe>,
    },
    Slice {
        ty: Box<Describe>,
    },
    Ref {
        ty: Box<Describe>,
    },
    RefMut {
        ty: Box<Describe>,
    },
    Function {
        args: Vec<Describe>,
        return_type: Box<Describe>,
    },
    Option {
        ty: Box<Describe>,
    },
}

impl Describe {
    pub fn value_count(&self) -> usize {
        match self {
            Describe::Void => 0,
            Describe::U8
            | Describe::U16
            | Describe::U32
            | Describe::I8
            | Describe::I16
            | Describe::I32
            | Describe::F32
            | Describe::F64
            | Describe::Boolean
            | Describe::ExternRef => 1,
            Describe::Function { .. } => unimplemented!(),
            Describe::Option { ty } => 1 + ty.value_count(),
            Describe::Ref { ty } => ty.value_count(),
            Describe::RefMut { ty } => ty.value_count(),
            Describe::Slice { .. } => 2,
            Describe::Vector { .. } => 2,
            Describe::String => 2,
        }
    }

    pub fn memory_size(&self) -> usize {
        let mut size = 0;
        let mut max_align = 0;

        for prim in self.primitive_values() {
            let byte_size = prim.byte_size();
            size = prim.next_align(size) + byte_size;
            max_align = max_align.max(byte_size - 1);
        }

        (size + max_align) & !max_align
    }

    pub fn primitive_values(&self) -> Vec<Primitive> {
        let mut vec = Vec::new();
        self._primitive_values(&mut vec);
        vec
    }

    fn _primitive_values(&self, out: &mut Vec<Primitive>) {
        match self {
            Describe::U8 => out.push(Primitive::U8),
            Describe::U16 => out.push(Primitive::U16),
            Describe::U32 => out.push(Primitive::U32),
            Describe::I8 => out.push(Primitive::I8),
            Describe::I16 => out.push(Primitive::I16),
            Describe::I32 => out.push(Primitive::I32),
            Describe::Boolean => out.push(Primitive::U32),
            Describe::ExternRef => out.push(Primitive::U32),
            Describe::Void => {}
            Describe::F32 => out.push(Primitive::F32),
            Describe::F64 => out.push(Primitive::F64),
            Describe::String => out.extend([Primitive::U32, Primitive::U32]),
            Describe::Vector { .. } => out.extend([Primitive::U32, Primitive::U32]),
            Describe::Slice { .. } => out.extend([Primitive::U32, Primitive::U32]),
            Describe::Function { .. } => unimplemented!(),
            Describe::Ref { ty } | Describe::RefMut { ty } => ty._primitive_values(out),
            Describe::Option { ty } => {
                out.push(Primitive::U8);
                ty._primitive_values(out);
            }
        }
    }

    pub fn parse(mut value: &[u32]) -> Describe {
        Describe::_parse(&mut value)
    }

    fn _parse(value: &mut &[u32]) -> Describe {
        match Describe::take(value) {
            U8 => Describe::U8,
            U16 => Describe::U16,
            U32 => Describe::U32,
            I8 => Describe::I8,
            I16 => Describe::I16,
            I32 => Describe::I32,
            BOOLEAN => Describe::Boolean,
            VOID => Describe::Void,
            F32 => Describe::F32,
            F64 => Describe::F64,
            EXTERNREF => Describe::ExternRef,
            STRING => Describe::String,
            VECTOR => Describe::Vector {
                ty: Box::new(Describe::_parse(value)),
            },
            SLICE => Describe::Slice {
                ty: Box::new(Describe::_parse(value)),
            },
            REF => Describe::Ref {
                ty: Box::new(Describe::_parse(value)),
            },
            REF_MUT => Describe::RefMut {
                ty: Box::new(Describe::_parse(value)),
            },
            FUNCTION => {
                let arg_count = Describe::take(value);

                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(Describe::_parse(value));
                }

                Describe::Function {
                    args,
                    return_type: Box::new(Describe::_parse(value)),
                }
            }
            OPTION => Describe::Option {
                ty: Box::new(Describe::_parse(value)),
            },
            _ => panic!("something is wrong"),
        }
    }

    fn take(value: &mut &[u32]) -> u32 {
        let first = value[0];
        *value = &value[1..];
        first
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
}

impl Primitive {
    pub fn byte_size(&self) -> usize {
        match self {
            Primitive::U8 => 1,
            Primitive::U16 => 2,
            Primitive::U32 => 4,
            Primitive::I8 => 1,
            Primitive::I16 => 2,
            Primitive::I32 => 4,
            Primitive::F32 => 4,
            Primitive::F64 => 8,
        }
    }

    pub fn next_align(&self, offset: usize) -> usize {
        let align = self.byte_size() - 1;

        (offset + align) & !align
    }

    pub fn buffer_name(&self) -> &'static str {
        match self {
            Primitive::F32 => "f32",
            Primitive::F64 => "f64",
            Primitive::I8 => "i8",
            Primitive::I16 => "i16",
            Primitive::I32 => "i32",
            Primitive::U8 => "u8",
            Primitive::U16 => "u16",
            Primitive::U32 => "u32",
        }
    }
}
