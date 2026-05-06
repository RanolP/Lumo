use crate::{lir, lir_memaware};

/// Convert a pure-functional `lir::File` into a `lir_memaware::File`.
///
/// This trivial pass wraps every function/method body in `Pure(…)`.
/// Dup/Drop insertion is added in a subsequent pass once usage analysis is in place.
pub fn elaborate(file: &lir::File) -> lir_memaware::File {
    lir_memaware::File {
        content_hash: file.content_hash.clone(),
        spans: file.spans.clone(),
        items: file.items.iter().map(elaborate_item).collect(),
    }
}

fn elaborate_item(item: &lir::Item) -> lir_memaware::Item {
    match item {
        lir::Item::Fn(f)         => lir_memaware::Item::Fn(elaborate_fn(f)),
        lir::Item::Impl(i)       => lir_memaware::Item::Impl(elaborate_impl(i)),
        lir::Item::ExternType(e) => lir_memaware::Item::ExternType(e.clone()),
        lir::Item::ExternFn(e)   => lir_memaware::Item::ExternFn(e.clone()),
        lir::Item::Data(d)       => lir_memaware::Item::Data(d.clone()),
        lir::Item::Cap(c)        => lir_memaware::Item::Cap(c.clone()),
        lir::Item::Use(u)        => lir_memaware::Item::Use(u.clone()),
    }
}

fn elaborate_fn(f: &lir::FnDecl) -> lir_memaware::FnDecl {
    lir_memaware::FnDecl {
        name:        f.name.clone(),
        generics:    f.generics.clone(),
        params:      f.params.clone(),
        return_type: f.return_type.clone(),
        cap:         f.cap.clone(),
        inline:      f.inline,
        span:        f.span,
        value:       lir_memaware::Expr::Pure(f.value.clone()),
    }
}

fn elaborate_impl(i: &lir::ImplDecl) -> lir_memaware::ImplDecl {
    lir_memaware::ImplDecl {
        name:         i.name.clone(),
        generics:     i.generics.clone(),
        target_type:  i.target_type.clone(),
        capability:   i.capability.clone(),
        assoc_types:  i.assoc_types.clone(),
        span:         i.span,
        methods:      i.methods.iter().map(elaborate_method).collect(),
    }
}

fn elaborate_method(m: &lir::ImplMethodDecl) -> lir_memaware::ImplMethodDecl {
    lir_memaware::ImplMethodDecl {
        name:        m.name.clone(),
        params:      m.params.clone(),
        return_type: m.return_type.clone(),
        span:        m.span,
        value:       lir_memaware::Expr::Pure(m.value.clone()),
    }
}
