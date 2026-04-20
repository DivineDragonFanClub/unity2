use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
mod util;

mod data_models {
    pub mod class_attrs;
    pub mod field;
    pub mod method;
}

pub(crate) use data_models::class_attrs::ClassAttrs;
pub(crate) use data_models::field::Field;
pub(crate) use data_models::method::{Method, Resolution};

type ParseResult<T> = Result<T, venial::Error>;

// Lifted from godot-rust, parses the input, runs the transform, converts errors to compile_error!
fn translate<F>(input: TokenStream, transform: F) -> TokenStream
where
    F: FnOnce(venial::Item) -> ParseResult<TokenStream2>,
{
    let input2 = TokenStream2::from(input);

    let result2 = venial::parse_item(input2)
        .and_then(transform)
        .unwrap_or_else(|e| e.to_compile_error());

    TokenStream::from(result2)
}

#[proc_macro_attribute]
pub fn class(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| class_inner(TokenStream2::from(attr), body))
}

#[proc_macro_attribute]
pub fn methods(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| methods_inner(TokenStream2::from(attr), body))
}

#[proc_macro_attribute]
pub fn enumeration(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| enum_inner(TokenStream2::from(attr), body))
}

#[proc_macro_attribute]
pub fn callback(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| callback_inner(TokenStream2::from(attr), body))
}

#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| hook_inner(TokenStream2::from(attr), body))
}

#[proc_macro_attribute]
pub fn from_offset(attr: TokenStream, item: TokenStream) -> TokenStream {
    translate(item, |body| from_offset_inner(TokenStream2::from(attr), body))
}

