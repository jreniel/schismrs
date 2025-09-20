// f90nmlrs/src/fortran_types/tests.rs

//! Tests for Fortran types module.

use super::parsing::{
    infer_fortran_type, parse_repeat_expression, parse_value_list, validate_parsed_value,
    ValueConstraints,
};
use super::*;

#[test]
fn test_integer_parsing() {
    assert_eq!(parse_integer("42").unwrap(), FortranValue::Integer(42));
    assert_eq!(parse_integer("-123").unwrap(), FortranValue::Integer(-123));
    assert_eq!(
        parse_integer("42_int64").unwrap(),
        FortranValue::Integer(42)
    );
    assert!(parse_integer("3.14").is_err());
}

#[test]
fn test_real_parsing() {
    assert_eq!(parse_real("3.14").unwrap(), FortranValue::Real(3.14));
    assert_eq!(parse_real("1.23e4").unwrap(), FortranValue::Real(1.23e4));
    assert_eq!(parse_real("1.23d4").unwrap(), FortranValue::Real(1.23e4));
    assert_eq!(parse_real("1.0_real64").unwrap(), FortranValue::Real(1.0));
    assert_eq!(
        parse_real("+inf").unwrap(),
        FortranValue::Real(f64::INFINITY)
    );
    assert_eq!(
        parse_real("-inf").unwrap(),
        FortranValue::Real(f64::NEG_INFINITY)
    );
    assert!(parse_real("nan").unwrap().as_real().unwrap().is_nan());
}

#[test]
fn test_complex_parsing() {
    let result = parse_complex("(1.0, 2.0)").unwrap();
    assert_eq!(result, FortranValue::Complex(1.0, 2.0));

    let result = parse_complex("(1.5e2, -3.7d-1)").unwrap();
    assert_eq!(result, FortranValue::Complex(150.0, -0.37));

    assert!(parse_complex("1.0, 2.0").is_err()); // Missing parentheses
    assert!(parse_complex("(1.0)").is_err()); // Only one component
}

#[test]
fn test_logical_parsing() {
    assert_eq!(
        parse_logical(".true.").unwrap(),
        FortranValue::Logical(true)
    );
    assert_eq!(parse_logical(".T.").unwrap(), FortranValue::Logical(true));
    assert_eq!(parse_logical("true").unwrap(), FortranValue::Logical(true));
    assert_eq!(parse_logical("T").unwrap(), FortranValue::Logical(true));

    assert_eq!(
        parse_logical(".false.").unwrap(),
        FortranValue::Logical(false)
    );
    assert_eq!(parse_logical(".F.").unwrap(), FortranValue::Logical(false));
    assert_eq!(
        parse_logical("false").unwrap(),
        FortranValue::Logical(false)
    );
    assert_eq!(parse_logical("F").unwrap(), FortranValue::Logical(false));

    // Test flexible parsing
    assert_eq!(
        parse_logical(".TRUE.").unwrap(),
        FortranValue::Logical(true)
    );
    assert_eq!(parse_logical(".t").unwrap(), FortranValue::Logical(true));
}

#[test]
fn test_character_parsing() {
    assert_eq!(
        parse_character("'hello'"),
        FortranValue::Character("hello".to_string())
    );
    assert_eq!(
        parse_character("\"world\""),
        FortranValue::Character("world".to_string())
    );
    assert_eq!(
        parse_character("'don''t'"),
        FortranValue::Character("don't".to_string())
    );
    assert_eq!(
        parse_character("\"say \"\"hello\"\"\""),
        FortranValue::Character("say \"hello\"".to_string())
    );
    assert_eq!(
        parse_character("unquoted"),
        FortranValue::Character("unquoted".to_string())
    );
}

#[test]
fn test_fortran_string_formatting() {
    let options = FormatOptions::default();

    assert_eq!(
        FortranValue::Integer(42).to_fortran_string_with_options(&options),
        "42"
    );
    assert_eq!(
        FortranValue::Real(3.14).to_fortran_string_with_options(&options),
        "3.14"
    );
    assert_eq!(
        FortranValue::Complex(1.0, 2.0).to_fortran_string_with_options(&options),
        "(1.0, 2.0)"
    );
    assert_eq!(
        FortranValue::Logical(true).to_fortran_string(false),
        ".true."
    );
    assert_eq!(
        FortranValue::Logical(true).to_fortran_string(true),
        ".TRUE."
    );
    assert_eq!(
        FortranValue::Character("test".to_string()).to_fortran_string_with_options(&options),
        "'test'"
    );
}

#[test]
fn test_advanced_formatting() {
    let mut options = FormatOptions::default();
    options.float_precision = Some(2);
    options.exponential_threshold = Some((1e-3, 1e6));

    let val = FortranValue::Real(0.0001);
    assert!(val.to_fortran_string_with_options(&options).contains('e'));

    options.use_fortran_double = true;
    let result = val.to_fortran_string_with_options(&options);
    assert!(result.contains('d'));

    // Test complex formatting
    options.complex_format = ComplexFormat::Mathematical;
    let complex_val = FortranValue::Complex(1.0, -2.0);
    let result = complex_val.to_fortran_string_with_options(&options);
    assert!(result.contains("1-2.00*i") || result.contains("1.00-2.00*i"));
}

