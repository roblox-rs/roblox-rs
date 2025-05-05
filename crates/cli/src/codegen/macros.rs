macro_rules! list {
    ($ctx:ident, $list:expr; |$el:ident| $b:expr) => {
        for (done, $el) in $list.iter().until_done() {
            write!($ctx, "{}", $b)?;

            if !done {
                write!($ctx, ", ")?;
            }
        }
    };
    ($ctx:ident, $list:expr) => {{
        let result = $list.join(", ");
        write!($ctx, "{result}")?;
    }};
}

pub(crate) use list;

macro_rules! text {
	// Write, no new line
	($ctx:ident, $($tt:tt)*) => {{
		write!($ctx, $($tt)*)?;
	}};
}

pub(crate) use text;

macro_rules! line {
	($ctx:ident, $($tt:tt)*) => {{
		writeln!($ctx, $($tt)*)?;
	}};
	($ctx:ident) => {{
		writeln!($ctx)?;
	}};

}

pub(crate) use line;

macro_rules! push {
	($ctx:ident, $($tt:tt)*) => {{
		writeln!($ctx, $($tt)*)?;
		write!($ctx, "\x0E")?;
	}};
}

pub(crate) use push;

macro_rules! pull {
	($ctx:ident, $($tt:tt)*) => {{
		write!($ctx, "\x0F")?;
		writeln!($ctx, $($tt)*)?;
	}};
}

pub(crate) use pull;
