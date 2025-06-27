use proc_macro2::Literal;
use quote::{ToTokens, quote};
use syn::{Expr, Lit, Type, parse_macro_input};

macro_rules! custom_compiler_error_msg {
    ($out: ident, $format: literal) => {
            let error_message = String::from($format);
            $out.extend::<proc_macro2::TokenStream>(quote! {  compile_error!(#error_message); });
        };
    ($out: ident, $format: literal, $($arg:expr),*) => {
            let error_message = format!($format, $($arg),*);
            $out.extend::<proc_macro2::TokenStream>(quote! {  compile_error!(#error_message); });
        };
}

#[proc_macro_attribute]
pub fn expand_struct(
    _attr: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut out = proc_macro2::TokenStream::new();
    let mut template_struct = parse_macro_input!(items as syn::ItemStruct);

    let fields = match &mut template_struct.fields {
        syn::Fields::Unit | syn::Fields::Unnamed(_) => {
            custom_compiler_error_msg!(out, "Wrong struct type.");
            return out.into();
        }
        syn::Fields::Named(fields) => fields,
    };

    let mut to_readme_table =
        String::from("| Key | Value Type | Description | Default |\n|-|-|-|-|");
    let mut to_file = String::new();
    let mut from_env_vars_code = proc_macro2::TokenStream::new();
    let mut from_cli_code = proc_macro2::TokenStream::new();

    for field in &mut fields.named {
        let mut field_key = String::new();
        let mut field_description = String::new();
        let mut field_default = String::new();
        let mut field_default_literal = Lit::Verbatim(Literal::character(' '));
        let field_type = field.ty.clone();

        let field_type_name = match type_to_string(&field.ty) {
            Ok(v) => v,
            Err(_) => {
                println!("{:#?}", field.ty.clone());
                custom_compiler_error_msg!(out, "Not covered field type.");
                return out.into();
            }
        };

        let field_ident = field.ident.clone().unwrap();

        for attributes in field.attrs.clone() {
            let data = match attributes.meta {
                syn::Meta::NameValue(v) => v,
                _ => continue,
            };
            let ident = match data.path.get_ident() {
                None => continue,
                Some(v) => v,
            };

            match ident.to_string().as_str() {
                "key" => match data.value {
                    Expr::Lit(v) => match v.lit {
                        Lit::Str(v) => field_key = v.value(),
                        _ => {
                            custom_compiler_error_msg!(out, "Invalid type for key input.");
                            return out.into();
                        }
                    },
                    _ => {
                        custom_compiler_error_msg!(out, "Invalid key input.");
                        return out.into();
                    }
                },
                "description" => match data.value {
                    Expr::Lit(v) => match v.lit {
                        Lit::Str(v) => field_description = v.value(),
                        _ => {
                            custom_compiler_error_msg!(out, "Invalid type for description input.");
                            return out.into();
                        }
                    },
                    _ => {
                        custom_compiler_error_msg!(out, "Invalid description input.");
                        return out.into();
                    }
                },
                "value" => match data.value {
                    Expr::Lit(v) => {
                        field_default_literal = v.lit;
                        match &field_default_literal {
                            Lit::Str(v) => field_default = v.value(),
                            Lit::Bool(v) => field_default = v.value().to_string(),
                            Lit::Int(v) => field_default = v.base10_digits().to_string(),
                            _ => {
                                custom_compiler_error_msg!(out, "Invalid type for value input.");
                                return out.into();
                            }
                        }
                    }
                    _ => {
                        custom_compiler_error_msg!(out, "Invalid value input.");
                        return out.into();
                    }
                },
                _ => continue,
            }
        }

        field.attrs.retain(|x| {
            let path = x.path().get_ident();
            match path {
                Some(v) => {
                    let v = v.to_string();
                    let v = v.as_str();
                    v != "key" && v != "description" && v != "value"
                }
                None => true,
            }
        });

        to_readme_table += &format!(
            "\n| {field_key} | {field_type_name} | {field_description} | {field_default} |"
        );
        to_file += &format!("{field_key}={field_default}\n");

        let method_for_parsing = match field_type_name.as_str() {
            "String" => quote! { .map_or(#field_default_literal.to_string(), |x| x.to_string()) },
            "&'static str" => quote! { .unwrap_or(#field_default_literal) },
            "u16" | "u32" | "u64" | "bool" => quote! { .map_or(#field_default_literal, |x| {
               x.parse::<#field_type>().unwrap_or(#field_default_literal)
            }) },
            _ => {
                custom_compiler_error_msg!(out, "No idea how to parse type {}.", field_type_name);
                return out.into();
            }
        };
        from_env_vars_code.extend(quote! {
            #field_ident: std::env::var(#field_key)#method_for_parsing,
        });

        let method_for_parsing = match field_type_name.as_str() {
            "String" => quote! { .to_string() },
            "&'static str" => quote! {},
            "u16" | "u32" | "u64" | "bool" => quote! { .parse::<#field_type>().unwrap() },
            _ => {
                custom_compiler_error_msg!(out, "No idea how to parse type {}.", field_type_name);
                return out.into();
            }
        };
        from_cli_code.extend(quote! {
            #field_key => self.#field_ident = val #method_for_parsing,
        });
    }

    out.extend::<proc_macro2::TokenStream>(template_struct.into_token_stream());
    out.extend(quote! {
        impl EnvSettings {
            fn generate_url(&mut self) {
                self.database_url = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database_name
                );
            }

            pub fn from_env_vars() -> Result<Self, anyhow::Error> {
                dotenvy::dotenv()?;
                let mut out = Self {
                    #from_env_vars_code
                };
                out.generate_url();
                Ok(out)
            }


            pub fn from_cli(&mut self) {
                let args: Vec<String> = std::env::args().collect();
                let args: Vec<&str> = args.iter().map(|v| v.as_str()).collect();
                for arg in args {
                    if !arg.chars().any(|x | x == '=') {
                        continue
                    }
                    let mut arg_split = arg.split('=');
                    let key = arg_split.next();
                    let val = arg_split.next();
                    if let (Some(key), Some(val)) = (key, val) {
                        match key {
                            #from_cli_code
                            _ =>continue
                        }
                    }
                }
                self.generate_url();
            }

            pub fn to_env_file(&self, file: &mut std::fs::File) -> Result<(), anyhow::Error> {
                use std::io::Write;

                let mut out = std::io::LineWriter::new(file);
                write!(out, #to_file)?;
                Ok(())
            }
        }

        mod test {
            #[test]
            fn generate_readme_table() {
                println!(#to_readme_table);
            }
        }
    });
    out.into()
}

fn type_to_string(x: &Type) -> Result<String, ()> {
    match x {
        Type::Path(v) => v
            .path
            .segments
            .last()
            .cloned()
            .map(|x| x.ident.to_string())
            .ok_or(()),
        Type::Reference(v) => {
            let underlying_type = type_to_string(&v.elem)?;
            Ok(match v.lifetime {
                None => format!("&{underlying_type}"),
                Some(ref v) => format!("&{v} {underlying_type}"),
            })
        }
        _ => Err(()),
    }
}