fn class_inner(attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    let class = match item {
        venial::Item::Struct(class) => class,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::class] can only be applied on Struct items",
            ));
        }
    };

    let class_ident = class.name.clone();
    let vis = class.vis_marker.clone();
    let class_attrs = ClassAttrs::parse(attr, class_ident.to_string(), &class.attributes)?;

    // Forward every outer attribute except #[parent(...)], which the macro consumes
    let passthrough_attrs: Vec<&venial::Attribute> = class
        .attributes
        .iter()
        .filter(|attr| !util::path_is_single(&attr.path, "parent"))
        .collect();

    let named_fields = match class.fields {
        venial::Fields::Named(named_fields) => named_fields.fields.inner,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::class] can only be used on Struct items with named fields",
            ));
        }
    };

    let fields = parse_fields(named_fields)?;
    let trait_ident = format_ident!("I{}", class_ident);

    // impl_generics goes on the left of every `impl`, type_generics goes on every type reference
    // A struct with type params needs a PhantomData<fn() -> (T, ...)> slot, variance-erased so
    // the wrapper stays Send + Sync regardless of T, at runtime ClassIdentity::class() calls
    // Class::make_generic with each T::class(), so the cached class is the runtime instantiation
    let (impl_generics, type_generics) = if let Some(gp) = class.generic_params.as_ref() {
        let inline = gp.as_inline_args();
        (quote! { #gp }, quote! { #inline })
    } else {
        (quote! {}, quote! {})
    };

    let type_param_idents: Vec<&proc_macro2::Ident> = class
        .generic_params
        .as_ref()
        .map(|gp| {
            gp.params
                .items()
                .filter(|p| p.tk_prefix.is_none())
                .map(|p| &p.name)
                .collect()
        })
        .unwrap_or_default();

    // Lifetimes and const generics have no IL2CPP analogue and would need special-casing
    if let Some(gp) = class.generic_params.as_ref() {
        for param in gp.params.items() {
            if param.tk_prefix.is_some() {
                return Err(venial::Error::new(
                    "#[unity2::class] only supports type generic parameters; \
                     lifetimes and const generics are not allowed here",
                ));
            }
        }
    }

    let (phantom_field_decl, phantom_init, class_lookup_body) = if type_param_idents.is_empty() {
        (
            quote! {},
            quote! {},
            quote! {
                ::unity2::Class::lookup(
                    <Self as ::unity2::ClassIdentity>::NAMESPACE,
                    <Self as ::unity2::ClassIdentity>::NAME,
                )
            },
        )
    } else {
        let type_args = &type_param_idents;
        (
            quote! { , ::core::marker::PhantomData<fn() -> (#(#type_args,)*)> },
            quote! { , ::core::marker::PhantomData },
            quote! {
                ::unity2::Class::lookup(
                    <Self as ::unity2::ClassIdentity>::NAMESPACE,
                    <Self as ::unity2::ClassIdentity>::NAME,
                )
                .make_generic(&[
                    #(<#type_args as ::unity2::ClassIdentity>::class()),*
                ])
                .unwrap_or_else(|| panic!(
                    "{}",
                    ::unity2::Il2CppError::FailedGenericInstantiation {
                        class: ::core::stringify!(#class_ident).to_string(),
                    }
                ))
            },
        )
    };

    let (parent_bound, parent_impls) = if class_attrs.parents.is_empty() {
        (quote! { ::unity2::SystemObject }, quote! {})
    } else {
        if !type_param_idents.is_empty() {
            return Err(venial::Error::new(
                "#[unity2::class] does not support `#[parent(...)]` on generic \
                 child classes yet; instantiate parent relationships manually",
            ));
        }

        // First parent entry becomes the direct supertrait bound on the field-accessor trait
        // The rest are older ancestors, we emit transitive trait and From bridges for each
        let direct = &class_attrs.parents[0];
        let direct_base = &direct.base;
        let direct_generics = &direct.generics;
        let direct_trait_ident = format_ident!("I{}", direct_base);

        let ancestor_impls = class_attrs.parents.iter().map(|p| {
            let base = &p.base;
            let generics = &p.generics;
            let trait_ident = format_ident!("I{}", base);
            quote! {
                // Required transitively, ITexture with supertrait IObject means impl ITexture for Texture2D
                // only type-checks if Texture2D also implements IObject
                impl #trait_ident #generics for #class_ident {}

                // Upcast, routes through FromIlInstance on the ancestor so each ancestor's
                // construction logic (PhantomData init for generic parents, plain Self(x) for
                // non-generic ones) is preserved
                impl ::core::convert::From<#class_ident> for #base #generics {
                    fn from(value: #class_ident) -> Self {
                        <Self as ::unity2::FromIlInstance>::from_il_instance(
                            <#class_ident as ::core::convert::Into<::unity2::IlInstance>>::into(value),
                        )
                    }
                }
            }
        });

        (
            quote! { #direct_trait_ident #direct_generics },
            quote! {
                #(#ancestor_impls)*
            },
        )
    };

    // Instance accessors land in IFoo with default-body implementations
    let instance_accessors = fields.iter().filter(|f| !f.is_static).map(|f| {
        let rust_name = &f.name;
        let ty = &f.ty;
        let il2cpp_name = &f.il2cpp_name;

        // Inherited fields keep their byte offset across the hierarchy, so one cached offset is correct for every Self
        let resolve_offset = quote! {
            static OFFSET: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();
            let __offset = *OFFSET.get_or_init(|| {
                let class = ::unity2::object_get_class(self);
                let field = ::unity2::class_get_field_from_name(class, #il2cpp_name);
                field.offset as usize
            });
        };

        let setter = if f.readonly {
            quote! {}
        } else {
            let setter_name = format_ident!("set_{}", rust_name);
            quote! {
                fn #setter_name(self, value: #ty) {
                    #resolve_offset
                    ::unity2::field_set_value_at_offset(self, __offset, value);
                }
            }
        };

        quote! {
            fn #rust_name(self) -> #ty {
                #resolve_offset
                ::unity2::field_get_value_at_offset(self, __offset)
            }

            #setter
        }
    });

    // Static accessors land as inherent associated functions, subclasses don't inherit them,
    // matching the common C# pattern of calling statics through the defining class name
    let static_fields: Vec<&Field> = fields.iter().filter(|f| f.is_static).collect();
    let static_accessors = static_fields.iter().map(|f| {
        let rust_name = &f.name;
        let ty = &f.ty;
        let il2cpp_name = &f.il2cpp_name;

        // Inside the generated impl block, Self already carries the class-plus-type-params path
        let resolve_offset = quote! {
            static OFFSET: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();
            let __offset = *OFFSET.get_or_init(|| {
                let __class = <Self as ::unity2::ClassIdentity>::class();
                let field = ::unity2::class_get_field_from_name(__class.raw(), #il2cpp_name);
                field.offset as usize
            });
        };

        let setter = if f.readonly {
            quote! {}
        } else {
            let setter_name = format_ident!("set_{}", rust_name);
            quote! {
                #vis fn #setter_name(value: #ty) {
                    #resolve_offset
                    ::unity2::static_field_set_value_at_offset(
                        <Self as ::unity2::ClassIdentity>::class(),
                        __offset,
                        value,
                    );
                }
            }
        };

        quote! {
            #vis fn #rust_name() -> #ty {
                #resolve_offset
                ::unity2::static_field_get_value_at_offset(
                    <Self as ::unity2::ClassIdentity>::class(),
                    __offset,
                )
            }

            #setter
        }
    });

    let statics_block = if static_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_generics #class_ident #type_generics {
                #(#static_accessors)*
            }
        }
    };

    let namespace_lit = class_attrs.namespace.as_deref().unwrap_or("");
    let class_name_lit = class_attrs.name.as_str();

    Ok(quote! {
        #(#passthrough_attrs)*
        #[repr(transparent)]
        #[derive(::core::clone::Clone, ::core::marker::Copy)]
        #vis struct #class_ident #impl_generics(::unity2::IlInstance #phantom_field_decl);

        impl #impl_generics #class_ident #type_generics {
            #[doc(hidden)]
            #[inline]
            pub fn __unity2_from_il_instance(instance: ::unity2::IlInstance) -> Self {
                Self(instance #phantom_init)
            }
        }

        impl #impl_generics ::core::convert::From<#class_ident #type_generics> for ::unity2::IlInstance {
            fn from(value: #class_ident #type_generics) -> Self {
                value.0
            }
        }

        impl #impl_generics ::core::convert::AsRef<::unity2::IlInstance> for #class_ident #type_generics {
            fn as_ref(&self) -> &::unity2::IlInstance {
                &self.0
            }
        }

        impl #impl_generics ::unity2::ClassIdentity for #class_ident #type_generics {
            const NAMESPACE: &'static str = #namespace_lit;
            const NAME: &'static str = #class_name_lit;

            fn class() -> ::unity2::Class {
                // Monomorphized per generic instantiation, each List<T> gets its own CACHE
                static CACHE: ::std::sync::OnceLock<::unity2::Class> =
                    ::std::sync::OnceLock::new();
                *CACHE.get_or_init(|| { #class_lookup_body })
            }
        }

        impl #impl_generics ::unity2::FromIlInstance for #class_ident #type_generics {
            #[inline]
            fn from_il_instance(instance: ::unity2::IlInstance) -> Self {
                Self::__unity2_from_il_instance(instance)
            }
        }

        #vis trait #trait_ident #impl_generics: #parent_bound {
            #(#instance_accessors)*
        }

        impl #impl_generics #trait_ident #type_generics for #class_ident #type_generics {}
        #parent_impls

        #statics_block
    })
}

