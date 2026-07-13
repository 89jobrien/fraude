use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Fields, Lit, Meta, Token, Type, parse_macro_input,
    punctuated::Punctuated,
};

// ── ToolSpec derive ───────────────────────────────────────────────────────────

/// Derive `ToolSpec` for a tool input struct.
///
/// Generates `impl <Struct> { pub fn tool_spec() -> ::tools::ToolSpec { ... } }`.
///
/// Required attribute: `#[tool(name = "...", description = "...", permission = "...")]`
///
/// Permission values: `"ReadOnly"` | `"WorkspaceWrite"` | `"DangerFullAccess"`
///
/// Field types are mapped to JSON Schema:
/// - `String`         → `{ "type": "string" }`
/// - `bool`           → `{ "type": "boolean" }`
/// - `u32` / `i32` / `usize` / `i64` / `u64` → `{ "type": "integer" }`
/// - `f32` / `f64`    → `{ "type": "number" }`
/// - `Option<T>`      → `T`'s schema (field omitted from `required`)
/// - `Vec<T>`         → `{ "type": "array", "items": T's schema }`
/// - `Value` / `serde_json::Value` → `{}` (any)
///
/// Field doc comments become `"description"` in the schema.
///
/// # Example
///
/// ```ignore
/// #[derive(ToolSpec, serde::Deserialize)]
/// #[tool(name = "glob_search", description = "Find files by glob.", permission = "ReadOnly")]
/// pub struct GlobSearchInput {
///     /// Glob pattern, e.g. `**/*.rs`.
///     pub glob: String,
///     /// Root directory to search (defaults to cwd).
///     pub path: Option<String>,
/// }
/// ```
#[proc_macro_derive(ToolSpec, attributes(tool))]
pub fn derive_tool_spec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match tool_spec_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn build_field_schema(input: &DeriveInput) -> syn::Result<(Vec<TokenStream2>, Vec<String>)> {
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "#[derive(ToolSpec)] only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "#[derive(ToolSpec)] only supports structs",
            ));
        }
    };
    let mut properties: Vec<TokenStream2> = Vec::new();
    let mut required: Vec<String> = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().expect("named field").to_string();
        let field_doc = extract_doc_comment(&field.attrs);
        let (is_optional, inner_type) = unwrap_option(&field.ty);
        let schema = type_to_schema(inner_type, &field_doc);
        if !is_optional {
            required.push(field_name.clone());
        }
        properties.push(quote! { #field_name: #schema, });
    }
    Ok((properties, required))
}

fn tool_spec_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;

    // Parse #[tool(...)] attribute
    let tool_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("tool"))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &input.ident,
                "#[derive(ToolSpec)] requires a #[tool(name = \"...\", description = \"...\", permission = \"...\")] attribute",
            )
        })?;

    let mut tool_name: Option<String> = None;
    let mut tool_description: Option<String> = None;
    let mut tool_permission: Option<String> = None;

    tool_attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let value = meta.value()?;
            let lit: Lit = value.parse()?;
            if let Lit::Str(s) = lit {
                tool_name = Some(s.value());
            }
        } else if meta.path.is_ident("description") {
            let value = meta.value()?;
            let lit: Lit = value.parse()?;
            if let Lit::Str(s) = lit {
                tool_description = Some(s.value());
            }
        } else if meta.path.is_ident("permission") {
            let value = meta.value()?;
            let lit: Lit = value.parse()?;
            if let Lit::Str(s) = lit {
                tool_permission = Some(s.value());
            }
        }
        Ok(())
    })?;

    let name = tool_name
        .ok_or_else(|| syn::Error::new_spanned(tool_attr, "#[tool] requires name = \"...\""))?;
    let description = tool_description.ok_or_else(|| {
        syn::Error::new_spanned(tool_attr, "#[tool] requires description = \"...\"")
    })?;
    let permission_str = tool_permission.ok_or_else(|| {
        syn::Error::new_spanned(tool_attr, "#[tool] requires permission = \"...\"")
    })?;

    let permission = permission_ident(&permission_str).ok_or_else(|| {
        syn::Error::new_spanned(
            tool_attr,
            format!(
                "unknown permission \"{permission_str}\"; expected ReadOnly | WorkspaceWrite | DangerFullAccess"
            ),
        )
    })?;

    let (properties, required) = build_field_schema(input)?;
    let required_lit = required.iter().map(|s| quote! { #s });

    Ok(quote! {
        impl #struct_name {
            #[must_use]
            pub fn tool_spec() -> ToolSpec {
                ToolSpec {
                    name: #name,
                    description: #description,
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            #(#properties)*
                        },
                        "required": [#(#required_lit),*],
                        "additionalProperties": false
                    }),
                    required_permission: PermissionMode::#permission,
                }
            }
        }
    })
}

fn permission_ident(s: &str) -> Option<proc_macro2::Ident> {
    match s {
        "ReadOnly" | "WorkspaceWrite" | "DangerFullAccess" => {
            Some(proc_macro2::Ident::new(s, proc_macro2::Span::call_site()))
        }
        _ => None,
    }
}

fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| {
            if let Meta::NameValue(nv) = &a.meta
                && let Expr::Lit(expr_lit) = &nv.value
                && let Lit::Str(s) = &expr_lit.lit
            {
                return Some(s.value().trim().to_string());
            }
            None
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns `(is_optional, inner_type)` — unwraps `Option<T>` if present.
fn unwrap_option(ty: &Type) -> (bool, &Type) {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return (true, inner);
    }
    (false, ty)
}

