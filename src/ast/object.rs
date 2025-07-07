//! AST nodes for PDF objects. See PDF Spec section 7.3

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Real(f64),
}
