use crate::operator::{MutantInfo, MutationOperator};
use crate::operators::{
    MOVE_ADDR_MAX, MOVE_ADDR_ZERO, MOVE_FALSE, MOVE_MAX_INFERRED_NUM, MOVE_MAX_U128, MOVE_MAX_U16,
    MOVE_MAX_U256, MOVE_MAX_U32, MOVE_MAX_U64, MOVE_MAX_U8, MOVE_TRUE, MOVE_ZERO, MOVE_ZERO_U128,
    MOVE_ZERO_U16, MOVE_ZERO_U256, MOVE_ZERO_U32, MOVE_ZERO_U64, MOVE_ZERO_U8,
};
use crate::report::{Mutation, Range};
use codespan::FileId;
use move_model::ast::Value;
use move_model::model::Loc;
use move_model::ty::{PrimitiveType, Type};
use num_traits::cast::ToPrimitive;
use std::fmt;
use std::fmt::Debug;

pub const OPERATOR_NAME: &str = "literal_replacement";

/// Literal replacement mutation operator.
/// Replaces literal statements with other ones but withing the same type.
#[derive(Debug, Clone)]
pub struct Literal {
    operation: Value,
    optype: Type,
    loc: Loc,
}

impl Literal {
    /// Creates a new instance of the literal mutation operator.
    #[must_use]
    pub fn new(operation: Value, optype: Type, loc: Loc) -> Self {
        Self {
            operation,
            optype,
            loc,
        }
    }
}

impl MutationOperator for Literal {
    fn apply(&self, source: &str) -> Vec<MutantInfo> {
        let start = self.loc.span().start().to_usize();
        let end = self.loc.span().end().to_usize();
        let cur_op = &source[start..end];

        // Group of literal statements for possible Value types.
        let ops: Vec<String> = match &self.optype {
            Type::Primitive(PrimitiveType::Address) => {
                vec![MOVE_ADDR_ZERO.to_owned(), MOVE_ADDR_MAX.to_owned()]
            },
            Type::Primitive(PrimitiveType::Bool) => {
                vec![MOVE_TRUE.to_owned(), MOVE_FALSE.to_owned()]
            },
            Type::Primitive(PrimitiveType::U8) => {
                if let Value::Number(bigint) = &self.operation {
                    let u8_val = bigint.to_u8().expect("Invalid u8 value");

                    vec![
                        MOVE_ZERO_U8.to_owned(),
                        MOVE_MAX_U8.to_owned(),
                        u8_val.saturating_add(1).to_string(),
                        u8_val.saturating_sub(1).to_string(),
                    ]
                } else {
                    vec![]
                }
            },
            Type::Primitive(PrimitiveType::U16) => {
                if let Value::Number(bigint) = &self.operation {
                    let u16_val = bigint.to_u16().expect("Invalid u16 value");

                    vec![
                        MOVE_ZERO_U16.to_owned(),
                        MOVE_MAX_U16.to_owned(),
                        u16_val.saturating_add(1).to_string(),
                        u16_val.saturating_sub(1).to_string(),
                    ]
                } else {
                    vec![]
                }
            },
            Type::Primitive(PrimitiveType::U32) => {
                if let Value::Number(bigint) = &self.operation {
                    let u32_val = bigint.to_u32().expect("Invalid u32 value");

                    vec![
                        MOVE_ZERO_U32.to_owned(),
                        MOVE_MAX_U32.to_owned(),
                        u32_val.saturating_add(1).to_string(),
                        u32_val.saturating_sub(1).to_string(),
                    ]
                } else {
                    vec![]
                }
            },
            Type::Primitive(PrimitiveType::U64) => {
                if let Value::Number(bigint) = &self.operation {
                    let u64_val = bigint.to_u64().expect("Invalid u64 value");
                    vec![
                        MOVE_ZERO_U64.to_owned(),
                        MOVE_MAX_U64.to_owned(),
                        u64_val.saturating_add(1).to_string(),
                        u64_val.saturating_sub(1).to_string(),
                    ]
                } else {
                    vec![]
                }
            },
            Type::Primitive(PrimitiveType::U128) => {
                if let Value::Number(bigint) = &self.operation {
                    let u128_val = bigint.to_u128().expect("Invalid u128 value");
                    vec![
                        MOVE_ZERO_U128.to_owned(),
                        MOVE_MAX_U128.to_owned(),
                        u128_val.saturating_add(1).to_string(),
                        u128_val.saturating_sub(1).to_string(),
                    ]
                } else {
                    vec![]
                }
            },
            Type::Primitive(PrimitiveType::U256) => {
                vec![MOVE_ZERO_U256.to_owned(), MOVE_MAX_U256.to_owned()]
            },
            Type::Primitive(PrimitiveType::Num) => {
                vec![MOVE_ZERO.to_owned(), MOVE_MAX_INFERRED_NUM.to_owned()]
            },
            _ => vec![],
        };

        ops.into_iter()
            .filter(|v| cur_op != *v)
            .map(|op| {
                let mut mutated_source = source.to_string();
                mutated_source.replace_range(start..end, op.as_str());
                MutantInfo::new(
                    mutated_source,
                    Mutation::new(
                        Range::new(start, end),
                        OPERATOR_NAME.to_string(),
                        cur_op.to_string(),
                        op.to_string(),
                    ),
                )
            })
            .collect()
    }