#[test]
fn test_type_conversions() {
    let val = FortranValue::Integer(42);
    assert_eq!(val.as_integer().unwrap(), 42);
    assert_eq!(val.as_real().unwrap(), 42.0);
    assert_eq!(val.as_complex().unwrap(), (42.0, 0.0));

    let val = FortranValue::Real(3.14);
    assert_eq!(val.as_real().unwrap(), 3.14);
    assert!(val.as_integer().is_err());

    let val = FortranValue::Real(42.0);
    assert_eq!(val.as_integer().unwrap(), 42);

    // Test bounds checking
    let val = FortranValue::Real(1e20);
    assert!(val.as_integer().is_err());
}

#[test]
fn test_array_repeat_formatting() {
    let values = vec![
        FortranValue::Integer(1),
        FortranValue::Integer(2),
        FortranValue::Integer(2),
        FortranValue::Integer(2),
        FortranValue::Integer(3),
    ];

    let options = FormatOptions::default();
    let result = FortranValue::format_array_with_repeats(&values, &options);
    assert_eq!(result, "1, 3*2, 3");
}

#[test]
fn test_type_inference() {
    assert_eq!(infer_fortran_type("42"), "integer");
    assert_eq!(infer_fortran_type("3.14"), "real");
    assert_eq!(infer_fortran_type("(1.0, 2.0)"), "complex");
    assert_eq!(infer_fortran_type(".true."), "logical");
    assert_eq!(infer_fortran_type("'hello'"), "character");
    assert_eq!(infer_fortran_type("hello"), "character");
    assert_eq!(infer_fortran_type(""), "null");
}

#[test]
fn test_value_summaries() {
    let val = FortranValue::Integer(42);
    assert_eq!(val.summary(), "integer(42)");

    let val = FortranValue::Character("hello world this is a very long string".to_string());
    assert!(val.summary().contains("hello world this ")); // Fixed test expectation

    let val = FortranValue::Array(vec![FortranValue::Integer(1), FortranValue::Integer(2)]);
    assert_eq!(val.summary(), "array[2]");
}

#[test]
fn test_conversion_checking() {
    let int_val = FortranValue::Integer(42);
    assert!(int_val.can_convert_to("real"));
    assert!(int_val.can_convert_to("complex"));
    assert!(int_val.can_convert_to("integer"));
    assert!(!int_val.can_convert_to("logical"));

    let real_val = FortranValue::Real(3.14);
    assert!(!real_val.can_convert_to("integer"));
    assert!(real_val.can_convert_to("complex"));

    let real_int = FortranValue::Real(42.0);
    assert!(real_int.can_convert_to("integer"));
}

#[test]
fn test_from_conversions() {
    // Test basic type conversions
    assert_eq!(FortranValue::from(42i32), FortranValue::Integer(42));
    assert_eq!(FortranValue::from(42i64), FortranValue::Integer(42));
    assert_eq!(
        FortranValue::from(3.14f32),
        FortranValue::Real(3.14f32 as f64)
    );
    assert_eq!(FortranValue::from(3.14f64), FortranValue::Real(3.14));
    assert_eq!(FortranValue::from(true), FortranValue::Logical(true));
    assert_eq!(
        FortranValue::from("hello"),
        FortranValue::Character("hello".to_string())
    );
    assert_eq!(
        FortranValue::from("hello".to_string()),
        FortranValue::Character("hello".to_string())
    );
    assert_eq!(
        FortranValue::from((1.0, 2.0)),
        FortranValue::Complex(1.0, 2.0)
    );

    // Test array conversions
    let int_vec = vec![1i32, 2i32, 3i32];
    let expected = FortranValue::Array(vec![
        FortranValue::Integer(1),
        FortranValue::Integer(2),
        FortranValue::Integer(3),
    ]);
    assert_eq!(FortranValue::from(int_vec), expected);

    // Test Option conversions
    assert_eq!(FortranValue::from(Some(42i32)), FortranValue::Integer(42));
    assert_eq!(FortranValue::from(None::<i32>), FortranValue::Null);
}

#[test]
fn test_try_into_conversions() {
    use std::convert::TryInto;

    // Test successful conversions
    let int_val = FortranValue::Integer(42);
    let result: i32 = int_val.clone().try_into().unwrap();
    assert_eq!(result, 42);

    let real_val = FortranValue::Real(3.14);
    let result: f64 = real_val.try_into().unwrap();
    assert_eq!(result, 3.14);

    let logical_val = FortranValue::Logical(true);
    let result: bool = logical_val.try_into().unwrap();
    assert_eq!(result, true);

    let char_val = FortranValue::Character("hello".to_string());
    let result: String = char_val.try_into().unwrap();
    assert_eq!(result, "hello");

    let complex_val = FortranValue::Complex(1.0, 2.0);
    let result: (f64, f64) = complex_val.try_into().unwrap();
    assert_eq!(result, (1.0, 2.0));

    // Test failed conversions
    let char_val = FortranValue::Character("hello".to_string());
    let result: Result<i32, _> = char_val.try_into();
    assert!(result.is_err());
}