/// Map a Rust type to a `serde_json::json!` schema fragment.
fn type_to_schema(ty: &Type, description: &str) -> TokenStream2 {
    let desc_field = if description.is_empty() {
        quote! {}
    } else {
        quote! { "description": #description, }
    };

    if let Type::Path(type_path) = ty {
        let last = type_path.path.segments.last();

        if let Some(seg) = last {
            match seg.ident.to_string().as_str() {
                "String" | "str" => {
                    return quote! { serde_json::json!({ #desc_field "type": "string" }) };
                }
                "bool" => {
                    return quote! { serde_json::json!({ #desc_field "type": "boolean" }) };
                }
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" => {
                    return quote! { serde_json::json!({ #desc_field "type": "integer" }) };
                }
                "f32" | "f64" => {
                    return quote! { serde_json::json!({ #desc_field "type": "number" }) };
                }
                "Value" => {
                    return quote! { serde_json::json!({ #desc_field }) };
                }
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        let item_schema = type_to_schema(inner, "");
                        return quote! {
                            serde_json::json!({
                                #desc_field
                                "type": "array",
                                "items": #item_schema
                            })
                        };
                    }
                }
                _ => {}
            }
        }
    }

    // Fallback: any
    quote! { serde_json::json!({ #desc_field }) }
}

// ── slash_command! declarative macro ─────────────────────────────────────────

/// Construct a `SlashCommandSpec` literal with less boilerplate.
///
/// # Example
///
/// ```ignore
/// use macros::slash_command;
///
/// const MY_CMD: SlashCommandSpec = slash_command! {
///     name: "mycommand",
///     aliases: ["mc"],
///     summary: "Does something useful.",
///     argument_hint: "<target>",
///     resume_supported: false,
///     category: Workspace,
/// };
/// ```
///
/// `argument_hint` and `aliases` are optional and default to `None` / `&[]`.
#[proc_macro]
pub fn slash(input: TokenStream) -> TokenStream {
    let input2 = TokenStream2::from(input);
    match slash_command_impl(input2) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct SlashCommandArgs {
    name: String,
    aliases: Vec<String>,
    summary: String,
    argument_hint: Option<String>,
    resume_supported: bool,
    category: String,
}

fn slash_command_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
    let args = parse_slash_command_args(input)?;

    let name = &args.name;
    let summary = &args.summary;
    let resume = args.resume_supported;
    let category = proc_macro2::Ident::new(&args.category, proc_macro2::Span::call_site());

    let aliases: Vec<_> = args.aliases.iter().map(|a| quote! { #a }).collect();

    let argument_hint = if let Some(hint) = &args.argument_hint {
        quote! { Some(#hint) }
    } else {
        quote! { None }
    };

    Ok(quote! {
        SlashCommandSpec {
            name: #name,
            aliases: &[#(#aliases),*],
            summary: #summary,
            argument_hint: #argument_hint,
            resume_supported: #resume,
            category: SlashCommandCategory::#category,
        }
    })
}

fn parse_slash_command_args(input: TokenStream2) -> syn::Result<SlashCommandArgs> {
    // Parse as a series of `key: value,` pairs
    let pairs = syn::parse::Parser::parse2(
        Punctuated::<syn::FieldValue, Token![,]>::parse_terminated,
        input,
    )?;

    let mut name: Option<String> = None;
    let mut aliases: Vec<String> = Vec::new();
    let mut summary: Option<String> = None;
    let mut argument_hint: Option<String> = None;
    let mut resume_supported = false;
    let mut category: Option<String> = None;

    for pair in &pairs {
        let key = match &pair.member {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(_) => continue,
        };

        match key.as_str() {
            "name" => {
                if let Expr::Lit(el) = &pair.expr
                    && let Lit::Str(s) = &el.lit
                {
                    name = Some(s.value());
                }
            }
            "summary" => {
                if let Expr::Lit(el) = &pair.expr
                    && let Lit::Str(s) = &el.lit
                {
                    summary = Some(s.value());
                }
            }
            "argument_hint" => {
                if let Expr::Lit(el) = &pair.expr
                    && let Lit::Str(s) = &el.lit
                {
                    argument_hint = Some(s.value());
                }
            }
            "resume_supported" => {
                if let Expr::Lit(el) = &pair.expr
                    && let Lit::Bool(b) = &el.lit
                {
                    resume_supported = b.value();
                }
            }
            "category" => {
                if let Expr::Path(ep) = &pair.expr
                    && let Some(seg) = ep.path.segments.last()
                {
                    category = Some(seg.ident.to_string());
                }
            }
            "aliases" => {
                if let Expr::Array(arr) = &pair.expr {
                    for elem in &arr.elems {
                        if let Expr::Lit(el) = elem
                            && let Lit::Str(s) = &el.lit
                        {
                            aliases.push(s.value());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(SlashCommandArgs {
        name: name.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "slash_command! requires `name: \"...\"`",
            )
        })?,
        aliases,
        summary: summary.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "slash_command! requires `summary: \"...\"`",
            )
        })?,
        argument_hint,
        resume_supported,
        category: category.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "slash_command! requires `category: <Category>`",
            )
        })?,
    })
}
