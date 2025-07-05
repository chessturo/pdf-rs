use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pdf);

mod ast;
mod lexer;
mod parser_helper;
#[cfg(test)]
mod parser_tests;
