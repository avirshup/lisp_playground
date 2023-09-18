use super::functions::BuiltinFnBuilder;
use super::special_forms::BuiltinSpecialBuilder;
use super::{functions, special_forms};
use crate::Scope;

pub fn builtins() -> Scope {
    let mut scope = Scope::new(None);

    // special forms
    special_forms::LambdaForm::register(&mut scope);
    special_forms::DefVarForm::register(&mut scope);
    special_forms::DefineForm::register(&mut scope);

    // functions
    functions::PrintFnBuilder::register(&mut scope);
    functions::AddFnBuilder::register(&mut scope);

    scope
}