fn parse_fields(
    named_fields: Vec<(venial::NamedField, proc_macro2::Punct)>,
) -> ParseResult<Vec<Field>> {
    let mut all_fields = Vec::with_capacity(named_fields.len());
    for (named_field, _) in named_fields {
        all_fields.push(Field::parse(&named_field)?);
    }
    Ok(all_fields)
}

fn enum_inner(attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    // Optional namespace and name pair, when both are given the generated impl includes a class()
    // method so reflection helpers can reach into IL2CPP's metadata, when absent the enum stays
    // a purely Rust-side type
    let (il2cpp_namespace, il2cpp_name) = if attr.is_empty() {
        (None, None)
    } else {
        let mut parser = util::KvParser::parse_args("enumeration", attr, proc_macro2::Span::call_site())?;
        let ns = parser.handle_literal("namespace", "string")?
            .map(|lit| data_models::field::unquote_string_literal(&lit.to_string()));
        let name = parser.handle_literal("name", "string")?
            .map(|lit| data_models::field::unquote_string_literal(&lit.to_string()));
        parser.finish()?;
        (ns, name)
    };

    let enum_item = match &item {
        venial::Item::Enum(e) => e,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::enumeration] can only be applied to enum items",
            ));
        }
    };

    // #[repr(<int type>)] drives from_value arg type and value() return type, explicit so the
    // Rust side can't silently drift from the IL2CPP enum's underlying discriminant type
    let repr_ident = {
        let mut found: Option<proc_macro2::Ident> = None;
        for attr in &enum_item.attributes {
            if !util::path_is_single(&attr.path, "repr") {
                continue;
            }
            for tt in attr.value.get_value_tokens() {
                if let proc_macro2::TokenTree::Ident(id) = tt {
                    let s = id.to_string();
                    if matches!(
                        s.as_str(),
                        "i8" | "i16" | "i32" | "i64" | "isize"
                            | "u8" | "u16" | "u32" | "u64" | "usize"
                    ) {
                        found = Some(id.clone());
                        break;
                    }
                }
            }
            if found.is_some() {
                break;
            }
        }
        match found {
            Some(id) => id,
            None => {
                return Err(venial::Error::new(
                    "#[unity2::enumeration] requires `#[repr(<int type>)]` with one of \
                     i8/i16/i32/i64/u8/u16/u32/u64 as the discriminant type",
                ));
            }
        }
    };

    let enum_name = &enum_item.name;

    // Reject variants with fields, generated from_value and VARIANTS and Display assume unit variants
    // with const discriminants, IL2CPP enums are pure discriminants anyway
    let mut variant_names: Vec<&proc_macro2::Ident> = Vec::new();
    for (variant, _) in enum_item.variants.inner.iter() {
        if !matches!(variant.fields, venial::Fields::Unit) {
            return Err(venial::Error::new(
                "#[unity2::enumeration] requires unit variants (no tuple or struct data)",
            ));
        }
        variant_names.push(&variant.name);
    }

    let variant_names_str: Vec<String> =
        variant_names.iter().map(|n| n.to_string()).collect();

    // `Self::Name as #repr` isn't valid in match patterns but IS a valid const expression in an `if`
    let from_value_arms = variant_names.iter().map(|name| {
        quote! {
            if v == Self::#name as #repr_ident {
                return ::core::option::Option::Some(Self::#name);
            }
        }
    });

    let display_arms = variant_names
        .iter()
        .zip(variant_names_str.iter())
        .map(|(name, name_str)| quote! { Self::#name => #name_str, });

    // With namespace and name we emit IL2CPP_NAMESPACE and IL2CPP_NAME constants and a cached class() lookup
    let identity_methods = match (&il2cpp_namespace, &il2cpp_name) {
        (Some(ns), Some(name)) => {
            let ns_lit = ns.as_str();
            let name_lit = name.as_str();
            quote! {
                pub const IL2CPP_NAMESPACE: &'static str = #ns_lit;
                pub const IL2CPP_NAME: &'static str = #name_lit;

                pub fn class() -> ::unity2::Class {
                    static CACHE: ::std::sync::OnceLock<::unity2::Class> =
                        ::std::sync::OnceLock::new();
                    *CACHE.get_or_init(|| ::unity2::Class::lookup(#ns_lit, #name_lit))
                }
            }
        }
        _ => quote! {},
    };

    Ok(quote! {
        #[derive(
            ::core::clone::Clone,
            ::core::marker::Copy,
            ::core::fmt::Debug,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
            ::core::hash::Hash,
        )]
        #item

        impl #enum_name {
            #identity_methods

            pub const VARIANTS: &'static [Self] = &[#(Self::#variant_names),*];

            // Returns None for unknown values, so a game patch that adds a variant doesn't UB
            pub const fn from_value(v: #repr_ident) -> ::core::option::Option<Self> {
                #(#from_value_arms)*
                ::core::option::Option::None
            }

            #[inline]
            pub const fn value(self) -> #repr_ident {
                self as #repr_ident
            }
        }

        impl ::core::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(match self {
                    #(#display_arms)*
                })
            }
        }
    })
}

