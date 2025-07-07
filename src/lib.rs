use lalrpop_util::lalrpop_mod;

// NOTE! This parser expects the token stream backwards. See the comment in `pdf.lalrpop` for
// details.
lalrpop_mod!(pdf);

mod ast;
mod lexer;
mod parser_helper;
#[cfg(test)]
mod parser_tests;
