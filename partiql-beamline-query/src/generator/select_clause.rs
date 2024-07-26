use crate::generator::{AstGenContext, AstGenerator};
use crate::strategy::path::{DatasetPaths, PathAndShape, PathGenStep};
use partiql_ast::ast;
use rand::distributions::Distribution;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;
use std::ops::DerefMut;

#[derive(Clone, Debug)]
pub struct SelectStar {}

impl AstGenerator<ast::Projection> for SelectStar {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Projection {
        ast::Projection {
            kind: ast::ProjectionKind::ProjectStar,
            setq: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectPaths {
    pub paths: DatasetPaths,
    pub amount: statrs::distribution::DiscreteUniform,
    pub rng: RefCell<Pcg64Mcg>,
}

impl AstGenerator<ast::Projection> for SelectPaths {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Projection {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let num = self.amount.sample(rng) as usize;

        let root = ast::VarRef {
            name: ast::SymbolPrimitive {
                value: self.paths.name.clone(),
                case: ast::CaseSensitivity::CaseInsensitive,
            },
            qualifier: ast::ScopeQualifier::Unqualified,
        };

        let path_sel =
            DiscreteUniform::new(0, (self.paths.paths.len() - 1) as i64).expect("sample");
        let mut items = Vec::with_capacity(num);
        for _ in 0..num {
            let root = Box::new(ast::Expr::VarRef(ctx.node(root.clone())));
            let idx = path_sel.sample(rng) as usize;
            let PathAndShape { steps, .. } = &self.paths.paths[idx];
            let steps = steps.iter().skip(1).map(|step| step.gen_ast(ctx)).collect();
            let path = ctx.node(ast::Path { root, steps });

            let item = ast::ProjectItem::ProjectExpr(ast::ProjectExpr {
                expr: Box::new(ast::Expr::Path(path)),
                as_alias: None,
            });
            items.push(ctx.node(item));
        }

        ast::Projection {
            kind: ast::ProjectionKind::ProjectList(items),
            setq: None,
        }
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
