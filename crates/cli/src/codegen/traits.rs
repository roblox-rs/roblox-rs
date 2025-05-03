use std::{
    collections::HashMap,
    io::{self, Write},
};

use roblox_rs_shared_context::shared_context::SharedIntrinsic;

use super::macros::line;

pub trait Instruction {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()>;
    fn get_inputs(&self) -> usize;
    fn get_outputs(&self) -> usize;
}

pub struct InstructionContext<'a> {
    pub inputs: Vec<String>,
    pub renderer: IndentedWriter<&'a mut dyn Write>,
    pub vars: Vars,
    pub intrinsics: Intrinsics<'a>,
}

impl<'a> InstructionContext<'a> {
    pub fn new(renderer: &'a mut dyn Write, intrinsics: &'a [SharedIntrinsic]) -> Self {
        Self {
            renderer: IndentedWriter::new(renderer),
            inputs: Vec::new(),
            vars: Vars {
                used_names: vec![HashMap::new()],
            },
            intrinsics: Intrinsics {
                intrinsics,
                used: Vec::new(),
            },
        }
    }

    pub fn push(&mut self, value: impl Into<String>) {
        self.inputs.push(value.into())
    }

    pub fn pop(&mut self) -> String {
        self.inputs.pop().expect("inputs is empty")
    }

    #[allow(unused)]
    pub fn pop_array<const T: usize>(&mut self) -> [String; T] {
        self.inputs
            .split_off(self.inputs.len() - T)
            .try_into()
            .map_err(|_| "invalid number of elements")
            .unwrap()
    }

    pub fn pop_many(&mut self, count: usize) -> Vec<String> {
        self.inputs.split_off(self.inputs.len() - count)
    }

    pub fn pop_complex(&mut self) -> io::Result<String> {
        let value = self.pop();
        self.prereq_complex(value)
    }

    fn prereq_complex(&mut self, expr: String) -> io::Result<String> {
        // If this expression isn't an identifier, we add it to a variable as it might have side effects.
        if expr.chars().any(|v| !v.is_alphanumeric() && v != '_') {
            let prereq_id = self.vars.next("prereq");
            line!(self, "local {prereq_id} = {expr}");
            Ok(prereq_id)
        } else {
            Ok(expr)
        }
    }
}

impl Write for InstructionContext<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.renderer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.renderer.flush()
    }
}

#[allow(unused)]
pub struct Intrinsics<'a> {
    pub used: Vec<&'a str>,
    pub intrinsics: &'a [SharedIntrinsic],
}

impl Intrinsics<'_> {
    #[allow(unused)]
    pub fn get(&mut self, name: &'static str) -> String {
        let Some(intrinsic) = self.intrinsics.iter().find(|v| v.name == name) else {
            panic!("unknown intrinsic '{name}'")
        };

        if !self.used.contains(&name) {
            self.used.push(name);
        }

        intrinsic.export_name.clone()
    }
}

pub struct Vars {
    pub used_names: Vec<HashMap<&'static str, usize>>,
}

impl Vars {
    fn names(&mut self) -> &mut HashMap<&'static str, usize> {
        self.used_names.last_mut().unwrap()
    }

    pub fn next(&mut self, name: &'static str) -> String {
        let names = self.names();
        let next_id = *names.get(name).unwrap_or(&0);
        names.insert(name, next_id + 1);

        format!("{name}_{next_id}")
    }

    pub fn many(&mut self, count: usize, name: &'static str) -> Vec<String> {
        (0..count).map(|_| self.next(name)).collect()
    }

    pub fn scope(&mut self) {
        let cloned = self.names().clone();
        self.used_names.push(cloned)
    }

    pub fn unscope(&mut self) {
        assert_ne!(self.used_names.len(), 1, "cannot unscope last scope");
        self.used_names.pop();
    }
}

pub struct IndentedWriter<T: Write> {
    writer: T,
    indent: usize,
    new_line: bool,
}

impl<T: Write> IndentedWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer,
            indent: 0,
            new_line: true,
        }
    }

    pub fn up(&mut self) {
        self.indent += 1;
    }

    pub fn down(&mut self) {
        self.indent -= 1;
    }
}

impl<T: Write> Write for IndentedWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match buf[0] {
            // Ascii Shift Out
            b'\x0E' => {
                self.up();
                return Ok(1);
            }
            // Ascii Shift In
            b'\x0F' => {
                self.down();
                return Ok(1);
            }
            _ => {}
        }

        let mut written = 0;

        for buf in buf.split_inclusive(|v| *v == b'\n') {
            if self.new_line {
                self.new_line = false;
                write!(self.writer, "{}", "\t".repeat(self.indent))?;
            }

            if buf.last().copied() == Some(b'\n') {
                self.new_line = true;
            }

            written += self.writer.write(buf)?;
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
