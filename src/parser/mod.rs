use pest::Parser;
use pest_derive::Parser;

use crate::ast::{self, Import, Module, Path};

#[derive(Parser)]
#[grammar = "./pest/language.pest"]
struct PestParser {}

pub fn parse_module<S: AsRef<str>>(txt: S) -> anyhow::Result<ast::Module> {
    let mut md = Module::new();

    let parser = PestParser::parse(Rule::module, txt.as_ref())?;
    let mut iter = parser.into_iter();
    let nxt = iter.next().unwrap().into_inner();
    for item in nxt {
        match item.as_rule() {
            Rule::import => {
                let mut inner = item.into_inner().into_iter();
                let import_kind = inner.next().unwrap();

                match import_kind.as_rule() {
                    Rule::c_import => {
                        let mut iter = import_kind.into_inner().into_iter();
                        let header = iter.next().unwrap();
                        let function = iter.next().unwrap();
                        let mut path = Path::default();
                        path.add_segment(header.as_str());
                        path.add_segment(function.as_str());
                        md.add_c_import(path);
                    }
                    Rule::niebo_import => {}
                    un => unreachable!("reached: {un:?}"),
                }
            }
            Rule::fn_declaration => {
                let inner = item.into_inner();
                println!("{inner:?}");
            }
            Rule::var_declaration => {}
            un => unreachable!("reached: {un:?}"),
        }
    }
    println!("{md:?}");

    return Ok(md);
}
