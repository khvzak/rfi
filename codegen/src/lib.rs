#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use rustc_plugin::Registry;

use syntax::symbol::Symbol;
use syntax::ext::base::{SyntaxExtension, ExtCtxt, Annotatable};
use syntax::ext::quote::rt::{Span, ToTokens};

use syntax::ext::build::AstBuilder;
use syntax::ast::{MetaItem, Ident, TyKind};

use syntax::codemap::{Spanned, DUMMY_SP};

use syntax::tokenstream::TokenTree;
use syntax::parse::token;

mod utils;
use utils::Function;

mod generator;
use generator::Generator;

fn expand(ectx: &mut ExtCtxt, sp: Span, _: &MetaItem, annotated: Annotatable) -> Vec<Annotatable> {
    let mut output = vec![annotated.clone()];

    let user_fn = Function::from(&annotated).unwrap_or_else(|item_sp| {
        ectx.span_err(sp, "this attribute can be used only on functions...");
        ectx.span_fatal(item_sp, "...but was applied to the item below.");
    });

    let user_fn_ident = user_fn.ident();
    let wrapper_fn_ident = Ident::from_str(&format!("__rfi_sub_{}", user_fn_ident.name));

    let arg_types: Vec<TokenTree> = user_fn.decl().inputs.iter().map(|arg| &arg.ty).flat_map(|ty| {
        use TyKind::*;
        match ty.node {
            Path(..) => {
                let mut x = ty.clone().unwrap().to_tokens(ectx).clone();
                x.push(TokenTree::Token(DUMMY_SP, token::Comma));
                x
            },
            _ => {
                ectx.struct_span_fatal(ty.span, "unsupported input argument type")
                    .help("rfi functions can take only owned values implementing serde::(De)serialize trait")
                    .emit();
                vec![]
            },
        }
    }).collect();

    let mut user_fn_params: Vec<TokenTree> = (0 .. user_fn.decl().inputs.len()).flat_map(|i| {
        let mut x = ectx.expr_tup_field_access(sp, quote_expr!(ectx, args), i).to_tokens(ectx);
        x.push(TokenTree::Token(DUMMY_SP, token::Comma));
        x
    }).collect();
    user_fn_params.pop();

    let args_decode_stmt = if user_fn_params.len() > 0 {
        quote_stmt!(ectx,
            let args: ($arg_types) = serde_json::from_slice(::std::slice::from_raw_parts(buf, len)).expect("Cannot deserialize input arguments");
        ).unwrap()
    } else {
        quote_stmt!(ectx, let _ = ();).unwrap()
    };

    let wrapper_fn = quote_item!(ectx,
        #[no_mangle]
        #[allow(unused_variables)]
        pub unsafe extern fn $wrapper_fn_ident(buf: *const u8, len: usize, cb: extern fn(*const u8, usize)) {
            use ::rfi::serde_json;

            // Register "dummy" panic hook to avoid logging to stderr
            if ::std::env::var("RUST_BACKTRACE").is_err() {
                ::std::panic::set_hook(Box::new(|_| {}));
            }

            let result = ::std::panic::catch_unwind(move || {
                $args_decode_stmt
                let retval = $user_fn_ident($user_fn_params);
                serde_json::to_vec(&[retval]).expect("Cannot serialize output argument")
            });

            ::std::panic::take_hook();

            match result {
                Ok(retval_encoded) => {
                    cb(retval_encoded.as_ptr(), retval_encoded.len());
                },
                Err(err_any) => {
                    let mut err: Result<(), &str> = Err("panic");
                    if let Some(as_string) = err_any.downcast_ref::<String>() {
                        err = Err(as_string);
                    }
                    if let Some(as_str) = err_any.downcast_ref::<&str>() {
                        err = Err(as_str);
                    }
                    let err_encoded = serde_json::to_vec(&err).unwrap();
                    cb(err_encoded.as_ptr(), err_encoded.len());
                }
            }
        }
    ).unwrap();

    let module_path: Vec<String> = ectx.current_expansion.module.mod_path.iter().map(|x| x.name.as_str().to_string()).collect();

    Generator::add_func(
        &ectx.ecfg.crate_name,
        &module_path,
        &ectx.codemap().span_to_filename(sp),
        &user_fn_ident.name.as_str(),
        &wrapper_fn_ident.name.as_str(),
        user_fn.decl().inputs.len(),
    );

    output.push(Annotatable::Item(wrapper_fn));

    return output;
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    use std::io::ErrorKind;

    // Re-create `target/rfi` directory
    if let Some(err) = ::std::fs::remove_dir_all("./target/rfi").err() {
        match err.kind() {
            ErrorKind::NotFound => {},
            _ => panic!("Cannot clean `target/rfi` directory"),
        }
    }
    ::std::fs::create_dir_all("./target/rfi").expect("Cannot create `target/rfi` directory");

    reg.register_syntax_extension(Symbol::intern("rfi"), SyntaxExtension::MultiModifier(Box::new(expand)));
}
