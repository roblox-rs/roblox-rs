use std::io::{self, Write};

pub trait Render {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()>;
}

impl<T: Render> Render for &T {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        (*self).render(ctx)
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

pub struct RenderContext<'a> {
    pub writer: IndentedWriter<&'a mut dyn Write>,
    pub prereq: RenderPrereqContext,
}

impl<'a> RenderContext<'a> {
    pub fn new(out: &'a mut dyn Write) -> Self {
        Self {
            writer: IndentedWriter::new(out),
            prereq: RenderPrereqContext { prereq: None },
        }
    }

    pub fn up(&mut self) {
        self.writer.up();
    }

    pub fn down(&mut self) {
        self.writer.down();
    }

    pub fn render(&mut self, ast: impl Render) -> io::Result<()> {
        ast.render(self)
    }

    /// Renders this implementation as an expression.
    /// This will write prereqs in-place, and return the resulting expression.
    /// A Render implementation might not return an expression, so it is optional.
    pub fn render_expr(&mut self, ast: impl Render) -> io::Result<(Vec<u8>, Option<String>)> {
        let mut writer = Vec::new();
        let mut context = RenderContext::new(&mut writer);
        context.prereq = RenderPrereqContext {
            prereq: Some(IndentedWriter::new(Vec::new())),
        };

        context.render(ast)?;

        let prereqs = context.prereq.prereq.unwrap().writer;
        let expression = String::from_utf8(writer).expect("decode failure");
        Ok((
            prereqs,
            if expression.is_empty() {
                None
            } else {
                Some(expression)
            },
        ))
    }
}

impl Write for RenderContext<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub struct RenderPrereqContext {
    prereq: Option<IndentedWriter<Vec<u8>>>,
}

impl RenderPrereqContext {
    pub fn up(&mut self) {
        self.writer().up();
    }

    pub fn down(&mut self) {
        self.writer().down();
    }

    fn writer(&mut self) -> &mut IndentedWriter<Vec<u8>> {
        self.prereq.as_mut().expect("prereq is not ready")
    }
}

impl Write for RenderPrereqContext {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer().flush()
    }
}
