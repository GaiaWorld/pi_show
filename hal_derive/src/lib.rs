#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate hal_core;

use proc_macro::TokenStream;
use quote::ToTokens;

/// Custom derive macro for the `Component` trait.
///
/// ## Example
///
/// ```rust,ignore
/// extern crate map;
/// use map::VecMap;
///
/// #[derive(Component, Debug)]
/// #[storage(VecMap)] //  `VecMap` is a data structure for a storage component, This line is optional, defaults to `VecMap`
/// struct Pos(f32, f32, f32);
/// ```
#[proc_macro]
pub fn uniform_buffer(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_uniform_buffer(&ast);
    gen.into()
}

#[proc_macro]
pub fn program_paramter(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_program_paramter(&ast);
    gen.into()
}

#[proc_macro]
pub fn defines(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_defines(&ast);
    gen.into()
}

fn impl_uniform_buffer(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(s) => {
            let fields = &s.fields;
            match fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!("UniformBuffer must is a struct, and field have name!"),
            }
        },
        _ => panic!("UniformBuffer must is a struct, and field have name!"),
    };
    let count = syn::Index::from(fields.len());
    let fields_names = FieldNamesArray(fields);
    let set_value_match = SetValueMatch(fields);
    let get_value_match = GetValueMatch(fields);
    let ubo_new_params = UboParamsArray(fields);
    let ubo_idents = UniformFieldIdentsArray(fields);
    let attrs = &ast.attrs;

    quote! {
        #(#attrs)*
        pub struct #name {
            values: [UniformValue; #count],
        }

        impl #name {
            pub const FIELDS: [&'static str; #count] = [#fields_names];
            #[inline]
            pub fn new(#ubo_new_params) -> Self {
                Self{
                    values: [#ubo_idents],
                }
            }
        }

        impl UniformBuffer for #name {
            #[inline]
            fn get_layout(&self) -> &[&str] {
                &Self::FIELDS[..]
            }

            #[inline]
            fn get_values(&self) -> &[UniformValue] {
                &self.values[..]
            }

            fn get_value(&self, name: &str) -> Option<&UniformValue> {
                match name {
                    #get_value_match
                    _ => None,
                }
            }

            fn set_value(&mut self, name: &str, value: UniformValue) -> bool {
                match name {
                    #set_value_match
                    _ => return false,
                };
                true
            }
        }
    }
}

fn impl_program_paramter(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(s) => {
            let fields = &s.fields;
            match fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!("UniformBuffer must is a struct, and field have name!"),
            }
        },
        _ => panic!("UniformBuffer must is a struct, and field have name!"),
    };
    let (textrue_count, uniform_count) = textrue_and_uniform_count(fields);
    let uniform_fields_names = UniformFieldNamesArray(fields);
    let texture_fields_names = TextureFieldNamesArray(fields);
    let uniform_set_value_match = UniformSetValueMatch(fields);
    let texture_set_value_match = TextureSetValueMatch(fields);
    let uniform_get_value_match = UniformGetValueMatch(fields);
    let texture_get_value_match = TextureGetValueMatch(fields);
    let texture_null = TextureFieldNewNullArray(fields);
    let uniform_null = UniformFieldNewNullArray(fields);
    let attrs = &ast.attrs;

    quote! {
        #(#attrs)*
        pub struct #name {
            uniforms: [Share<dyn UniformBuffer>; #uniform_count],
            textures: [(Share<HalTexture>, Share<HalSampler>); #textrue_count],
        }

        impl #name {
            pub const FIELDS: [&'static str; #uniform_count] = [#uniform_fields_names];
            pub const TEXTURE_FIELDS: [&'static str; #textrue_count] = [#texture_fields_names];
        }

        impl std::default::Default for #name{
            fn default() -> Self {
                Self {
                    uniforms: [#uniform_null],
                    textures: [#texture_null],
                }
            }
        }

        impl ProgramParamter for #name {
            #[inline]
            fn get_layout(&self) -> &[&str] {
                &Self::FIELDS[..]
            }

            #[inline]
            fn get_texture_layout(&self) -> &[&str] {
                &Self::TEXTURE_FIELDS[..]
            }

            #[inline]
            fn get_values(&self) -> &[Share<dyn UniformBuffer>] {
                &self.uniforms[..]
            }

            #[inline]
            fn get_textures(&self) -> &[(Share<HalTexture>, Share<HalSampler>)] {
                &self.textures[..]
            }

            fn get_value(&mut self, name: &str) -> Option<&Share<dyn UniformBuffer>> {
                match name {
                    #uniform_get_value_match
                    _ => None,
                }
            }

            fn get_texture(&mut self, name: &str) -> Option<&(Share<HalTexture>, Share<HalSampler>)> {
                match name {
                    #texture_get_value_match
                    _ => None,
                }
            }

            fn set_value(&mut self, name: &str, value: Share<dyn UniformBuffer>) -> bool {
                match name {
                    #uniform_set_value_match
                    _ => return false,
                };
                true
            }

            fn set_texture(&mut self, name: &str, value: (Share<HalTexture>, Share<HalSampler>)) -> bool {
                match name {
                    #texture_set_value_match
                    _ => return false,
                };
                true
            }
        }
    }
}

fn impl_defines(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(s) => {
            let fields = &s.fields;
            match fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!("UniformBuffer must is a struct, and field have name!"),
            }
        },
        _ => panic!("UniformBuffer must is a struct, and field have name!"),
    };
    let count = syn::Index::from(fields.len());
    let add_match = DefinesAddMatch(fields);
    let del_match = DefinesDelMatch(fields);
    let attrs = &ast.attrs;

    quote! {
        #(#attrs)*
        #[derive(Default)]
        pub struct #name {
            values: [Option<&'static str>; #count],
            id: u32,
        }

        impl Defines for #name {
            fn add(&mut self, value: &'static str) -> Option<&'static str> {
                match value {
                    #add_match
                    _ => None,
                } 
            }

            fn remove(&mut self, value: &'static str) -> Option<&'static str> {
                match value {
                    #del_match
                    _ => None,
                }
            }

            fn list(&self) -> &[Option<&str>] {
                &self.values[..]
            }
            
            fn id(&self) -> u32 {
                self.id
            }
        }
    }
}