    fn get_file_id(&self) -> FileId {
        self.loc.file_id()
    }

    fn name(&self) -> String {
        OPERATOR_NAME.to_string()
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LiteralOperator({:?}, location: file id: {:?}, index start: {}, index stop: {})",
            self.operation,
            self.loc.file_id(),
            self.loc.span().start(),
            self.loc.span().end()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codespan::Files;

    #[test]
    fn test_apply_u8() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 2));

        let operator = Literal::new(
            Value::Number(51.into()),
            Type::Primitive(PrimitiveType::U8),
            loc,
        );
        let source = "51";
        let expected = vec![MOVE_ZERO_U8, MOVE_MAX_U8, "52", "50"];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_apply_u16() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 3));

        let operator = Literal::new(
            Value::Number(963.into()),
            Type::Primitive(PrimitiveType::U16),
            loc,
        );
        let source = "963";
        let expected = vec![MOVE_ZERO_U16, MOVE_MAX_U16, "964", "962"];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_apply_u32() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 7));

        let operator = Literal::new(
            Value::Number(1000963.into()),
            Type::Primitive(PrimitiveType::U32),
            loc,
        );
        let source = "1000963";
        let expected = vec![MOVE_ZERO_U32, MOVE_MAX_U32, "1000964", "1000962"];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_apply_u64() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 12));

        let operator = Literal::new(
            Value::Number(963251000963u128.into()),
            Type::Primitive(PrimitiveType::U64),
            loc,
        );
        let source = "963251000963";
        let expected = vec![MOVE_ZERO_U64, MOVE_MAX_U64, "963251000964", "963251000962"];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_apply_u128() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 15));

        let operator = Literal::new(
            Value::Number(123963251000963u128.into()),
            Type::Primitive(PrimitiveType::U128),
            loc,
        );
        let source = "123963251000963";
        let expected = vec![
            MOVE_ZERO_U128,
            MOVE_MAX_U128,
            "123963251000964",
            "123963251000962",
        ];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_apply_bool() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 4));

        let operator = Literal::new(Value::Bool(true), Type::Primitive(PrimitiveType::Bool), loc);
        let source = MOVE_TRUE;
        let expected = vec![MOVE_FALSE];
        let result = operator.apply(source);
        assert_eq!(result.len(), expected.len());
        for (i, r) in result.iter().enumerate() {
            assert_eq!(r.mutated_source, expected[i]);
        }
    }

    #[test]
    fn test_get_file_id() {
        let mut files = Files::new();
        let fid = files.add("test", "test");
        let loc = Loc::new(fid, codespan::Span::new(0, 4));

        let operator = Literal::new(Value::Bool(true), Type::Primitive(PrimitiveType::Bool), loc);
        assert_eq!(operator.get_file_id(), fid);
    }
}
