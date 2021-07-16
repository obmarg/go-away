use std::fmt::{Result, Write};

/// Converts any 4 spaces written to this `fmt::Write` into tabs.
///
/// This allows us to write templates like we're writing rust but have the output be tabbed,
/// as is idiomatic in go.
pub fn tabify<D: ?Sized>(inner: &'_ mut D) -> Tabify<'_, D> {
    Tabify { inner }
}

/// A Wrapper around a `core::fmt::Write` that converts 4 spaces into tabs
/// This allows us to write templates like we're writing rust but have the output be tabbed,
/// as is idiomatic in go.
pub struct Tabify<'a, D: ?Sized> {
    inner: &'a mut D,
}

impl<T> Write for Tabify<'_, T>
where
    T: Write + ?Sized,
{
    fn write_str(&mut self, s: &str) -> Result {
        for (ind, line) in s.split("    ").enumerate() {
            if ind > 0 {
                self.inner.write_char('\t')?;
            }

            self.inner.write_fmt(format_args!("{}", line))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use indoc::writedoc;

    use super::*;

    #[test]
    fn test_tabify() {
        let mut buffer = String::new();
        writedoc!(
            tabify(&mut buffer),
            r#"
			Hello There
				Hopefully I am indented with tabs
					Ever so slightly at least
				You tell me?
						What about now?
						Is this indented more?
			"#
        )
        .unwrap();

        insta::assert_snapshot!(buffer, @r###"
        Hello There
        	Hopefully I am indented with tabs
        		Ever so slightly at least
        	You tell me?
        			What about now?
        			Is this indented more?
        "###);
    }
}
