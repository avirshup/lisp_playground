use super::functions::BuiltinFnBuilder;
use super::special_forms::BuiltinSpecialBuilder;
use super::{functions, special_forms};
use crate::Scope;

pub fn builtins() -> Scope {
    let mut scope = Scope::new(None);

    // special forms
    special_forms::QuoteFormBuilder::register(&mut scope);
    special_forms::LambdaFormBuilder::register(&mut scope);
    special_forms::DefVarForm::register(&mut scope);
    special_forms::DefineFormBuilder::register(&mut scope);

    // functions
    functions::IdentityFnBuilder::register(&mut scope);
    functions::PrintFnBuilder::register(&mut scope);
    functions::AddFnBuilder::register(&mut scope);
    functions::FirstFnBuilder::register(&mut scope);
    functions::RestFnBuilder::register(&mut scope);
    functions::ConcatFnBuilder::register(&mut scope);
    functions::RecordFnBuilder::register(&mut scope);

    scope
}
