use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput, Ident};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let name = derive_input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site());

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

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                if self.executable == None || self.args == None 
                    || self.env == None || self.current_dir == None {
                        return Err("All fields must be set".to_string().into());
                }
                Ok(#name {
                    executable: self.executable.to_owned().unwrap(),
                    args: self.args.to_owned().unwrap(),
                    env: self.env.to_owned().unwrap(),
                    current_dir: self.current_dir.to_owned().unwrap(),
                })
            }
        }

    };
    output.into()
}
