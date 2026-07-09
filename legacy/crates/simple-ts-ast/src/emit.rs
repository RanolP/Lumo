use crate::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitTarget {
    TypeScript,
    TypeScriptDefinition,
    JavaScript,
}

#[derive(Debug, Default)]
pub struct Emitter {
    out: String,
    indent: usize,
}

impl Emitter {
    pub fn emit_program(mut self, program: &Program, target: EmitTarget) -> String {
        for (i, stmt) in program.body.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.emit_stmt(stmt, target);
        }
        self.out
    }

    fn emit_stmt(&mut self, stmt: &Stmt, target: EmitTarget) {
        match stmt {
            Stmt::Expr(expr) => {
                self.line(&format!("{};", self.emit_expr(expr, target)));
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.line(&format!("return {};", self.emit_expr(expr, target)));
                } else {
                    self.line("return;");
                }
            }
            Stmt::Const(decl) => {
                if target == EmitTarget::TypeScriptDefinition {
                    let export = if decl.export { "export " } else { "" };
                    let ty = decl
                        .type_ann
                        .as_ref()
                        .map(|t| format!(": {}", self.emit_ts_type(t)))
                        .unwrap_or_else(|| ": unknown".to_string());
                    self.line(&format!("{}declare const {}{};", export, decl.name, ty));
                } else {
                    let export = if decl.export { "export " } else { "" };
                    let ty = if target == EmitTarget::TypeScript {
                        decl.type_ann
                            .as_ref()
                            .map(|t| format!(": {}", self.emit_ts_type(t)))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    self.line(&format!(
                        "{}const {}{} = {};",
                        export,
                        decl.name,
                        ty,
                        self.emit_expr(&decl.init, target)
                    ));
                }
            }
            Stmt::Let { name, export, type_ann, init } => {
                let export = if *export { "export " } else { "" };
                let ty = if target == EmitTarget::TypeScript {
                    type_ann.as_ref()
                        .map(|t| format!(": {}", self.emit_ts_type(t)))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                if let Some(init_expr) = init {
                    self.line(&format!(
                        "{}let {}{} = {};",
                        export, name, ty,
                        self.emit_expr(init_expr, target)
                    ));
                } else {
                    self.line(&format!("{}let {}{};", export, name, ty));
                }
            }
            Stmt::Assign { name, value } => {
                self.line(&format!("{} = {};", name, self.emit_expr(value, target)));
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.emit_if_chain(cond, then_branch, else_branch.as_ref(), target);
            }
            Stmt::Block(block) => {
                self.line("{");
                self.indent += 1;
                self.emit_block_items(block, target);
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Function(decl) => {
                if target == EmitTarget::TypeScriptDefinition {
                    self.emit_function_decl_signature(decl, true);
                } else {
                    self.emit_function_decl(decl, target);
                }
            }
            Stmt::TypeAlias(alias) => {
                if target != EmitTarget::JavaScript {
                    let export = if alias.export { "export " } else { "" };
                    let type_params = if alias.type_params.is_empty() {
                        String::new()
                    } else {
                        format!("<{}>", alias.type_params.join(", "))
                    };
                    self.line(&format!(
                        "{}type {}{} = {};",
                        export,
                        alias.name,
                        type_params,
                        self.emit_ts_type(&alias.ty)
                    ));
                }
            }
            Stmt::Interface(interface) => {
                if target != EmitTarget::JavaScript {
                    let export = if interface.export { "export " } else { "" };
                    self.line(&format!("{}interface {} {{", export, interface.name));
                    self.indent += 1;
                    for member in &interface.members {
                        let opt = if member.optional { "?" } else { "" };
                        self.line(&format!(
                            "{}{}: {};",
                            member.name,
                            opt,
                            self.emit_ts_type(&member.ty)
                        ));
                    }
                    self.indent -= 1;
                    self.line("}");
                }
            }
        }
    }

    fn emit_function_decl(&mut self, decl: &FunctionDecl, target: EmitTarget) {
        let export = if decl.export { "export " } else { "" };
        let type_params = if target == EmitTarget::TypeScript && !decl.type_params.is_empty() {
            format!("<{}>", decl.type_params.join(", "))
        } else {
            String::new()
        };
        let params = self.emit_params(&decl.params, target);
        let ret = if target == EmitTarget::TypeScript {
            decl.return_type
                .as_ref()
                .map(|t| format!(": {}", self.emit_ts_type(t)))
                .unwrap_or_default()
        } else {
            String::new()
        };
        self.line(&format!(
            "{}function {}{}({}){} {{",
            export, decl.name, type_params, params, ret
        ));
        self.indent += 1;
        match &decl.body {
            FunctionBody::Expr(expr) => {
                self.line(&format!("return {};", self.emit_expr(expr, target)));
            }
            FunctionBody::Block(block) => {
                self.emit_block_items(block, target);
            }
        }
        self.indent -= 1;
        self.line("}");
    }

    fn emit_function_decl_signature(&mut self, decl: &FunctionDecl, declare_kw: bool) {
        let export = if decl.export { "export " } else { "" };
        let declare = if declare_kw { "declare " } else { "" };
        let type_params = if decl.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", decl.type_params.join(", "))
        };
        let params = self.emit_params(&decl.params, EmitTarget::TypeScript);
        let ret = decl
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.emit_ts_type(t)))
            .unwrap_or_else(|| ": unknown".to_string());
        self.line(&format!(
            "{}{}function {}{}({}){};",
            export, declare, decl.name, type_params, params, ret
        ));
    }

    fn emit_block_items(&mut self, block: &Block, target: EmitTarget) {
        for stmt in &block.stmts {
            self.emit_stmt(stmt, target);
        }
    }

    /// Emit an if/else chain, flattening `else { if ... }` into `else if`.
    fn emit_if_chain(
        &mut self,
        cond: &Expr,
        then_branch: &Block,
        else_branch: Option<&Block>,
        target: EmitTarget,
    ) {
        self.line(&format!("if ({}) {{", self.emit_expr(cond, target)));
        self.indent += 1;
        self.emit_block_items(then_branch, target);
        self.indent -= 1;
        let mut current = else_branch;
        while let Some(branch) = current {
            // Detect `else { if (...) ... }` and collapse to `else if`.
            if branch.stmts.len() == 1 {
                if let Stmt::If {
                    cond: nested_cond,
                    then_branch: nested_then,
                    else_branch: nested_else,
                } = &branch.stmts[0]
                {
                    self.line(&format!(
                        "}} else if ({}) {{",
                        self.emit_expr(nested_cond, target)
                    ));
                    self.indent += 1;
                    self.emit_block_items(nested_then, target);
                    self.indent -= 1;
                    current = nested_else.as_ref();
                    continue;
                }
            }
            self.line("} else {");
            self.indent += 1;
            self.emit_block_items(branch, target);
            self.indent -= 1;
            self.line("}");
            return;
        }
        self.line("}");
    }

    fn emit_params(&self, params: &[Param], target: EmitTarget) -> String {
        params
            .iter()
            .map(|p| {
                if target == EmitTarget::JavaScript {
                    p.name.clone()
                } else if let Some(ty) = &p.type_ann {
                    format!("{}: {}", p.name, self.emit_ts_type(ty))
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn emit_expr(&self, expr: &Expr, target: EmitTarget) -> String {
        match expr {
            Expr::Ident(name) => name.clone(),
            Expr::String(value) => {
                let escaped = value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                format!("\"{escaped}\"")
            }
            Expr::Number(value) => value.to_string(),
            Expr::Bool(value) => value.to_string(),
            Expr::Null => "null".to_string(),
            Expr::Undefined => "undefined".to_string(),
            Expr::Void(expr) => format!("void {}", self.emit_expr(expr, target)),
            Expr::Unary { op, expr } => {
                format!("({}{})", op.as_str(), self.emit_expr(expr, target))
            }
            Expr::Binary { left, op, right } => format!(
                "({} {} {})",
                self.emit_expr(left, target),
                op.as_str(),
                self.emit_expr(right, target)
            ),
            Expr::Call { callee, args } => {
                let args = args
                    .iter()
                    .map(|a| self.emit_expr(a, target))
                    .collect::<Vec<_>>()
                    .join(", ");
                let callee_text = self.emit_expr(callee, target);
                if needs_parens_as_callee(callee) {
                    format!("({callee_text})({args})")
                } else {
                    format!("{callee_text}({args})")
                }
            }
            Expr::Member { object, property } => {
                format!("{}.{}", self.emit_expr(object, target), property)
            }
            Expr::Index { object, index } => format!(
                "{}[{}]",
                self.emit_expr(object, target),
                self.emit_expr(index, target)
            ),
            Expr::Array(items) => {
                let items = items
                    .iter()
                    .map(|item| self.emit_expr(item, target))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items)
            }
            Expr::Object(props) => {
                let props = props
                    .iter()
                    .map(|prop| {
                        format!(
                            "{}: {}",
                            self.emit_object_key(&prop.key, target),
                            self.emit_expr(&prop.value, target)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", props)
            }
            Expr::Arrow {
                params,
                return_type,
                body,
            } => {
                let params = self.emit_params(params, target);
                let ret = if target == EmitTarget::TypeScript {
                    return_type
                        .as_ref()
                        .map(|t| format!(": {}", self.emit_ts_type(t)))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                match body.as_ref() {
                    FunctionBody::Expr(expr) => {
                        format!("({}){} => {}", params, ret, self.emit_expr(expr, target))
                    }
                    FunctionBody::Block(block) => {
                        let mut body_emitter = Emitter {
                            out: String::new(),
                            indent: 0,
                        };
                        body_emitter.line("{");
                        body_emitter.indent += 1;
                        body_emitter.emit_block_items(block, target);
                        body_emitter.indent -= 1;
                        body_emitter.line("}");
                        format!("({}){} => {}", params, ret, body_emitter.out.trim_end())
                    }
                }
            }
            Expr::IfElse {
                cond,
                then_expr,
                else_expr,
            } => format!(
                "({} ? {} : {})",
                self.emit_expr(cond, target),
                self.emit_expr(then_expr, target),
                self.emit_expr(else_expr, target)
            ),
        }
    }

    fn emit_object_key(&self, key: &ObjectKey, target: EmitTarget) -> String {
        match key {
            ObjectKey::Ident(name) => name.clone(),
            ObjectKey::String(value) => format!("\"{}\"", value.replace('"', "\\\"")),
            ObjectKey::Computed(expr) => format!("[{}]", self.emit_expr(expr, target)),
        }
    }

    fn emit_ts_type(&self, ty: &TsType) -> String {
        match ty {
            TsType::Any => "any".to_string(),
            TsType::Unknown => "unknown".to_string(),
            TsType::Never => "never".to_string(),
            TsType::Void => "void".to_string(),
            TsType::Boolean => "boolean".to_string(),
            TsType::Number => "number".to_string(),
            TsType::String => "string".to_string(),
            TsType::Null => "null".to_string(),
            TsType::Undefined => "undefined".to_string(),
            TsType::TypeRef(name) => name.clone(),
            TsType::Array(inner) => format!("{}[]", self.emit_ts_type(inner)),
            TsType::Union(items) => items
                .iter()
                .map(|it| self.emit_ts_type(it))
                .collect::<Vec<_>>()
                .join(" | "),
            TsType::Func { params, ret } => format!(
                "({}) => {}",
                params
                    .iter()
                    .map(|p| {
                        let ty = p
                            .type_ann
                            .as_ref()
                            .map(|t| self.emit_ts_type(t))
                            .unwrap_or_else(|| "unknown".to_string());
                        format!("{}: {}", p.name, ty)
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                self.emit_ts_type(ret)
            ),
            TsType::Raw(text) => text.clone(),
        }
    }

    fn line(&mut self, text: &str) {
        // Multi-line expressions (e.g. `emit_expr` returning a block-body arrow)
        // arrive with their own internal indentation relative to column 0. To
        // nest them into the current context, prefix every non-empty line —
        // not just the first — with the current indent level.
        let prefix = "  ".repeat(self.indent);
        let mut first = true;
        for segment in text.split('\n') {
            if !first {
                self.out.push('\n');
            }
            if !segment.is_empty() {
                self.out.push_str(&prefix);
                self.out.push_str(segment);
            }
            first = false;
        }
        self.out.push('\n');
    }

    fn newline(&mut self) {
        self.out.push('\n');
    }
}

fn needs_parens_as_callee(expr: &Expr) -> bool {
    !matches!(
        expr,
        Expr::Ident(_) | Expr::Member { .. } | Expr::Index { .. } | Expr::Call { .. }
    )
}
