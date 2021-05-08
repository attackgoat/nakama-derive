#![allow(unused_imports)]
#![allow(unused_variables)]

use {
    proc_macro::TokenStream,
    proc_macro2::{Ident, Span},
    quote::quote,
    syn::{parse, DeriveInput, Visibility, ItemFn},
};

#[proc_macro_attribute]
pub fn module_initializer(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    // println!("item: \"{}\"", item.to_string());

    let ast: ItemFn = parse(item).unwrap();
    let name = &ast.sig.ident;
    let gen = quote! {
        static mut ALREADY_NK_INIT: std::sync::atomic::AtomicBool
            = std::sync::atomic::AtomicBool::new(false);

        #[no_mangle]
        pub unsafe extern "C" fn nkinit(
            ctx: nakama::sys::NkContext,
            logger: nakama::sys::NkLogger,
            db: nakama::sys::NkDb,
            nk: nakama::sys::NkModule,
            initializer: nakama::sys::NkInitializer,
        ) -> usize {
            // Make sure this function has not been called in this process before
            if unsafe { ALREADY_NK_INIT.swap(true, std::sync::atomic::Ordering::SeqCst) } {
                panic!("Nakama already initialized");
            }

            #name(
                ctx.into(),
                logger.into(),
                db.into(),
                nk.into(),
                initializer.into()
            );

            return 0;
        }

        #ast
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn rpc_callback(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    // println!("item: \"{}\"", item.to_string());

    let mut ast: ItemFn = parse(item).unwrap();
    let name = ast.sig.ident;
    ast.sig.ident = Ident::new("call", name.span());
    ast.vis = Visibility::Inherited;

    let gen = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #name(
            ctx: nakama::sys::NkContext,
            logger: nakama::sys::NkLogger,
            db: nakama::sys::NkDb,
            nk: nakama::sys::NkModule,
            payload: nakama::sys::NkString,
            out_payload: *mut *mut ::std::os::raw::c_char,
            out_error: *mut *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int {
            #ast

            match call(
                ctx.into(),
                logger.into(),
                db.into(),
                nk.into(),
                ::std::ffi::CStr::from_ptr(payload.p).to_str().unwrap(),
            ) {
                Err(err) => {
                    1
                },
                Ok(res) => {
                    0
                }
            }
        }
    };
    gen.into()
}
