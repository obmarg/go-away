use std::fmt::{self, Write};

use indoc::writedoc;

use super::{indented, Union};

pub struct UnionValidate<'a>(pub &'a Union);

impl<'a> fmt::Display for UnionValidate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(
            f,
            "func (u {union_name}) Validate() error {{",
            union_name = self.0.name
        )?;
        writeln!(indented(f), "var count int\n")?;
        for variant in &self.0.variants {
            let f = &mut indented(f);
            writedoc!(
                f,
                r#"
					if u.{variant_name} != nil {{
						count++
					}}

				"#,
                variant_name = variant.go_name()
            )?;
        }
        writedoc!(
            indented(f),
            r#"
			if count != 1 {{
				return fmt.Errorf("one variant must be populated, found %d", count)
			}}

			return nil
			"#
        )?;
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::{
        output::{FieldType, UnionRepresentation, UnionVariant},
        registry::TypeRef,
    };

    #[test]
    fn test_validate_output() {
        assert_snapshot!(UnionValidate (
            &Union {
                name: "MyUnion".into(),
                representation: UnionRepresentation::AdjacentlyTagged {
                    tag: "type".into(),
                    content: "data".into(),
                },
                variants: vec![
                    UnionVariant {
                        name: Some("VarOne".into()),
                        ty: FieldType::Named(TypeRef {
                            name: "VarOne".into()
                        }),
                        serialized_name: "VAR_ONE".into(),
                    },
                    UnionVariant {
                        name: Some("VarTwo".into()),
                        ty: FieldType::Named(TypeRef {
                            name: "VarTwo".into()
                        }),
                        serialized_name: "VAR_TWO".into(),
                    }
                ]
            }
        )
        .to_string(),
        @r###"
        func (u MyUnion) Validate() error {
        	var count int

        	if u.VarOne != nil {
        		count++
        	}

        	if u.VarTwo != nil {
        		count++
        	}

        	if count != 1 {
        		return fmt.Errorf("one variant must be populated, found %d", count)
        	}

        	return nil
        }
        "###
        );
    }
}
