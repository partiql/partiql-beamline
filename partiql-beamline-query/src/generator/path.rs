use crate::generator::{AstGenContext, AstGenerator};
use partiql_ast::ast;
use partiql_types::PartiqlShape;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub enum PathGenStep {
    PathProject(String),
    PathIndex(u32),
    PathForEach,
    PathUnpivot,
}

#[derive(Clone)]
pub struct PathAndShape {
    pub steps: Vec<PathGenStep>,
    pub shape: PartiqlShape,
}

impl Debug for PathAndShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut path = String::new();
        for step in &self.steps {
            match step {
                PathGenStep::PathProject(k) => {
                    path.push('.');
                    path.push_str(k);
                }
                PathGenStep::PathIndex(i) => {
                    path.push('[');
                    path.push_str(&i.to_string());
                    path.push(']');
                }
                PathGenStep::PathForEach => {
                    path.push_str("[*]");
                }
                PathGenStep::PathUnpivot => {
                    path.push_str(".*");
                }
            }
        }
        let shape = &self.shape;
        write!(f, "{path}: {shape:?}")
    }
}

pub type PathAndShapeSet = Vec<PathAndShape>;

#[derive(Debug, Clone)]
pub struct DatasetPaths {
    pub name: String,
    pub paths: PathAndShapeSet,
}

impl AstGenerator<ast::ProjectItem> for (&str, &PathAndShape) {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::ProjectItem {
        let path = self.gen_node(ctx);
        ast::ProjectItem::ProjectExpr(ast::ProjectExpr {
            expr: Box::new(ast::Expr::Path(path)),
            as_alias: None,
        })
    }
}

impl AstGenerator<ast::Expr> for (&str, &PathAndShape) {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Path(self.gen_node(ctx))
    }
}

impl AstGenerator<ast::Path> for (&str, &PathAndShape) {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Path {
        let (name, path_and_shape) = self;
        let root = ast::VarRef {
            name: ast::SymbolPrimitive {
                value: name.to_string(),
                case: ast::CaseSensitivity::CaseInsensitive,
            },
            qualifier: ast::ScopeQualifier::Unqualified,
        };
        let root = Box::new(ast::Expr::VarRef(ctx.node(root)));
        let steps = path_and_shape
            .steps
            .iter()
            .skip(1)
            .map(|step| step.gen_ast(ctx))
            .collect();
        ast::Path { root, steps }
    }
}

impl AstGenerator<ast::PathStep> for PathGenStep {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::PathStep {
        match self {
            PathGenStep::PathProject(name) => {
                let name = ast::SymbolPrimitive {
                    value: name.clone(),
                    case: ast::CaseSensitivity::CaseInsensitive,
                };
                let varref = ast::VarRef {
                    name,
                    qualifier: ast::ScopeQualifier::Unqualified,
                };
                let index = Box::new(ast::Expr::VarRef(ctx.node(varref)));
                let e = ast::PathExpr { index };
                ast::PathStep::PathProject(e)
            }
            PathGenStep::PathIndex(i) => {
                let idx = ast::Lit::Int64Lit(*i as i64);
                let index = Box::new(ast::Expr::Lit(ctx.node(idx)));
                let e = ast::PathExpr { index };
                ast::PathStep::PathIndex(e)
            }
            PathGenStep::PathForEach => ast::PathStep::PathForEach,
            PathGenStep::PathUnpivot => ast::PathStep::PathUnpivot,
        }
    }
}

impl AstGenerator<ast::ExcludePath> for (&str, &PathAndShape) {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::ExcludePath {
        let (name, path_and_shape) = self;
        let root = ast::VarRef {
            name: ast::SymbolPrimitive {
                value: name.to_string(),
                case: ast::CaseSensitivity::CaseInsensitive,
            },
            qualifier: ast::ScopeQualifier::Unqualified,
        };
        let root = ctx.node(root.clone());
        let steps = path_and_shape
            .steps
            .iter()
            .skip(1)
            .map(|step| step.gen_ast(ctx))
            .collect();
        ast::ExcludePath { root, steps }
    }
}

impl AstGenerator<ast::ExcludePathStep> for PathGenStep {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::ExcludePathStep {
        match self {
            PathGenStep::PathProject(name) => {
                let name = ast::SymbolPrimitive {
                    value: name.clone(),
                    case: ast::CaseSensitivity::CaseInsensitive,
                };
                ast::ExcludePathStep::PathProject(ctx.node(name))
            }
            PathGenStep::PathIndex(i) => {
                let idx = ast::Lit::Int64Lit(*i as i64);
                ast::ExcludePathStep::PathIndex(ctx.node(idx))
            }
            PathGenStep::PathForEach => ast::ExcludePathStep::PathForEach,
            PathGenStep::PathUnpivot => ast::ExcludePathStep::PathUnpivot,
        }
    }
}
