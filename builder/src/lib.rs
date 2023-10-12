use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Ident, Type, Fields, PathArguments, GenericArgument, parse_macro_input, DeriveInput, Expr, Lit, Attribute, TypePath};
use quote::quote;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let name = derive_input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site());

    let fields = match derive_input.data {
        syn::Data::Struct(sd) => sd.fields,
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    };
    

    let fields = match parse_fields(fields){
        Ok(f) => f,
        Err(e) => {return e;},
    };
    let required: Vec<&Field> = fields.iter().filter(|f| !f.optional && !f.repeated).collect();
    let required_names: Vec<&Ident> = required.iter().map(|f| &f.name).collect();
    let required_name_lits: Vec<String> = required_names.iter().map(|i| i.to_string()).collect();
    let required_types: Vec<&Type> = required.iter().map(|f| &f.ty).collect();
    let optional: Vec<&Field> = fields.iter().filter(|f| f.optional).collect();
    let optional_names: Vec<&Ident> = optional.iter().map(|f| &f.name).collect();
    let optional_types: Vec<&Type> = optional.iter().map(|f| &f.ty).collect();
    let repeated: Vec<&Field> = fields.iter().filter(|f| f.repeated).collect();
    let repeated_names: Vec<&Ident> = repeated.iter().map(|f| &f.name).collect();
    let repeated_types: Vec<&Type> = repeated.iter().map(|f| &f.ty).collect();
    let appender_names: Vec<&Ident> = repeated.iter().map(|f| f.appender.as_ref().unwrap()).collect();
    let singles: Vec<&Field> = fields.iter().filter(|f| !f.repeated).collect();
    let single_names: Vec<&Ident> = singles.iter().map(|f| &f.name).collect();
    let single_types: Vec<&Type> = singles.iter().map(|f| &f.ty).collect();
    let repeated_setters: Vec<&Field> = repeated.iter().map(|f| *f).filter(|f| if let Some(_) = f.setter {true} else {false}).collect();
    let repeated_setter_names: Vec<&Ident> = repeated_setters.iter().map(|f| &f.name).collect();
    let repeated_setter_types: Vec<&Type> = repeated_setters.iter().map(|f| &f.ty).collect();
    
    let output = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#required_names: None,)*
                    #(#optional_names: None,)*
                    #(#repeated_names: vec![],)*
                }
            }
        }

        pub struct #builder_name {
            #(#required_names: Option<#required_types>,)*
            #(#optional_names: Option<#optional_types>,)*
            #(#repeated_names: Vec<#repeated_types>,)*
        }

        impl #builder_name {
            
            #(pub fn #single_names(&mut self, value: #single_types) -> &mut Self {
                self.#single_names = Some(value);
                self
            })*

            #(pub fn #repeated_setter_names(&mut self, vec: Vec<#repeated_setter_types>) -> &mut Self {
                self.#repeated_setter_names = vec;
                self
            })*

            #(pub fn #appender_names(&mut self, value: #repeated_types) -> &mut Self {
                self.#repeated_names.push(value);
                self
            })*
            
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>>{
                #(if self.#required_names == None {
                    return Err(format!("No value given for non optional field {}", #required_name_lits).into());
                })*
                
                Ok(#name {
                    #(#required_names: self.#required_names.as_ref().unwrap().clone(),)*
                    #(#optional_names: self.#optional_names.clone(),)*
                    #(#repeated_names: self.#repeated_names.clone().into_iter().collect(),)*
                })
            }
        }

    };
    output.into()
}

struct Field {
    name: Ident,
    optional: bool,
    repeated: bool,
    ty: Type,
    appender: Option<Ident>,
    setter: Option<Ident>,
}


fn parse_fields(fields: Fields) -> Result<Vec<Field>, TokenStream> {
    let mut parsed = vec![];
    for f in fields {
        let field = if let Type::Path(path) = &f.ty {
            let name = f.ident.as_ref().unwrap().clone();
            let mut repeated = false;
            let mut appender = None;
            let mut ty= f.ty.clone();
            let optional;
            let mut setter = None;
            
            for a in f.attrs{
                if let Some(ident) = a.path().get_ident() {
                    match ident.to_string().as_str() {
                        "builder" => {
                            if let Some((attr, name))  = get_assignment(&a) {
                                match attr.to_string().as_str() {
                                    "each" => {
                                        appender = Some(name.clone());
                                        repeated = true;
                                    }
                                    _ => {
                                            return Err(syn::Error::new_spanned(a.meta, "expected `builder(each = \"...\")`").into_compile_error().into());
                                        }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            optional = is_option(path);
            if repeated || optional {
                ty = get_bracketed_type(path);
            }
            
            if !repeated || !appender.eq(&f.ident) {
                setter = Some(f.ident.as_ref().unwrap().clone());
            }
            Field {
                name, optional, ty, appender, repeated, setter,
            }
        } else {
            unimplemented!()
        };
        parsed.push(field);

        
    }

    Ok(parsed)
}

fn get_assignment(attr: &Attribute) -> Option<(Ident, Ident)> {
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Assign(assign) = expr {
        let left = if let Expr::Path(path) = assign.left.as_ref() {
            if let Some(ident) = path.path.get_ident() {
                ident.clone()
            } else {
                return None;
            }
        } else {
            return None;
        };

        let right: Ident = if let Expr::Lit(literal) = assign.right.as_ref() {
            if let Lit::Str(str) = &literal.lit {
                Ident::new(str.value().as_str(), Span::call_site())
            } else {
                return None;
            }
        } else {
            return None;
        };
        Some((left, right))
    } else {
        None
    }
}

fn is_option(type_path: &TypePath) -> bool {
    if let Some(last) = type_path.path.segments.iter().last() {
        if last.ident == "Option" {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn get_bracketed_type(type_path: &TypePath) -> Type {
    if let Some(last) = type_path.path.segments.iter().last() {
        return if let PathArguments::AngleBracketed(args) = &last.arguments {
            args.args.iter()
                .find_map(|a| {
                    if let GenericArgument::Type(ty) = a {
                        Some(ty.clone())
                    } else {
                        None
                    }
                }).expect("Could not determine bracketed type")
        } else {
            unimplemented!()
        };
    } else {
        unimplemented!()
    }
}