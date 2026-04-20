use proc_macro2::{Ident, Span, TokenStream, TokenTree};

use crate::ParseResult;
use crate::data_models::field::unquote_string_literal;
use crate::util::{KvParser, bail, path_is_single};

pub struct ClassAttrs {
    pub namespace: Option<String>,
    pub name: String,
    // First entry is the direct parent, rest are progressively older ancestors
    pub parents: Vec<ParentType>,
}

// base is the leading ident, generics is any trailing tokens (e.g. <PersonData>)
pub struct ParentType {
    pub base: Ident,
    pub generics: TokenStream,
}

// Walks the struct attributes for a single #[parent(...)] chain
pub fn parse_parent_attr(attributes: &[venial::Attribute]) -> ParseResult<Vec<ParentType>> {
    let mut found: Option<Vec<ParentType>> = None;

    for attr in attributes {
        if !path_is_single(&attr.path, "parent") {
            continue;
        }

        if found.is_some() {
            return bail!(attr, "only a single #[parent] attribute is allowed");
        }

        let tokens = attr.value.get_value_tokens();
        let mut entries: Vec<ParentType> = Vec::new();
        let mut current_base: Option<Ident> = None;
        let mut current_generics: TokenStream = TokenStream::new();
        let mut depth: i32 = 0;

        for tt in tokens.iter() {
            match tt {
                TokenTree::Punct(p) if p.as_char() == ',' && depth == 0 => {
                    let base = current_base.take().ok_or_else(|| {
                        venial::Error::new("#[parent(...)] entry is missing a type")
                    })?;
                    entries.push(ParentType {
                        base,
                        generics: std::mem::take(&mut current_generics),
                    });
                }
                TokenTree::Punct(p) if p.as_char() == '<' => {
                    depth += 1;
                    current_generics.extend(std::iter::once(tt.clone()));
                }
                TokenTree::Punct(p) if p.as_char() == '>' => {
                    depth -= 1;
                    current_generics.extend(std::iter::once(tt.clone()));
                }
                TokenTree::Ident(id) if current_base.is_none() => {
                    current_base = Some(id.clone());
                }
                _ => {
                    if current_base.is_none() {
                        return bail!(
                            tt,
                            "#[parent(...)] entry must start with a Rust type ident",
                        );
                    }
                    current_generics.extend(std::iter::once(tt.clone()));
                }
            }
        }

        if let Some(base) = current_base {
            entries.push(ParentType {
                base,
                generics: current_generics,
            });
        }

        if entries.is_empty() {
            return bail!(attr, "#[parent(...)] expects at least one type");
        }

        found = Some(entries);
    }

    Ok(found.unwrap_or_default())
}

impl ClassAttrs {
    pub fn parse(
        attr: TokenStream,
        default_name: String,
        struct_attrs: &[venial::Attribute],
    ) -> ParseResult<Self> {
        let mut parser = KvParser::parse_args("class", attr, Span::call_site())?;

        let namespace = parser
            .handle_literal("namespace", "string")?
            .map(|lit| unquote_string_literal(&lit.to_string()));

        let name = parser
            .handle_literal("name", "string")?
            .map(|lit| unquote_string_literal(&lit.to_string()))
            .unwrap_or(default_name);

        parser.finish()?;

        let parents = parse_parent_attr(struct_attrs)?;

        Ok(Self {
            namespace,
            name,
            parents,
        })
    }
}