fn callback_inner(attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    if !attr.is_empty() {
        return Err(venial::Error::new(
            "#[unity2::callback] takes no arguments",
        ));
    }

    let func = match item {
        venial::Item::Function(f) => f,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::callback] can only be applied to extern \"C\" fn items",
            ));
        }
    };

    let fn_name = &func.name;
    let helper_name = format_ident!("{}_method_info", fn_name);
    let static_name = format_ident!("__UNITY2_MI_{}", fn_name);

    // IL2CPP parameters_count excludes the implicit target receiver and the trailing MethodInfo* slot
    let typed_count = func
        .params
        .inner
        .iter()
        .filter(|(p, _)| matches!(p, venial::FnParam::Typed(_)))
        .count();
    let parameters_count: u8 = typed_count.saturating_sub(2).min(u8::MAX as usize) as u8;

    // Built at compile time, the resulting byte-array pointer is stable for the plugin's lifetime
    let fn_name_c_lit = {
        let s = fn_name.to_string();
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0u8)).collect();
        let lit = proc_macro2::Literal::byte_string(&bytes);
        quote! { #lit }
    };

    const METHOD_ATTRIBUTE_STATIC: u16 = 0x0010;

    // The MethodInfo lives as a plain `static` in the plugin's binary, no heap allocation, no
    // leak, no runtime init, every field is const-initializable and the record sits in .rodata
    // MethodInfo's unsafe impl Sync is honest here, the record's fields are immutable post-init
    // and all raw pointers are either 'static or stable-address fn pointers
    Ok(quote! {
        #func

        #[allow(non_upper_case_globals)]
        static #static_name: ::unity2::MethodInfo = ::unity2::MethodInfo {
            method_ptr: #fn_name as *mut u8,
            invoker_method: ::core::ptr::null(),
            name: (#fn_name_c_lit).as_ptr(),
            class: ::core::option::Option::None,
            return_type: ::core::ptr::null(),
            parameters: ::core::ptr::null(),
            info_or_definition: ::core::ptr::null(),
            generic_method_or_container: ::core::ptr::null(),
            token: 0,
            flags: #METHOD_ATTRIBUTE_STATIC,
            iflags: 0,
            slot: 0,
            parameters_count: #parameters_count,
            bitflags: 0,
        };

        #[allow(non_snake_case)]
        fn #helper_name() -> &'static ::unity2::MethodInfo {
            &#static_name
        }
    })
}

struct IlHookArgs {
    namespace: String,
    class: String,
    method: String,
    args_override: Option<usize>,
}

fn parse_hook_args(attr: TokenStream2, attr_name: &str) -> ParseResult<IlHookArgs> {
    let tokens: Vec<proc_macro2::TokenTree> = attr.into_iter().collect();
    let mut strings: Vec<String> = Vec::new();
    let mut args_override: Option<usize> = None;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            proc_macro2::TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if let Some(stripped) = s.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
                    strings.push(stripped.to_string());
                } else if let Ok(n) = s.parse::<usize>() {
                    if args_override.is_some() {
                        return Err(venial::Error::new(format!(
                            "#[unity2::{attr_name}] accepts at most one numeric args override"
                        )));
                    }
                    args_override = Some(n);
                } else {
                    return Err(venial::Error::new(format!(
                        "#[unity2::{attr_name}] expects string literals for namespace/class/method (got {s})"
                    )));
                }
                i += 1;
            }
            proc_macro2::TokenTree::Punct(p) if p.as_char() == ',' => {
                i += 1;
            }
            other => {
                return Err(venial::Error::new(format!(
                    "#[unity2::{attr_name}] unexpected token `{other}`; expected `(\"NS\", \"Class\", \"Method\")` or `(\"NS\", \"Class\", \"Method\", N)`"
                )));
            }
        }
    }

    if strings.len() != 3 {
        return Err(venial::Error::new(format!(
            "#[unity2::{attr_name}] requires exactly 3 string literals (namespace, class, method); got {}",
            strings.len()
        )));
    }

    let method = strings.pop().unwrap();
    let class = strings.pop().unwrap();
    let namespace = strings.pop().unwrap();

    Ok(IlHookArgs {
        namespace,
        class,
        method,
        args_override,
    })
}

