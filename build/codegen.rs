//! Module with code generation utilities.

use std::fmt;
use std::io::{self, Write};
use std::thread;


/// The whitespace string that's used as one level of indentation.
const TAB: &str = "    ";


/// Code generation context object.
///
/// This objects keep the state of the code generation process and allows to delineate
/// some rudimentary structure (mostly indented blocks) that it helps maintain.
#[derive(Debug)]
pub struct Context<W: Write> {
    /// The output writer.
    writer: W,
    /// Current indentation level, in "tabs".
    indent_level: usize,
    /// How many indentation levels are yet to be applied to the current line.
    pending_line_indents: usize,
}

impl<W: Write> Context<W> {
    /// Create a new code generation `Context`.
    #[inline]
    pub fn new(writer: W) -> Self {
        Context {
            writer: writer,
            indent_level: 0,
            pending_line_indents: 0,
        }
    }
}
impl<W: Write> Drop for Context<W> {
    fn drop(&mut self) {
        if thread::panicking() {
            return;
        }
        if self.indent_level > 0 {
            panic!(format!("Dropping a Context with non-zero indent level (got {})",
                self.indent_level));
        }
    }
}

// Indentation / code blocks control.
impl<W: Write> Context<W> {
    /// Begin a new indented block with the given header.
    ///
    /// Use this for braced blocks (loops, ifs, etc.), function calls that span
    /// multiple lines, and so on.
    pub fn begin(&mut self, head: &str) -> io::Result<&mut Self> {
        writeln!(self.writer, "{}", head)?;
        self.indent();
        Ok(self)
    }

    /// Decrease the indentation level by one.
    pub fn dedent(&mut self) -> &mut Self {
        if self.indent_level == 0 {
            panic!("Context::dedent() with no indentation");
        }
        self.indent_level -= 1;
        if self.pending_line_indents > 0 {
            // If current line hasn't been indented yet,
            // adjust its indent level to be lower.
            // Otherwise, the dedent will become applicable starting from the next line.
            self.pending_line_indents = self.indent_level;
        }
        self
    }

    /// End a block of code with given footer (usually the closing curly brace).
    pub fn end(&mut self, footer: &str) -> io::Result<&mut Self> {
        self.dedent();
        writeln!(self.writer, "{}", footer)?;
        Ok(self)
    }

    /// Increase the indentation level by one.
    pub fn indent(&mut self) -> &mut Self {
        self.indent_level += 1;
        self.pending_line_indents += 1;
        self
    }
}

impl<W: Write> Context<W> {
    /// Emit a line of code at the current indentation level.
    pub fn emit<L: AsRef<str>>(&mut self, line: L) -> io::Result<&mut Self> {
        let mut line = line.as_ref();

        // Don't be too fussy about a line ending with single newline,
        // but otherwise there shouldn't be any.
        if line.ends_with("\n") && !line.ends_with("\n\n") {
            line = line.trim_right_matches("\n");
        }
        if line.contains('\n') {
            panic!(format!("Context::emit() got a string with newline: {:?}", line));
        }

        while self.pending_line_indents > 0 {
            write!(self.writer, "{}", TAB)?;
            self.pending_line_indents -= 1;
        }
        writeln!(self.writer, "{}\n", line)?;
        self.pending_line_indents = self.indent_level;
        Ok(self)
    }

    /// Emit a formatted line of code at the current indentation level.
    ///
    /// Note that you should typically use the `emit!` macro instead.
    pub fn emit_fmt(&mut self, args: fmt::Arguments) -> io::Result<&mut Self> {
        self.emit(format!("{}", args))
    }
}

/// Emit a formatted line of code to given `Context`.
macro_rules! emit {
    ($ctx:expr, $fmt:expr, $($args:tt)*) => (
        $ctx.emit_fmt(format_args!($fmt, $($args)*))
    );
}
