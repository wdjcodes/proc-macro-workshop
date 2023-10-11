use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Ident, Type, Fields, PathArguments, GenericArgument, parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let name = derive_input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site());

    let fields = match derive_input.data {
        syn::Data::Struct(sd) => sd.fields,
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    };
    

    let fields = parse_fields(fields);

    let names: Vec<&Ident> = fields.iter().map(|f| &f.name).collect();
    let required= fields.iter().filter(|f| !f.optional);
    let required_idents: Vec<&Ident> = required.clone().map(|f| &f.name).collect();
    let required_names: Vec<String> = required.map(|f| f.name.to_string()).collect();
    let optional_idents: Vec<&Ident> = fields.iter().filter(|f| f.optional).map(|f| &f.name).collect();
    let types: Vec<&Type> = fields.iter().map(|f| &f.ty).collect();
    
    let output = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#names: None,)*
                }
            }
        }

        pub struct #builder_name {
            #(#names: Option<#types>,)*
        }

        impl #builder_name {
            
            #(pub fn #names(&mut self, value: #types) -> &mut Self {
                self.#names = Some(value);
                self
            })*
            
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>>{
                #(if self.#required_idents == None {
                    return Err(format!("No value given for non optional field {}", #required_names).into());
                })*
                
                Ok(#name {
                    #(#required_idents: self.#required_idents.as_ref().unwrap().clone(),)*
                    #(#optional_idents: self.#optional_idents.clone(),)*
                })
            }
        }

    };
    output.into()
}

struct Field {
    name: Ident,
    optional: bool,
    ty: Type,
}


fn parse_fields(fields: Fields) -> Vec<Field> {
    let mut parsed = vec![];
    fields.iter().for_each(|f| {
        let field = if let Type::Path(path) = &f.ty {
            let last = path.path.segments.iter().last().unwrap();
            if last.ident == "Option" {
                let ty = if let PathArguments::AngleBracketed(args) = &last.arguments {
                    let arg = args.args.iter().next().unwrap();
                    if let GenericArgument::Type(ty) = arg {
                        ty.clone()
                    } else {
                        unimplemented!();
                    }
                } else {
                    unimplemented!();
                };
                Field {
                    name: f.ident.as_ref().unwrap().clone(),
                    optional: true,
                    ty: ty,
                }
            } else {
                Field {
                    name: f.ident.as_ref().unwrap().clone(),
                    optional: false,
                    ty: f.ty.clone(),
                }

            }
        } else {
            unimplemented!()
        };
        parsed.push(field);

        
    });

    parsed
}