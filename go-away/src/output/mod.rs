pub mod go;
pub mod kotlin;
pub mod swift;
pub mod typescript;

pub use go::GoType;
pub use kotlin::KotlinType;
pub use swift::SwiftType;
pub use typescript::TypeScriptType;

mod tabify;

mod prelude {
    pub use std::fmt::{self, Write};

    pub use indenter::indented;
    pub use indoc::writedoc;

    macro_rules! writeln_for {
        ($f:expr, $p:pat in $e:expr, $($arg:tt)*) => {{
            #[allow(unused_mut)]
            let mut f = $f;
            for $p in $e {
                writeln!(f, $($arg)*)?;
            }
        }}
    }

    macro_rules! writedoc_for {
        ($f:expr, $p:pat in $e:expr, $($arg:tt)*) => {{
            #[allow(unused_mut)]
            let mut f = $f;
            for $p in $e {
                writedoc!(f, $($arg)*)?;
            }
        }}
    }

    pub(super) use writedoc_for;
    pub(super) use writeln_for;
}