#[test]
fn test_parse_value_list() {
    // Test simple list
    let result = parse_value_list("1, 2, 3", Some("integer")).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], FortranValue::Integer(1));
    assert_eq!(result[1], FortranValue::Integer(2));
    assert_eq!(result[2], FortranValue::Integer(3));

    // Test list with null values
    let result = parse_value_list("1, , 3", Some("integer")).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], FortranValue::Integer(1));
    assert_eq!(result[1], FortranValue::Null);
    assert_eq!(result[2], FortranValue::Integer(3));

    // Test list with quoted strings containing commas
    let result = parse_value_list("'hello, world', 'test'", Some("character")).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0],
        FortranValue::Character("hello, world".to_string())
    );
    assert_eq!(result[1], FortranValue::Character("test".to_string()));

    // Test list with complex numbers
    let result = parse_value_list("(1.0, 2.0), (3.0, 4.0)", Some("complex")).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], FortranValue::Complex(1.0, 2.0));
    assert_eq!(result[1], FortranValue::Complex(3.0, 4.0));
}

#[test]
fn test_parse_repeat_expression() {
    // Test basic repeat
    let (count, value) = parse_repeat_expression("3*42").unwrap();
    assert_eq!(count, 3);
    assert_eq!(value, FortranValue::Integer(42));

    // Test repeat with logical
    let (count, value) = parse_repeat_expression("5*.true.").unwrap();
    assert_eq!(count, 5);
    assert_eq!(value, FortranValue::Logical(true));

    // Test no repeat (single value)
    let (count, value) = parse_repeat_expression("42").unwrap();
    assert_eq!(count, 1);
    assert_eq!(value, FortranValue::Integer(42));

    // Test repeat with null value
    let (count, value) = parse_repeat_expression("3*").unwrap();
    assert_eq!(count, 3);
    assert_eq!(value, FortranValue::Null);
}

#[test]
fn test_value_constraints() {
    let constraints = ValueConstraints::new()
        .with_integer_range(-100, 100)
        .with_real_range(0.0, 1.0)
        .with_max_string_length(10)
        .with_max_array_length(5);

    // Test valid values
    let valid_int = FortranValue::Integer(50);
    assert!(validate_parsed_value(&valid_int, &constraints).is_ok());

    let valid_real = FortranValue::Real(0.5);
    assert!(validate_parsed_value(&valid_real, &constraints).is_ok());

    let valid_string = FortranValue::Character("hello".to_string());
    assert!(validate_parsed_value(&valid_string, &constraints).is_ok());

    // Test invalid values
    let invalid_int = FortranValue::Integer(200);
    assert!(validate_parsed_value(&invalid_int, &constraints).is_err());

    let invalid_real = FortranValue::Real(2.0);
    assert!(validate_parsed_value(&invalid_real, &constraints).is_err());

    let invalid_string = FortranValue::Character("this string is too long".to_string());
    assert!(validate_parsed_value(&invalid_string, &constraints).is_err());

    let invalid_array = FortranValue::Array(vec![
        FortranValue::Integer(1),
        FortranValue::Integer(2),
        FortranValue::Integer(3),
        FortranValue::Integer(4),
        FortranValue::Integer(5),
        FortranValue::Integer(6),
    ]);
    assert!(validate_parsed_value(&invalid_array, &constraints).is_err());
}

#[test]
fn test_auto_type_detection() {
    // Test that parse_fortran_value correctly detects types
    assert_eq!(
        parse_fortran_value("42", None).unwrap(),
        FortranValue::Integer(42)
    );
    assert_eq!(
        parse_fortran_value("3.14", None).unwrap(),
        FortranValue::Real(3.14)
    );
    assert_eq!(
        parse_fortran_value(".true.", None).unwrap(),
        FortranValue::Logical(true)
    );
    assert_eq!(
        parse_fortran_value("(1.0, 2.0)", None).unwrap(),
        FortranValue::Complex(1.0, 2.0)
    );
    assert_eq!(
        parse_fortran_value("hello", None).unwrap(),
        FortranValue::Character("hello".to_string())
    );
}

#[test]
fn test_edge_cases() {
    // Test empty string
    assert_eq!(parse_fortran_value("", None).unwrap(), FortranValue::Null);
    assert_eq!(
        parse_fortran_value("   ", None).unwrap(),
        FortranValue::Null
    );

    // Test very large numbers
    let large_int = "9223372036854775807"; // i64::MAX
    assert!(parse_integer(large_int).is_ok());

    // Test special float values
    assert!(parse_real("inf").unwrap().as_real().unwrap().is_infinite());
    assert!(parse_real("nan").unwrap().as_real().unwrap().is_nan());

    // Test malformed complex numbers
    assert!(parse_complex("(1.0)").is_err());
    assert!(parse_complex("1.0, 2.0").is_err());
    assert!(parse_complex("(1.0, 2.0, 3.0)").is_err());
}