fn infer_il_arg_count(func: &venial::Function) -> usize {
    let mut count = 0;
    for (param, _) in func.params.inner.iter() {
        match param {
            venial::FnParam::Receiver(_) => {}
            venial::FnParam::Typed(t) => {
                let name = t.name.to_string();
                if name == "this" || name == "method_info" || name.starts_with('_') {
                    continue;
                }
                count += 1;
            }
        }
    }
    count
}

fn hook_inner(attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    let func = match item {
        venial::Item::Function(f) => f,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::hook] can only be applied to function items",
            ));
        }
    };

    if func.body.is_none() {
        return Err(venial::Error::new(
            "#[unity2::hook] requires a function with a body; for a bare declaration use #[unity2::from_offset]",
        ));
    }

    let parsed = parse_hook_args(attr, "hook")?;
    let args_count = parsed.args_override.unwrap_or_else(|| infer_il_arg_count(&func));

    let fn_name = &func.name;
    let lookup_mod_ident = format_ident!("__unity2_hook_lookup_{}", fn_name);
    let namespace = &parsed.namespace;
    let class = &parsed.class;
    let method = &parsed.method;

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #lookup_mod_ident {
            static OFFSET: ::std::sync::LazyLock<::unity2::Il2CppResult<usize>> =
                ::std::sync::LazyLock::new(|| {
                    ::unity2::lookup::method_offset_by_name(#namespace, #class, #method, #args_count)
                });
            pub fn get_offset() -> usize {
                match &*OFFSET {
                    ::core::result::Result::Ok(o) => *o,
                    ::core::result::Result::Err(e) => panic!(
                        "#[unity2::hook({:?}, {:?}, {:?})] install failed: {}",
                        #namespace, #class, #method, e
                    ),
                }
            }
        }

        #[::skyline::hook(offset = #lookup_mod_ident::get_offset())]
        #func
    })
}

fn from_offset_inner(attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    let func = match item {
        venial::Item::Function(f) => f,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::from_offset] can only be applied to function declarations",
            ));
        }
    };

    if func.body.is_some() {
        return Err(venial::Error::new(
            "#[unity2::from_offset] requires a bare function declaration (no body)",
        ));
    }

    let parsed = parse_hook_args(attr, "from_offset")?;
    let args_count = parsed.args_override.unwrap_or_else(|| infer_il_arg_count(&func));

    let fn_name = &func.name;
    let lookup_mod_ident = format_ident!("__unity2_offset_lookup_{}", fn_name);
    let namespace = &parsed.namespace;
    let class = &parsed.class;
    let method = &parsed.method;

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #lookup_mod_ident {
            static OFFSET: ::std::sync::LazyLock<::unity2::Il2CppResult<usize>> =
                ::std::sync::LazyLock::new(|| {
                    ::unity2::lookup::method_offset_by_name(#namespace, #class, #method, #args_count)
                });
            pub fn get_offset() -> usize {
                match &*OFFSET {
                    ::core::result::Result::Ok(o) => *o,
                    ::core::result::Result::Err(e) => panic!(
                        "#[unity2::from_offset({:?}, {:?}, {:?})] lookup failed: {}",
                        #namespace, #class, #method, e
                    ),
                }
            }
        }

        #[::skyline::from_offset(#lookup_mod_ident::get_offset() as usize)]
        #func
    })
}

