use crate::gpu::GpuModule;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_quote_spanned, token, Field, FieldMutability, Fields, FieldsNamed, GenericParam,
    ItemImpl, ItemStruct, LitInt, Token, Visibility,
};

pub(crate) fn item_to_gpu(mut item: ItemStruct, module: &mut GpuModule) -> TokenStream {
    let span = item.span();
    let cpu_struct = cpu_struct(&item);
    let gpu_impl = gpu_impl(&item, &cpu_struct);
    let cpu_impl = cpu_impl(&item, &cpu_struct);
    match &mut item.fields {
        Fields::Unit => {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported empty struct"));
            return quote! {#item};
        }
        Fields::Unnamed(_) => {
            module
                .errors
                .push(syn::Error::new(item.span(), "unsupported tuple struct"));
            return quote! {#item};
        }
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                module
                    .errors
                    .push(syn::Error::new(item.span(), "unsupported empty struct"));
                return quote! {#item};
            }
            fields.named.push(Field {
                attrs: vec![],
                vis: value_visibility(span, fields),
                mutability: FieldMutability::None,
                ident: Some(Ident::new("__value", span)),
                colon_token: Some(Token![:](span)),
                ty: parse_quote_spanned! { span => ::ragna::GpuValue },
            });
        }
    };
    item.attrs
        .push(parse_quote_spanned! { span => #[derive(Clone, Copy)] });
    quote_spanned! {
        span =>
        #item
        #gpu_impl
        #cpu_struct
        #cpu_impl
    }
}

fn cpu_struct(item: &ItemStruct) -> ItemStruct {
    let gpu_ident = &item.ident;
    ItemStruct {
        attrs: item
            .attrs
            .iter()
            .filter(|attr| attr.path().segments[0].ident != "doc")
            .cloned()
            .chain([
                parse_quote_spanned! { item.span() => #[allow(unused)] },
                parse_quote_spanned! { item.span() => #[doc = concat!(
                    "The CPU type corresponding to [",
                    stringify!(#gpu_ident),
                    "] GPU type.")
                ] },
            ])
            .collect(),
        vis: item.vis.clone(),
        struct_token: item.struct_token,
        ident: Ident::new(&format!("{}Cpu", item.ident), item.ident.span()),
        generics: item.generics.clone(),
        fields: Fields::Named(FieldsNamed {
            brace_token: token::Brace(item.span()),
            named: item
                .fields
                .iter()
                .map(|field| {
                    let ty = &field.ty;
                    Field {
                        attrs: field.attrs.clone(),
                        vis: field.vis.clone(),
                        mutability: field.mutability.clone(),
                        ident: field.ident.clone(),
                        colon_token: field.colon_token,
                        ty: parse_quote_spanned! { ty.span() => <#ty as ::ragna::Gpu>::Cpu },
                    }
                })
                .collect(),
        }),
        semi_token: item.semi_token,
    }
}

fn value_visibility(span: Span, fields: &FieldsNamed) -> Visibility {
    let restriction = fields
        .named
        .iter()
        .map(|field| match &field.vis {
            Visibility::Public(_) => VisibilityRestriction::Pub,
            Visibility::Restricted(_) => VisibilityRestriction::Restricted,
            Visibility::Inherited => VisibilityRestriction::Inherited,
        })
        .min()
        .expect("internal error: no field in struct");
    match restriction {
        VisibilityRestriction::Inherited => parse_quote_spanned! { span => },
        VisibilityRestriction::Restricted => parse_quote_spanned! { span => pub(crate) },
        VisibilityRestriction::Pub => parse_quote_spanned! { span => pub },
    }
}

fn gpu_impl(gpu_struct: &ItemStruct, cpu_struct: &ItemStruct) -> ItemImpl {
    let gpu_ident = &gpu_struct.ident;
    let cpu_ident = &cpu_struct.ident;
    let (impl_generics, type_generics, where_clause) = gpu_struct.generics.split_for_impl();
    let field_idents = gpu_struct.fields.iter().flat_map(|field| &field.ident);
    let field_types: Vec<_> = gpu_struct.fields.iter().map(|field| &field.ty).collect();
    let field_index = gpu_struct
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| LitInt::new(&index.to_string(), field.span()));
    let generic_params = gpu_struct.generics.params.iter().map(|param| match param {
        GenericParam::Lifetime(param) => &param.lifetime.ident,
        GenericParam::Type(param) => &param.ident,
        GenericParam::Const(param) => &param.ident,
    });
    parse_quote_spanned! {
        gpu_struct.span() =>
        impl #impl_generics ::ragna::Gpu for #gpu_ident #type_generics #where_clause {
            type Cpu = #cpu_ident<#(#generic_params),*>;

            fn details() -> ::ragna::GpuTypeDetails {
                ::ragna::GpuTypeDetails::new_struct::<Self>(
                    vec![#(<#field_types as ::ragna::Gpu>::details()),*]
                )
            }

            fn value(self) -> ::ragna::GpuValue {
                self.__value
            }

            fn from_value(value: ::ragna::GpuValue) -> Self {
                Self {
                    #(#field_idents: <#field_types as ::ragna::Gpu>::from_value(
                        value.field::<#field_types>(#field_index)
                    ),)*
                    __value: value,
                }
            }
        }
    }
}

fn cpu_impl(gpu_struct: &ItemStruct, cpu_struct: &ItemStruct) -> ItemImpl {
    let cpu_ident = &cpu_struct.ident;
    let gpu_ident = &gpu_struct.ident;
    let (impl_generics, type_generics, where_clause) = cpu_struct.generics.split_for_impl();
    let field_idents: Vec<_> = cpu_struct
        .fields
        .iter()
        .flat_map(|field| &field.ident)
        .collect();
    let field_types: Vec<_> = cpu_struct.fields.iter().map(|field| &field.ty).collect();
    let generic_params = cpu_struct.generics.params.iter().map(|param| match param {
        GenericParam::Lifetime(param) => &param.lifetime.ident,
        GenericParam::Type(param) => &param.ident,
        GenericParam::Const(param) => &param.ident,
    });
    let offset_indexes = (0..=gpu_struct.fields.len())
        .map(|index| LitInt::new(&index.to_string(), gpu_struct.span()));
    let field_indexes = (0..gpu_struct.fields.len())
        .map(|index| LitInt::new(&index.to_string(), gpu_struct.span()));
    let next_field_indexes = (1..=gpu_struct.fields.len())
        .map(|index| LitInt::new(&index.to_string(), gpu_struct.span()));
    parse_quote_spanned! {
        gpu_struct.span() =>
        impl #impl_generics ::ragna::Cpu for #cpu_ident #type_generics #where_clause {
            type Gpu = #gpu_ident<#(#generic_params),*>;

            #[allow(clippy::cast_possible_truncation)]
            fn from_gpu(bytes: &[u8]) -> Self {
                let field_offsets = [
                    #(<Self::Gpu as ::ragna::Gpu>::details().field_offset(#offset_indexes) as usize),*
                ];
                #cpu_ident {
                    #(#field_idents: <#field_types as ::ragna::Cpu>::from_gpu(
                        &bytes[field_offsets[#field_indexes]..field_offsets[#next_field_indexes]]
                    ),)*
                }
            }

            fn to_wgsl(&self) -> ::ragna::Wgsl {
                ::ragna::Wgsl::Constructor(::ragna::WgslConstructor {
                    type_id: ::std::any::TypeId::of::<Self::Gpu>(),
                    args: vec![#(<#field_types as ::ragna::Cpu>::to_wgsl(&self.#field_idents)),*],
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum VisibilityRestriction {
    Inherited,
    Restricted,
    Pub,
}
