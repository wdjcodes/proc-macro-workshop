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

    let names = fields.iter().map(|f| &f.name);
    let required= fields.iter().filter(|f| !f.optional);
    let required_idents = required.clone().map(|f| &f.name);
    let required_names = required.map(|f| f.name.to_string());
    let types = fields.iter().map(|f| &f.ty);
    
    let output = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #builder_name {
            pub fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }

            pub fn args(&mut self, args: Vec<String>) -> &mut Self {
                self.args = Some(args);
                self
            }

            pub fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }

            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }
            
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>>{
                #(if self.#required_idents == None {
                    Err("No value given for non optional field {}", #required_names);
                })*;
                
                Ok(#name {
                    #(#names: #types,)*
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
        println!("{:?}", f.ident.as_ref().unwrap());
        let field = if let Type::Path(path) = &f.ty {
            path.path.segments.iter().for_each(|p| println!("\t{:?}", p.ident));
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