fn methods_inner(_attr: TokenStream2, item: venial::Item) -> ParseResult<TokenStream2> {
    let impl_block = match item {
        venial::Item::Impl(i) => i,
        _ => {
            return Err(venial::Error::new(
                "#[unity2::methods] can only be applied to inherent impl blocks",
            ));
        }
    };

    if impl_block.trait_ty.is_some() {
        return Err(venial::Error::new(
            "#[unity2::methods] does not support trait impls; use `impl Foo { ... }`",
        ));
    }

    let self_ty = impl_block.self_ty.clone();
    let self_ident = util::extract_typename(&self_ty)
        .map(|seg| seg.ident)
        .ok_or_else(|| {
            venial::Error::new("#[unity2::methods] requires `impl Foo` (single type ident)")
        })?;

    let mut methods = Vec::new();
    for member in &impl_block.body_items {
        match member {
            venial::ImplMember::AssocFunction(func) => {
                methods.push(Method::parse(func)?);
            }
            _ => {
                return Err(venial::Error::new(
                    "#[unity2::methods] only supports function items",
                ));
            }
        }
    }

    // Generic impl blocks route through a trait-based pattern so children inherit parent methods,
    // the ergonomic equivalent of C# static method inheritance (PersonData.UnsafeGet works because
    // it's declared on StructData<T>), offset and pattern and vtable resolutions are rejected,
    // they bind a fixed function pointer and can't carry the per-instantiation MethodInfo
    //
    // Emitted shape,
    //   pub trait I{Base}Methods<T: ClassIdentity>: ClassIdentity { fn foo(..) -> T { ... } }
    //   impl<T, __U: I{Base}<T> + ClassIdentity> I{Base}Methods<T> for __U {}
    if let Some(impl_generics) = impl_block.impl_generic_params.as_ref() {
        for m in &methods {
            if !matches!(m.resolution, Resolution::Name { .. }) {
                return Err(venial::Error::new(
                    "generic `impl<T> Foo<T>` blocks only support name-based method \
                     resolution (`#[method]` / `#[method(name = \"...\")]`); \
                     offset / pattern / vtable_index bind a fixed function pointer \
                     and can't carry the per-instantiation MethodInfo",
                ));
            }
        }

        // Skip lifetime and const generics, IL2CPP has no analogue
        let type_param_idents: Vec<&proc_macro2::Ident> = impl_generics
            .params
            .items()
            .filter(|p| p.tk_prefix.is_none())
            .map(|p| &p.name)
            .collect();
        if type_param_idents.is_empty() {
            return Err(venial::Error::new(
                "generic `#[unity2::methods]` requires at least one type parameter \
                 on the impl block; non-generic impls already use the static-offset \
                 path automatically",
            ));
        }

        let methods_trait_ident = format_ident!("I{}Methods", self_ident);
        let field_trait_ident = format_ident!("I{}", self_ident);

        let method_defaults = methods.iter().map(|m| build_generic_trait_default(m));

        return Ok(quote! {
            pub trait #methods_trait_ident<
                #(#type_param_idents: ::unity2::ClassIdentity),*
            >: ::unity2::ClassIdentity {
                #(#method_defaults)*
            }

            // Blanket, any type that implements the parent's field-accessor trait AND
            // ClassIdentity inherits the methods trait
            impl<
                #(#type_param_idents: ::unity2::ClassIdentity,)*
                __U: #field_trait_ident<#(#type_param_idents),*> + ::unity2::ClassIdentity
            > #methods_trait_ident<#(#type_param_idents),*> for __U
            {}
        });
    }

    let raw_module_ident = format_ident!("__{}_unity2_raw", self_ident);
    let trait_ident = format_ident!("I{}", self_ident);
    let methods_trait_ident = format_ident!("I{}Methods", self_ident);

    let raw_items = methods.iter().map(|m| {
        let name = &m.name;
        let lookup_mod_ident = format_ident!("__lookup_{}", name);

        // Build the raw extern fn attribute plus the optional sibling lookup submodule for
        // resolution kinds that need a runtime offset
        let (raw_attr, lookup_mod) = match &m.resolution {
            Resolution::Offset(lit) => (
                quote! { #[::skyline::from_offset(#lit)] },
                quote! {},
            ),
            Resolution::Pattern(s) => (
                quote! { #[::lazysimd::from_pattern(#s)] },
                quote! {},
            ),
            Resolution::Name { name: il_name, args } => {
                let args_count = args.unwrap_or(m.params.len());
                let il_name_lit = il_name.as_str();
                (
                    quote! { #[::skyline::from_offset(#lookup_mod_ident::get_offset() as usize)] },
                    quote! {
                        #[doc(hidden)]
                        #[allow(non_snake_case)]
                        pub mod #lookup_mod_ident {
                            use super::*;
                            static OFFSET: ::std::sync::LazyLock<::unity2::Il2CppResult<usize>> =
                                ::std::sync::LazyLock::new(|| {
                                    ::unity2::lookup::method_offset_by_name(
                                        <#self_ty as ::unity2::ClassIdentity>::NAMESPACE,
                                        <#self_ty as ::unity2::ClassIdentity>::NAME,
                                        #il_name_lit,
                                        #args_count,
                                    )
                                });
                            pub fn get_offset() -> usize {
                                match &*OFFSET {
                                    ::core::result::Result::Ok(o) => *o,
                                    ::core::result::Result::Err(e) => panic!(
                                        "#[unity2::methods] {}::{} lookup failed: {}",
                                        <#self_ty as ::unity2::ClassIdentity>::NAME,
                                        #il_name_lit,
                                        e
                                    ),
                                }
                            }
                        }
                    },
                )
            }
            Resolution::VtableIndex(idx) => (
                quote! { #[::skyline::from_offset(#lookup_mod_ident::get_offset() as usize)] },
                quote! {
                    #[doc(hidden)]
                    #[allow(non_snake_case)]
                    pub mod #lookup_mod_ident {
                        use super::*;
                        static OFFSET: ::std::sync::LazyLock<::unity2::Il2CppResult<usize>> =
                            ::std::sync::LazyLock::new(|| {
                                ::unity2::lookup::method_offset_by_vtable_index(
                                    <#self_ty as ::unity2::ClassIdentity>::NAMESPACE,
                                    <#self_ty as ::unity2::ClassIdentity>::NAME,
                                    #idx,
                                )
                            });
                        pub fn get_offset() -> usize {
                            match &*OFFSET {
                                ::core::result::Result::Ok(o) => *o,
                                ::core::result::Result::Err(e) => panic!(
                                    "#[unity2::methods] {}[vtable {}] lookup failed: {}",
                                    <#self_ty as ::unity2::ClassIdentity>::NAME,
                                    #idx,
                                    e
                                ),
                            }
                        }
                    }
                },
            ),
        };

        let typed_params = m.params.iter().map(|p| {
            let pname = &p.name;
            let pty = &p.ty;
            quote! { #pname: #pty }
        });
        let receiver = if m.is_static {
            quote! {}
        } else {
            quote! { this: #self_ty, }
        };
        let ret = match &m.return_ty {
            Some(t) => quote! { -> #t },
            None => quote! {},
        };
        quote! {
            #lookup_mod

            #raw_attr
            // Hygienic name for the implicit trailing MethodInfo* slot so users can declare
            // their own `method_info` params on methods taking *const MethodInfo
            pub fn #name(#receiver #(#typed_params,)* __unity2_method_info: ::unity2::OptionalMethod) #ret;
        }
    });

    let static_fns = methods
        .iter()
        .filter(|m| m.is_static)
        .map(|m| build_static_wrapper(m, &raw_module_ident));
    let static_block = if methods.iter().any(|m| m.is_static) {
        quote! {
            impl #self_ty {
                #(#static_fns)*
            }
        }
    } else {
        quote! {}
    };

    let instance_block = if methods.iter().any(|m| !m.is_static) {
        let instance_fns = methods
            .iter()
            .filter(|m| !m.is_static)
            .map(|m| build_instance_wrapper(m, &self_ty, &raw_module_ident));
        quote! {
            pub trait #methods_trait_ident: #trait_ident {
                #(#instance_fns)*
            }

            impl<__T: #trait_ident> #methods_trait_ident for __T {}
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case, non_camel_case_types)]
        mod #raw_module_ident {
            use super::*;
            #(#raw_items)*
        }

        #static_block
        #instance_block
    })
}

