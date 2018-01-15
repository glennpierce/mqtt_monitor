extern crate meval;

use meval::{eval_str, eval_str_with_context, Expr, Error, Context, FuncEvalError, builtin};
use std::str::FromStr;

#[test]
fn test_eval() {
    assert_eq!(eval_str("2 + 3"), Ok(5.));
    println!("{}", eval_str("2 + 3").unwrap());
    assert_eq!(eval_str("2 + (3 + 4)"), Ok(9.0));
    assert_eq!(eval_str("-2^(4 - 3) * (3 + 4)"), Ok(-14.));
    assert_eq!(eval_str("a + 3"), Err(Error::UnknownVariable("a".into())));
    assert_eq!(eval_str("round(sin (pi) * cos(0))"), Ok(0.));
    assert_eq!(eval_str("round( sqrt(3^2 + 4^2)) "), Ok(5.));
    assert_eq!(eval_str("max(1.)"), Ok(1.));
    assert_eq!(eval_str("max(1., 2., -1)"), Ok(2.));
    assert_eq!(eval_str("min(1., 2., -1)"), Ok(-1.));
    assert_eq!(eval_str("sin(1.) + cos(2.)"),
                Ok((1f64).sin() + (2f64).cos()));
    assert_eq!(eval_str("10 % 9"), Ok(10f64 % 9f64));

    //println!("{}", eval_str("6.2 + 3,").unwrap());
}

#[test]
fn test_builtins() {
    assert_eq!(eval_str("atan2(1.,2.)"), Ok((1f64).atan2(2.)));
}

#[test]
fn test_eval_func_ctx() {
    use std::collections::{HashMap, BTreeMap};
    let y = 5.;
    assert_eq!(eval_str_with_context("phi(2.)", Context::new().func("phi", |x| x + y + 3.)),
                Ok(2. + y + 3.));
    assert_eq!(eval_str_with_context("phi(2., 3.)",
                                        Context::new().func2("phi", |x, y| x + y + 3.)),
                Ok(2. + 3. + 3.));
    assert_eq!(eval_str_with_context("phi(2., 3., 4.)",
                                        Context::new().func3("phi", |x, y, z| x + y * z)),
                Ok(2. + 3. * 4.));
    assert_eq!(eval_str_with_context("phi(2., 3.)",
                                        Context::new()
                                            .funcn("phi", |xs: &[f64]| xs[0] + xs[1], 2)),
                Ok(2. + 3.));
    let mut m = HashMap::new();
    m.insert("x", 2.);
    m.insert("y", 3.);
    assert_eq!(eval_str_with_context("x + y", &m), Ok(2. + 3.));
    assert_eq!(eval_str_with_context("x + z", m),
                Err(Error::UnknownVariable("z".into())));
    let mut m = BTreeMap::new();
    m.insert("x", 2.);
    m.insert("y", 3.);
    assert_eq!(eval_str_with_context("x + y", &m), Ok(2. + 3.));
    assert_eq!(eval_str_with_context("x + z", m),
                Err(Error::UnknownVariable("z".into())));
}

#[test]
fn test_bind() {
    let expr = Expr::from_str("x + 3").unwrap();
    let func = expr.clone().bind("x").unwrap();
    assert_eq!(func(1.), 4.);

    assert_eq!(expr.clone().bind("y").err(),
                Some(Error::UnknownVariable("x".into())));

    let ctx = (("x", 2.), builtin());
    let func = expr.bind_with_context(&ctx, "y").unwrap();
    assert_eq!(func(1.), 5.);

    let expr = Expr::from_str("x + y + 2.").unwrap();
    let func = expr.clone().bind2("x", "y").unwrap();
    assert_eq!(func(1., 2.), 5.);
    assert_eq!(expr.clone().bind2("z", "y").err(),
                Some(Error::UnknownVariable("x".into())));
    assert_eq!(expr.bind2("x", "z").err(),
                Some(Error::UnknownVariable("y".into())));

    let expr = Expr::from_str("x + y^2 + z^3").unwrap();
    let func = expr.clone().bind3("x", "y", "z").unwrap();
    assert_eq!(func(1., 2., 3.), 32.);

    let expr = Expr::from_str("sin(x)").unwrap();
    let func = expr.clone().bind("x").unwrap();
    assert_eq!(func(1.), (1f64).sin());

    let expr = Expr::from_str("sin(x,2)").unwrap();
    match expr.clone().bind("x") {
        Err(Error::Function(_, FuncEvalError::NumberArgs(1))) => {}
        _ => panic!("bind did not error"),
    }
    let expr = Expr::from_str("hey(x,2)").unwrap();
    match expr.clone().bind("x") {
        Err(Error::Function(_, FuncEvalError::UnknownFunction)) => {}
        _ => panic!("bind did not error"),
    }
}

#[test]
fn hash_context() {
    let y = 0.;
    {
        let z = 0.;

        let mut ctx = Context::new();
        ctx.var("x", 1.).func("f", |x| x + y).func("g", |x| x + z);
        ctx.func2("g", |x, y| x + y);
    }
}
