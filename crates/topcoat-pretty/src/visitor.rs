use std::collections::HashMap;

use syn::{parse::Parse, spanned::Spanned, visit::Visit};

use crate::{MacroSnippet, PrettyPrint, registry::MacroRegistry};

use super::{MARGIN, Macro, pretty_print_str};
