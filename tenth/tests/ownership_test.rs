use tenth::lexer::lexer::Lexer;
use tenth::parser::parser::Parser;
use tenth::hir::lower::Lowerer;
use tenth::runtime::interpreter::Interpreter;
use tenth::runtime::value::Value;

fn run_code(src: &str) -> Result<Option<Value>, String> {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| e.to_string())?;
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&program).map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new(&hir);
    interpreter.execute_program(&hir).map_err(|e| e.to_string())
}

#[test]
fn test_ref_and_deref() {
    let src = r#"
        let x = 42;
        let r = &x;
        *r
    "#;
    let result = run_code(src).unwrap();
    match result {
        Some(Value::Int(42)) => {},
        v => panic!("expected Int(42), got {:?}", v),
    }
}

#[test]
fn test_mut_ref_and_assign() {
    let src = r#"
        let mut y = 10;
        let m = &mut y;
        *m = 20;
        y
    "#;
    let result = run_code(src).unwrap();
    match result {
        Some(Value::Int(20)) => {},
        v => panic!("expected Int(20), got {:?}", v),
    }
}

#[test]
fn test_ref_to_struct_field() {
    let src = r#"
        struct Point { x: f64, y: f64 }
        let p = Point { x: 1.0, y: 2.0 };
        let r = &p;
        (*r).x
    "#;
    let result = run_code(src).unwrap();
    match result {
        Some(Value::Float(v)) => assert!((v - 1.0).abs() < 0.001),
        v => panic!("expected Float(1.0), got {:?}", v),
    }
}

#[test]
fn test_move_simple() {
    let src = r#"
        let x = 42;
        let y = move x;
        y
    "#;
    let result = run_code(src).unwrap();
    match result {
        Some(Value::Int(42)) => {},
        v => panic!("expected Int(42), got {:?}", v),
    }
}

#[test]
fn test_move_struct() {
    let src = r#"
        struct Point { x: f64, y: f64 }
        let p = Point { x: 1.0, y: 2.0 };
        let q = move p;
        q.x
    "#;
    let result = run_code(src).unwrap();
    match result {
        Some(Value::Float(v)) => assert!((v - 1.0).abs() < 0.001),
        v => panic!("expected Float(1.0), got {:?}", v),
    }
}

#[test]
fn test_move_source_unusable() {
    let src = r#"
        let x = 42;
        let y = move x;
        x
    "#;
    let result = run_code(src);
    assert!(result.is_err(), "expected error accessing moved variable, got {:?}", result);
}

#[test]
fn test_cannot_move_twice() {
    let src = r#"
        let x = 42;
        let y = move x;
        let z = move x;
        z
    "#;
    let result = run_code(src);
    assert!(result.is_err(), "expected error moving same variable twice, got {:?}", result);
}

#[test]
fn test_cannot_borrow_mut_while_shared() {
    let src = r#"
        let mut x = 42;
        let r = &x;
        let m = &mut x;
        *m
    "#;
    let result = run_code(src);
    assert!(result.is_err(), "expected error borrowing mut while shared, got {:?}", result);
}

#[test]
fn test_cannot_borrow_shared_while_mut_borrowed() {
    let src = r#"
        let mut x = 42;
        let m = &mut x;
        let r = &x;
        *m
    "#;
    let result = run_code(src);
    assert!(result.is_err(), "expected error borrowing shared while mut, got {:?}", result);
}

#[test]
fn test_cannot_move_while_borrowed() {
    let src = r#"
        let x = 42;
        let r = &x;
        let y = move x;
        *r
    "#;
    let result = run_code(src);
    assert!(result.is_err(), "expected error moving while borrowed, got {:?}", result);
}