use std::fmt::{self, Debug};

use annotate_snippets::{Level, Renderer, Snippet};
use eyre::eyre;
use liblumoc::{Scope, coalesce_type, infer_item, scan};
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
    let mut scope = scan(&items).map_err(|e| eyre!("{}", e.message))?;
    if DEBUG_TYPES {
        println!("{:#?}", DebugScope(&scope));
    }
    for item in &items {
        let ty = infer_item(&mut scope, &item).map_err(|e| eyre!("{}", e.message))?;
        if DEBUG_TYPES {
            println!(
                "{} : {:#?}",
                item.representative_name(),
                coalesce_type(&scope, ty)
            );
        }
    }
    Ok(())
}

struct DebugScope<'a>(&'a Scope);

impl Debug for DebugScope<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Scope");
        for (name, ty) in self.0.entries() {
            dbg.field(&name, &coalesce_type(self.0, ty.clone()));
        }
        dbg.finish()
    }
}
