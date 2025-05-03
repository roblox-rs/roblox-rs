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
    Ref,
    RefMut,
    ExternRef,
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
            | Describe::Ref
            | Describe::RefMut
            | Describe::ExternRef => 1,
            Describe::Function { .. } => unimplemented!(),
            Describe::Option { ty } => 1 + ty.value_count(),
        }
    }

    pub fn memory_size(&self) -> usize {
        self.primitive_values().iter().map(|v| v.byte_size()).sum()
    }

    pub fn primitive_values(&self) -> Vec<Primitive> {
        let mut vec = Vec::new();
        self._primitive_values(&mut vec);
        vec
    }

    fn _primitive_values(&self, out: &mut Vec<Primitive>) {
        match self {
            Describe::U8 | Describe::U16 | Describe::U32 => out.push(Primitive::U32),
            Describe::I8 | Describe::I16 | Describe::I32 => out.push(Primitive::I32),
            Describe::Boolean => out.push(Primitive::U32),
            Describe::ExternRef => out.push(Primitive::U32),
            Describe::Void => {}
            Describe::F32 => out.push(Primitive::F32),
            Describe::F64 => out.push(Primitive::F64),
            Describe::Ref | Describe::RefMut => unimplemented!(),
            Describe::Function { .. } => unimplemented!(),
            Describe::Option { ty } => {
                out.push(Primitive::U32);
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
            REF => Describe::Ref,
            REF_MUT => Describe::RefMut,
            VOID => Describe::Void,
            F32 => Describe::F32,
            F64 => Describe::F64,
            EXTERNREF => Describe::ExternRef,
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
    U32,
    I32,
    F32,
    F64,
}

impl Primitive {
    pub fn byte_size(&self) -> usize {
        match self {
            Primitive::U32 => 4,
            Primitive::I32 => 4,
            Primitive::F32 => 4,
            Primitive::F64 => 8,
        }
    }

    pub fn buffer_name(&self) -> &'static str {
        match self {
            Primitive::F32 => "f32",
            Primitive::F64 => "f64",
            Primitive::I32 => "i32",
            Primitive::U32 => "u32",
        }
    }
}
