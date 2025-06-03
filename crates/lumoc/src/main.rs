use std::fmt::{self, Debug};

use annotate_snippets::{Level, Renderer, Snippet};
use eyre::eyre;
use liblumoc::{Scope, infer_item};
use lumo_core::{SimpleType, SimpleTypeRef};
use lumo_syntax::{parse, tokenize};

const DEBUG_TOKENS: bool = false;
const DEBUG_ITEMS: bool = false;
const DEBUG_TYPES: bool = true;

fn main() -> eyre::Result<()> {
    let source = include_str!("../source.lumo");
    let tokens = match tokenize(source) {
        Ok(tokens) => tokens,
        Err(e) => {
            let input = e.input().input;
            let span = e.char_span();

            let message = Level::Error.title("invalid token").snippet(
                Snippet::source(input)
                    .fold(true)
                    .annotation(Level::Error.span(span)),
            );

            let renderer = Renderer::styled();
            println!("{}", renderer.render(message));

            eyre::bail!("error while tokenization");
        }
    };
    if DEBUG_TOKENS {
        for token in &tokens[..tokens.len() - 1] {
            println!("{}", token);
        }
    }
    let items = match parse(&tokens[..tokens.len() - 1]) {
        Ok(items) => items,
        Err(e) => {
            let message = e.inner().to_string().replace('\n', ": ");
            let input = e.input().input;
            let token = &input[e.offset().min(input.len() - 1)];

            let begin_token_idx = input
                .binary_search_by_key(&(token.span().start().line - 1), |t| t.span().start().line)
                .unwrap_or_else(|e| e);
            let end_token_idx = input
                .binary_search_by_key(&(token.span().end().line), |t| t.span().end().line)
                .unwrap_or_else(|e| e);

            let source: String = input[begin_token_idx..end_token_idx + 1]
                .iter()
                .map(|token| token.content.clone())
                .collect();
            let relative_offset =
                token.span().start().offset - input[begin_token_idx].span().start().offset;

            let message = Level::Error.title(&message).snippet(
                Snippet::source(&source).fold(true).annotation(
                    Level::Error.span(relative_offset..relative_offset + token.content.len()),
                ),
            );

            let renderer = Renderer::styled();
            println!("{}", renderer.render(message));

            eyre::bail!("error while parsing");
        }
    };
    if DEBUG_ITEMS {
        for item in &items {
            println!("{:#?}", item);
        }
    }
    let mut scope = Scope::new();
    for item in &items {
        let ty = infer_item(&mut scope, &item).map_err(|e| eyre!("{}", e.message))?;
        if DEBUG_TYPES {
            println!("{:#?}", RenderType(&scope, ty));
        }
    }
    Ok(())
}

struct RenderType<'a>(&'a Scope, SimpleTypeRef);

impl Debug for RenderType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Some(ty) = self.0.get(self.1.clone()) else {
            return Ok(());
        };
        match ty {
            SimpleType::Variable(variable_state) => {
                struct Bound<'a>(&'a Scope, &'a Vec<SimpleTypeRef>);
                impl Debug for Bound<'_> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        let mut tuple = f.debug_tuple("Bound");
                        for e in self.1 {
                            tuple.field(&RenderType(self.0, e.clone()));
                        }
                        tuple.finish()
                    }
                }
                let mut map = f.debug_struct(&format!("<#{}>", self.1.0));
                map.field("lower_bounds", &Bound(self.0, &variable_state.lower_bounds));
                map.field("upper_bounds", &Bound(self.0, &variable_state.upper_bounds));
                map.finish()
            }
            SimpleType::Primitive(name) => f.write_str(name),
            SimpleType::Function(args, ret) => {
                let mut tuple = f.debug_tuple("fn");
                for arg in args {
                    tuple.field(&RenderType(self.0, arg.clone()));
                }
                tuple.finish()?;
                f.write_str("=>")?;
                if f.alternate() {
                    write!(f, "{:#?}", RenderType(self.0, ret.clone()))
                } else {
                    write!(f, "{:?}", RenderType(self.0, ret.clone()))
                }
            }
            SimpleType::Todo => f.write_str("#TODO"),
        }
    }
}