// Used to skip `impl Into<T>` coercion on params whose type contains a reference, anonymous
// lifetimes inside impl Trait aren't stable, and adding explicit lifetime params would force
// the wrapper signature to grow noise
fn type_has_reference(ty: &venial::TypeExpr) -> bool {
    ty.tokens.iter().any(|tt| match tt {
        proc_macro2::TokenTree::Punct(p) => p.as_char() == '&',
        _ => false,
    })
}

fn build_static_wrapper(m: &Method, raw_mod: &proc_macro2::Ident) -> TokenStream2 {
    let name = &m.name;
    let vis = &m.vis;
    let typed_params = m.params.iter().map(|p| {
        let pname = &p.name;
        let pty = &p.ty;
        if type_has_reference(pty) {
            quote! { #pname: #pty }
        } else {
            quote! { #pname: impl ::core::convert::Into<#pty> }
        }
    });
    let arg_exprs = m.params.iter().map(|p| {
        let pname = &p.name;
        if type_has_reference(&p.ty) {
            quote! { #pname }
        } else {
            quote! { ::core::convert::Into::into(#pname) }
        }
    });
    let ret = match &m.return_ty {
        Some(t) => quote! { -> #t },
        None => quote! {},
    };
    let call = quote! { #raw_mod::#name(#(#arg_exprs,)* ::core::option::Option::None) };

    if m.is_unsafe {
        quote! {
            #vis unsafe fn #name(#(#typed_params),*) #ret {
                #call
            }
        }
    } else {
        quote! {
            #vis fn #name(#(#typed_params),*) #ret {
                unsafe { #call }
            }
        }
    }
}

// Converts &T or &mut T to *const T or *mut T, same ABI, avoids higher-ranked lifetimes that
// break MethodSignature inference, the macro casts at the call site so the Rust-facing
// signature keeps the reference for ergonomics
fn reference_to_raw_pointer(ty: &venial::TypeExpr) -> (proc_macro2::TokenStream, bool) {
    let mut iter = ty.tokens.iter().peekable();
    let Some(proc_macro2::TokenTree::Punct(p)) = iter.peek() else {
        return (ty.to_token_stream(), false);
    };
    if p.as_char() != '&' {
        return (ty.to_token_stream(), false);
    }
    iter.next(); // consume &

    // Optional lifetime ('a), a single punct `'` followed by an ident
    if let Some(proc_macro2::TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == '\'' {
            iter.next();
            if matches!(iter.peek(), Some(proc_macro2::TokenTree::Ident(_))) {
                iter.next();
            }
        }
    }

    let is_mut = matches!(
        iter.peek(),
        Some(proc_macro2::TokenTree::Ident(id)) if *id == "mut"
    );
    if is_mut {
        iter.next();
    }

    let inner: proc_macro2::TokenStream = iter.cloned().collect();
    let ptr_kind = if is_mut {
        quote! { *mut }
    } else {
        quote! { *const }
    };
    (quote! { #ptr_kind #inner }, true)
}

// Default body routes through runtime Class::method::<Sig> lookup, the cache stores only
// (method_ptr as usize, &'static MethodInfo), neither leg mentions outer generics, so the
// static OnceLock is accepted, the fn(...) signature materializes only as a local binding
// at call time where outer generics are allowed, unlike the static-offset path this does
// NOT apply impl Into<T> coercion, the extern "C" fn we transmute to has fixed types
fn build_generic_trait_default(m: &Method) -> TokenStream2 {
    let name = &m.name;

    let il2cpp_name = match &m.resolution {
        Resolution::Name { name, .. } => name.clone(),
        _ => unreachable!("non-Name resolutions are rejected upstream for generic impls"),
    };
    let il2cpp_name_lit = il2cpp_name.as_str();

    // Rust-facing signature keeps &mut V for ergonomics, Sig and extern fn types use raw
    // pointers, same ABI, no higher-ranked lifetimes, reference params are cast at the call site
    let typed_params = m.params.iter().map(|p| {
        let pname = &p.name;
        let pty = &p.ty;
        quote! { #pname: #pty }
    });

    struct ParamInfo {
        name: proc_macro2::TokenStream,
        abi_ty: proc_macro2::TokenStream,
        is_ref: bool,
    }
    let param_info: Vec<ParamInfo> = m
        .params
        .iter()
        .map(|p| {
            let (abi_ty, is_ref) = reference_to_raw_pointer(&p.ty);
            let pname = &p.name;
            ParamInfo {
                name: quote! { #pname },
                abi_ty,
                is_ref,
            }
        })
        .collect();

    let abi_types: Vec<_> = param_info.iter().map(|p| p.abi_ty.clone()).collect();
    let call_exprs: Vec<proc_macro2::TokenStream> = param_info
        .iter()
        .map(|p| {
            if p.is_ref {
                let name = &p.name;
                let abi_ty = &p.abi_ty;
                quote! { #name as #abi_ty }
            } else {
                p.name.clone()
            }
        })
        .collect();

    let ret_ty = match &m.return_ty {
        Some(t) => quote! { #t },
        None => quote! { () },
    };
    let ret_clause = match &m.return_ty {
        Some(t) => quote! { -> #t },
        None => quote! {},
    };

    let sig = if m.is_static {
        quote! { fn(#(#abi_types),*) -> #ret_ty }
    } else {
        quote! { fn(Self, #(#abi_types),*) -> #ret_ty }
    };

    let extern_fn_ty = if m.is_static {
        quote! {
            extern "C" fn(
                #(#abi_types,)*
                ::core::option::Option<&'static ::unity2::MethodInfo>,
            ) -> #ret_ty
        }
    } else {
        quote! {
            extern "C" fn(
                Self,
                #(#abi_types,)*
                ::core::option::Option<&'static ::unity2::MethodInfo>,
            ) -> #ret_ty
        }
    };

    let receiver = if m.is_static {
        quote! {}
    } else {
        // The Self Sized bound keeps the trait object-safe when it has no instance methods
        quote! { self, }
    };
    let sized_bound = if m.is_static {
        quote! {}
    } else {
        quote! { where Self: ::core::marker::Sized }
    };
    let call_args = if m.is_static {
        quote! { #(#call_exprs,)* ::core::option::Option::Some(__info) }
    } else {
        quote! { self, #(#call_exprs,)* ::core::option::Option::Some(__info) }
    };

    let unsafe_kw = if m.is_unsafe { quote! { unsafe } } else { quote! {} };
    let missing_msg = format!(
        "unity2::methods: `{}` not found on this class or any ancestor",
        il2cpp_name,
    );

    quote! {
        #unsafe_kw fn #name(#receiver #(#typed_params),*) #ret_clause #sized_bound {
            static CACHE: ::std::sync::OnceLock<(
                usize,
                &'static ::unity2::MethodInfo,
            )> = ::std::sync::OnceLock::new();
            let (__ptr, __info) = *CACHE.get_or_init(|| {
                let __m = <Self as ::unity2::ClassIdentity>::class()
                    .method::<#sig>(#il2cpp_name_lit)
                    .expect(#missing_msg);
                (__m.raw_ptr() as usize, __m.info())
            });
            let __f: #extern_fn_ty = unsafe { ::std::mem::transmute(__ptr) };
            __f(#call_args)
        }
    }
}

fn build_instance_wrapper(
    m: &Method,
    self_ty: &venial::TypeExpr,
    raw_mod: &proc_macro2::Ident,
) -> TokenStream2 {
    let name = &m.name;
    let typed_params = m.params.iter().map(|p| {
        let pname = &p.name;
        let pty = &p.ty;
        if type_has_reference(pty) {
            quote! { #pname: #pty }
        } else {
            quote! { #pname: impl ::core::convert::Into<#pty> }
        }
    });
    let arg_exprs = m.params.iter().map(|p| {
        let pname = &p.name;
        if type_has_reference(&p.ty) {
            quote! { #pname }
        } else {
            quote! { ::core::convert::Into::into(#pname) }
        }
    });
    let ret = match &m.return_ty {
        Some(t) => quote! { -> #t },
        None => quote! {},
    };

    let body = quote! {
        let __receiver = #self_ty::__unity2_from_il_instance(
            <Self as ::unity2::SystemObject>::as_instance(self),
        );
        #raw_mod::#name(__receiver, #(#arg_exprs,)* ::core::option::Option::None)
    };

    if m.is_unsafe {
        quote! {
            unsafe fn #name(self, #(#typed_params),*) #ret {
                #body
            }
        }
    } else {
        quote! {
            fn #name(self, #(#typed_params),*) #ret {
                unsafe { #body }
            }
        }
    }
}
