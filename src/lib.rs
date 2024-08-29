use core::str;
use std::{
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
};

use annotate_snippets::{Level, Renderer, Snippet};
use anyhow::anyhow;
use proc_macro2::{Span, TokenStream};
use syn::{
    parse2, spanned::Spanned, visit::Visit, Attribute, Expr, ExprLit, File, ItemMod, Lit, Meta,
};

pub struct Source {
    path: PathBuf,
    text: String,
}

pub struct ExpandError {
    e: anyhow::Error,
    span: Option<Range<usize>>,
    source: Option<Source>,
}
impl ExpandError {
    fn new(span: Option<Span>, e: impl Into<anyhow::Error>) -> Self {
        let e = e.into();
        let span = span.map(|s| s.byte_range());
        let source = None;
        Self { e, span, source }
    }
    pub fn show(&self) {
        let title = self.e.to_string();
        let path;
        let mut m = Level::Error.title(&title);
        if let (Some(source), Some(span)) = (&self.source, self.span.clone()) {
            path = source.path.to_string_lossy();
            m = m.snippet(
                Snippet::source(&source.text)
                    .fold(true)
                    .origin(&path)
                    .annotation(Level::Error.span(span)),
            );
        }
        let renderer = Renderer::styled();
        eprintln!("{}", renderer.render(m));
    }
}

impl<E: Into<anyhow::Error>> From<E> for ExpandError {
    fn from(e: E) -> Self {
        let e = e.into();
        let span = e.downcast_ref::<syn::Error>().map(|e| e.span());
        Self::new(span, e)
    }
}
type Result<T> = std::result::Result<T, ExpandError>;

fn with_path<T>(r: io::Result<T>, path: &Path) -> Result<T> {
    r.map_err(|e| anyhow!("Could not read file : `{}` ({e})", path.display()).into())
}
fn with_source<T>(r: Result<T>, path: &Path, text: &str) -> Result<T> {
    r.map_err(|mut e| {
        if e.source.is_none() {
            let path = path.to_path_buf();
            let text = text.to_string();
            e.source = Some(Source { path, text });
        }
        e
    })
}

pub fn expand_from_path(path: &Path, is_root: bool) -> Result<String> {
    let s = with_path(fs::read_to_string(path), path)?;
    with_source(expand_from_text(path, is_root, &s), path, &s)
}
fn expand_from_text(path: &Path, is_root: bool, s: &str) -> Result<String> {
    let tokens = parse_token_stream(s)?;
    let file: File = parse2(tokens)?;
    let mut b = CodeBuilder::new();
    b.visit_file(&file);
    let mut text = String::new();
    for part in b.finish(s.len()) {
        match part {
            Part::Text(r) => text.push_str(&s[r]),
            Part::Mod(m) => {
                text.push_str(" {\n");
                text.push_str(&expand_from_path(
                    &path_from_mod(path, is_root, &m)?,
                    false,
                )?);
                text.push_str("}\n");
            }
        }
    }
    Ok(text)
}

fn parse_token_stream(s: &str) -> syn::Result<TokenStream> {
    match TokenStream::from_str(s) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(syn::Error::new(e.span(), e)),
    }
}

fn path_from_mod(path: &Path, is_root: bool, m: &ItemMod) -> Result<PathBuf> {
    match path_from_attrs(&m.attrs) {
        Some(p) => Ok(path.parent().unwrap().join(p)),
        None => {
            let name = m.ident.to_string();
            let file_name = path.file_name().unwrap();
            let base = if is_root || file_name == "mod.rs" {
                path.parent().unwrap().to_path_buf()
            } else {
                path.with_extension("")
            };
            let p0 = base.join(format!("{name}.rs"));
            if p0.is_file() {
                return Ok(p0);
            }
            let p1 = base.join(format!("{name}/mod.rs"));
            if p1.is_file() {
                return Ok(p1);
            }
            Err(ExpandError::new(
                Some(m.span()),
                anyhow!("Could not find source file : `{}`", p0.display()),
            ))
        }
    }
}

fn path_from_attrs(attr: &[Attribute]) -> Option<PathBuf> {
    for attr in attr {
        if let Some(p) = path_from_attr(attr) {
            return Some(p);
        }
    }
    None
}

fn path_from_attr(attr: &Attribute) -> Option<PathBuf> {
    match &attr.meta {
        Meta::NameValue(nv) => {
            if nv.path.is_ident("path") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    return Some(PathBuf::from(s.value()));
                }
            }
            None
        }
        _ => None,
    }
}

enum Part {
    Text(Range<usize>),
    Mod(ItemMod),
}

struct CodeBuilder {
    offset: usize,
    parts: Vec<Part>,
}
impl CodeBuilder {
    fn new() -> Self {
        Self {
            offset: 0,
            parts: Vec::new(),
        }
    }
    fn finish(self, source_len: usize) -> Vec<Part> {
        let mut parts = self.parts;
        parts.push(Part::Text(self.offset..source_len));
        parts
    }
}
impl<'ast> Visit<'ast> for CodeBuilder {
    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if i.content.is_some() {
            return;
        }
        let end = i.ident.span().byte_range().end;
        self.parts.push(Part::Text(self.offset..end));
        self.parts.push(Part::Mod(i.clone()));
        self.offset = i.span().byte_range().end;
    }
}
