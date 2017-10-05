use syntax::codemap::Spanned;
use syntax::ast::{Ident, ItemKind, FnDecl};
use syntax::ext::base::Annotatable;
use syntax::ext::quote::rt::Span;

#[derive(Debug)]
pub struct Function(Spanned<(Ident, FnDecl)>);

impl Function {
    pub fn from(annotated: &Annotatable) -> Result<Function, Span> {
        let item = annotated.clone().expect_item();

        match item.node {
            ItemKind::Fn(ref fn_decl, ..) => {
                Ok(Function(Spanned { node: (item.ident, fn_decl.clone().unwrap()), span: item.span }))
            },
            _ => Err(item.span),
        }
    }

    pub fn ident(&self) -> &Ident {
        &self.0.node.0
    }

    pub fn decl(&self) -> &FnDecl {
        &self.0.node.1
    }

    pub fn span(&self) -> Span {
        self.0.span
    }
}
