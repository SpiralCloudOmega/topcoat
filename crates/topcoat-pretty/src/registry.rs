use std::collections::HashMap;

use syn::parse::Parse;

use crate::{Lexer, Macro, MacroSnippet, PrettyPrint, Printer};

type PrettyPrintFn = fn(&MacroRegistry, &MacroSnippet) -> syn::Result<String>;

#[derive(Default)]
pub struct MacroRegistry {
    pretty_print_fns: HashMap<String, PrettyPrintFn>,
}

impl MacroRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn one<T>(name: impl Into<String>) -> Self
    where
        T: Parse + PrettyPrint,
    {
        let mut result = Self::default();
        result.register::<T>(name);
        result
    }

    pub fn register<T>(&mut self, name: impl Into<String>) -> &mut Self
    where
        T: Parse + PrettyPrint,
    {
        let name = name.into();
        let pretty_print_fn = |registry: &MacroRegistry, snippet: &MacroSnippet| {
            let ast: Macro<T> = syn::parse_str(snippet.source_text())?;
            let trivia = Lexer::new(snippet.source_text()).collect::<Vec<_>>();
            let mut printer = Printer::new(
                registry,
                &trivia,
                snippet.initial_space(),
                snippet.initial_indent(),
            );
            ast.pretty_print(&mut printer);
            Ok(printer.eof())
        };

        if self.pretty_print_fns.contains_key(&name) {
            panic!(
                "registered multiple pretty print macros under the name `{}`",
                &name
            );
        }

        self.pretty_print_fns.insert(name, pretty_print_fn);

        self
    }

    pub fn pretty_print_macro_snippet(
        &self,
        snippet: &MacroSnippet,
    ) -> Option<syn::Result<String>> {
        self.pretty_print_fns
            .get(snippet.name())
            .map(|pretty_print_fn| pretty_print_fn(self, snippet))
    }
}
