use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Type, spanned::Spanned};
use quote::{quote, quote_spanned, format_ident};

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new($span, $msg).to_compile_error().into()
    };
}

enum ImplType {
    Iter,
    Single
}

// Returns impl type and the inner type
fn kind(ty: &Type) -> Result<(ImplType, &Type), TokenStream> {
    Ok(match ty {
        Type::Slice(ts) => {
            (ImplType::Iter, ts.elem.as_ref())
        }
        Type::Array(ta) => {
            (ImplType::Iter, ta.elem.as_ref())
        }
        Type::Reference(tr) => {
            // Recurse!
            kind(tr.elem.as_ref())?
        }
        Type::Path(_) => {
            (ImplType::Single, ty)
        }
        _ => return Err(error!(ty.span(), "Representation type is not a regular type, array, nor slice"))
    })
}

#[proc_macro_derive(Searchable, attributes(representation))]
pub fn searchable_derive(ts: TokenStream) -> TokenStream {
    let ast = syn::parse(ts).unwrap();
    searchable_impl(&ast)
}

fn searchable_impl(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // Find tagged field
    let field = match &ast.data {
        Data::Struct(struc) => {
            struc.fields.iter().find(|f| {
                f.attrs.iter().any(|a| a.path.is_ident("representation"))
            })
        },
        _ => return error!(ast.span(), "`Searchable` can only derive structs.")
    };

    let field = if let Some(field) = field {
        field
    } else {
        return error!(ast.span(), "No `#[representable]` annotated member.")
    };

    let ty_span = field.ty.span();

    // Determine impl type and get inner type of representation
    let (kind, ty) = match kind(&field.ty) {
        Ok(a) => a,
        Err(ts) => return ts
    };

    // Enforce that the representation type has to be `AsRef<str>`
    let assert_ident = format_ident!("{}AssertAsRefStr", name);
    let assert_as_ref_str = quote_spanned! {ty_span=>
        struct #assert_ident where #ty: AsRef<str>;
    };

    let ident = field.ident.as_ref().unwrap();
    let impl_ = match kind {
        ImplType::Iter => quote! {
            self.#ident
                    .iter()
                    .find(|repr| search.to_lowercase().starts_with(&repr.to_lowercase()))
                    .map(|repr| (self, repr.len()))
        },
        ImplType::Single => quote! {
            if search.starts_with(&self.#ident) {
                Some((self, self.#ident.len()))
            } else {
                None
            }
        }
    };

    let gen = quote! {
        #[allow(dead_code)]
        #assert_as_ref_str
        impl Searchable for #name {
            fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)> {
                #impl_
            }
        }
    };
    
    TokenStream::from(gen)
}