use crate::token::Token;
use crate::object::Object;
use crate::ast;

pub enum Node {
    Program(ast::Program),
    Statement(ast::Statement),
    Expression(ast::Expression),
}

pub fn eval(node: Node) -> Option<Object> {
    match node {
        Node::Program(x) => eval_statements(x),
        Node::Statement(x) => match x {
            ast::Statement::ExpressionStatement{ expr } => eval(Node::Expression(expr)),
            ast::Statement::ReturnStatement{ value } => {
                if let Some(x) = eval(Node::Expression(value)) {
                    Some(Object::Return(Box::new(x)))
                } else { None }
            }
            _ => None
        },
        Node::Expression(x) => {
            //println!("{:?}", x);
            match x {
            ast::Expression::IntegerLiteral(Token::Int(x)) => Some(Object::Integer(x)),
            ast::Expression::Boolean(Token::True) => Some(Object::Boolean(true)),
            ast::Expression::Boolean(Token::False) => Some(Object::Boolean(false)),
            ast::Expression::InfixExpression{ left, opr, right } => {
                let left = eval(Node::Expression(*left)).unwrap();
                let right = eval(Node::Expression(*right)).unwrap();
                Some(eval_infix_expression(left, opr, right))
            }
            ast::Expression::PrefixExpression{ opr, right } => {
                let right = eval(Node::Expression(*right));
                Some(eval_prefix_expression(opr, right.unwrap()))
            },
            ast::Expression::IfExpression{ condition, conseqence, alternative } => {
                eval_if_expression(*condition, conseqence, alternative)
            }
            _ => None
        }}
    }
}

fn eval_statements(stmts: ast::Program) -> Option<Object> {
    let mut result = None;
    for stmt in &*stmts {
        result = eval(Node::Statement(stmt.clone()));
        if let Some(Object::Return(x)) = result {
            return Some(*x)
        }
        if let Some(Object::Error(x)) = result {
            return Some(Object::Error(x))
        }
    }
    result
}

fn eval_if_expression(condition: ast::Expression, conseqence: ast::Program, alternative: Option<ast::Program>) -> Option<Object> {
    let cd = eval(Node::Expression(condition));
    match cd {
        Some(Object::Boolean(false)) | Some(Object::Null) => {
            if let Some(x) = alternative {
                eval(Node::Program(x))
            } else {
                Some(Object::Null)
            }
        },
        _ => eval(Node::Program(conseqence))
    }
}

fn eval_infix_expression(left: Object, opr: Token, right: Object) -> Object {
    if left.get_type() == "INTEGER" && right.get_type() == "INTEGER" {
        let x = if let Object::Integer(x) = left { x } else { 0 };
        let y = if let Object::Integer(x) = right { x } else { 0 };
        eval_integer_infix_expression(x, opr, y)
    } else if left.get_type() != right.get_type() {
        Object::Error(format!("type mismatch: {} {} {}", left.get_type(), opr, right.get_type()))
    } else if opr == Token::Equal {
        native_bool_to_boolean_object(left == right)
    } else if opr == Token::NotEqual {
        native_bool_to_boolean_object(left != right)
    } else {
        Object::Error(format!("unknown operator: {} {} {}", left.get_type(), opr, right.get_type()))
    }
}

fn eval_integer_infix_expression(left: i64, opr: Token, right: i64) -> Object {
    match opr {
        Token::Plus => Object::Integer(left + right),
        Token::Minus => Object::Integer(left - right),
        Token::Asterisk => Object::Integer(left * right),
        Token::Slash => Object::Integer(left / right),
        Token::Lt => native_bool_to_boolean_object(left < right),
        Token::Gt => native_bool_to_boolean_object(left > right),
        Token::Equal => native_bool_to_boolean_object(left == right),
        Token::NotEqual => native_bool_to_boolean_object(left != right),
        _ => Object::Error(format!("unknown operator: INTEGER {} INTEGER", opr))
    }
}

fn native_bool_to_boolean_object(b: bool) -> Object {
    match b {
        true => Object::Boolean(true),
        false => Object::Boolean(false),
    }
}

fn eval_prefix_expression(opr: Token, right: Object) -> Object {
    match opr {
        Token::Minus => eval_minus_prefix_iperator_expression(right),
        Token::Bang => eval_bang_operator_expression(right),
        _ => Object::Error(format!("unknown operator: {} {}", opr, right.get_type()))
    }
}

fn eval_minus_prefix_iperator_expression(right: Object) -> Object {
    match right {
        Object::Integer(x) => Object::Integer(-x),
        _ => Object::Error(format!("unknown operator: -{}", right.get_type()))
    }
}

fn eval_bang_operator_expression(right: Object) -> Object {
    match right {
        Object::Boolean(true) => Object::Boolean(false),
        Object::Boolean(false) => Object::Boolean(true),
        Object::Null => Object::Boolean(true),
        _ => Object::Boolean(false)
    }
}


#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::object::Object;
    use crate::evaluator;
    
    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for tt in tests {
            if let Some(evaluated) = test_eval(tt.0) {
                test_integer_object(evaluated, tt.1);
            } else { assert!(false) }
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let program = p.parse_program();

        return evaluator::eval(evaluator::Node::Program(program));
    }

    fn test_integer_object(obj: Object, expected: i64) {
        if let Object::Integer(x) = obj {
            assert_eq!(x, expected);
        } else { assert!(false); }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

        for tt in tests {
            if let Some(evaluated) = test_eval(tt.0) {
                test_boolean_object(evaluated, tt.1);
            } else { assert!(false) }
        }
    }

    fn test_boolean_object(obj: Object, expected: bool) {
        if let Object::Boolean(x) = obj {
            assert_eq!(x, expected);
        } else { assert!(false); }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];

        for tt in tests {
            if let Some(evaluated) = test_eval(tt.0) {
                test_boolean_object(evaluated, tt.1);
            } else { assert!(false) }
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Some(10)),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(10)),
            ("if (1 < 2) { 10 }", Some(10)),
            ("if (1 > 2) { 10 }", None),
            ("if (1 > 2) { 10 } else { 20 }", Some(20)),
            ("if (1 < 2) { 10 } else { 20 }", Some(10)),
        ];

        for tt in tests {
            if let Some(evaluated) = test_eval(tt.0) {
                if let Some(x) = tt.1 {
                    test_integer_object(evaluated, x);
                } else { test_null_object(evaluated); }
            } else { panic!("{:?}", tt) }
        }
    }

    fn test_null_object(obj: Object) {
        assert_eq!(obj, Object::Null);
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            (r#"if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }
            "#, 10)
        ];

        for tt in tests {
            if let Some(evaluated) = test_eval(tt.0) {
                if evaluated != Object::Null {
                    test_integer_object(evaluated, tt.1);
                } else {
                    test_null_object(evaluated);
                }
            } else { assert!(false) }
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            (
                "5 + true;",
                "type mismatch: INTEGER + BOOLEAN",
            ),
            (
                "5 + true; 5;",
                "type mismatch: INTEGER + BOOLEAN",
            ),
            (
                "-true;",
                "unknown operator: -BOOLEAN",
            ),
            (
                "true + false;",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                "5; true + false; 5;",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                r#"
    if (10 > 1) {
        if (10 > 1) {
            return true + false;
        }
        return 1;
    }
    "#,
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            //(
            //    "foobar",
            //    "identifier not found: foobar",
            //),
        ];

        for tt in tests {
            if let Some(Object::Error(x)) = test_eval(tt.0) {
                assert_eq!(&x, tt.1);
            } else { panic!("{:?}", tt) }
        }
    }
}