struct FieldNamesArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for FieldNamesArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            let field_name_str = v.ident.clone().unwrap().to_string();
            tokens.extend(quote! {#field_name_str,});
        }          
    }
}

struct SetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for SetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            let field_name_str = v.ident.clone().unwrap().to_string();
            let index = syn::Index::from(i);
            tokens.extend(quote! {
                #field_name_str => self.values[#index] = value,
            });
            i += 1;
        }          
    }
}

struct GetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for GetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            let field_name_str = v.ident.clone().unwrap().to_string();
            let index = syn::Index::from(i);
            tokens.extend(quote! {
                #field_name_str => Some(&self.values[#index]),
            });
            i += 1;
        }          
    }
}

struct UboParamsArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UboParamsArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            let field_name = &v.ident;
            tokens.extend(quote! {#field_name: UniformValue,});
        }          
    }
}

struct UniformFieldIdentsArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UniformFieldIdentsArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            let field_name = &v.ident;
            tokens.extend(quote! {#field_name,});
        }          
    }
}

struct UniformFieldNamesArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UniformFieldNamesArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            if !v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                tokens.extend(quote! {#field_name_str,});
            }
        }          
    }
}

struct TextureFieldNamesArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for TextureFieldNamesArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            if v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                tokens.extend(quote! {#field_name_str,});
            }
        }          
    }
}

struct TextureFieldNewNullArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for TextureFieldNewNullArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            if v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                tokens.extend(quote! {(Share::new(HalTexture(0, 0)), Share::new(HalSampler(0, 0))),});
            }
        }          
    }
}

struct UniformFieldNewNullArray<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UniformFieldNewNullArray<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for v in self.0.iter(){
            if !v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                tokens.extend(quote! {Share::new(NullUniformBuffer),});
            }
        }          
    }
}

struct UniformSetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UniformSetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            if !v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                let index = syn::Index::from(i);
                tokens.extend(quote! {
                    #field_name_str => self.uniforms[#index] = value,
                });
                i += 1;
            }
        }          
    }
}

struct TextureSetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for TextureSetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            if v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                let index = syn::Index::from(i);
                tokens.extend(quote! {
                    #field_name_str => self.textures[#index] = value,
                });
                i += 1;
            }
        }          
    }
}

struct UniformGetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for UniformGetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            if !v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                let index = syn::Index::from(i);
                tokens.extend(quote! {
                    #field_name_str => Some(&self.uniforms[#index]),
                });
                i += 1;
            }
        }          
    }
}

struct TextureGetValueMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for TextureGetValueMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            if v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
                let field_name_str = v.ident.clone().unwrap().to_string();
                let index = syn::Index::from(i);
                tokens.extend(quote! {
                    #field_name_str => Some(&self.textures[#index]),
                });
                i += 1;
            }
        }          
    }
}

struct DefinesAddMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for DefinesAddMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            let field_name_str = v.ident.clone().unwrap().to_string();
            let index = syn::Index::from(i);
            tokens.extend(quote! {
                #field_name_str => {
                    self.id += 1 << #index;
                    std::mem::replace(&mut self.values[#index], Some(#field_name_str))
                },
            });
            i += 1;
        }          
    }
}

struct DefinesDelMatch<'a>(&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>);
impl<'a> ToTokens for DefinesDelMatch<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut i = 0;
        for v in self.0.iter(){
            let field_name_str = v.ident.clone().unwrap().to_string();
            let index = syn::Index::from(i);
            tokens.extend(quote! {
                #field_name_str => {
                    self.id -= 1 << #index;
                    std::mem::replace(&mut self.values[#index], None)
                },
            });
            i += 1;
        }          
    }
}

fn textrue_and_uniform_count(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> (syn::Index, syn::Index) {
    let (mut textrue_count, mut uniform_count) = (0, 0);
    for v in fields.iter() {
        if v.ty.clone().into_token_stream().to_string().contains("HalTexture"){
            textrue_count += 1;
        }else {
            uniform_count += 1;
        }
    }
    (syn::Index::from(textrue_count), syn::Index::from(uniform_count))
}
