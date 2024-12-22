#[macro_export]
macro_rules! find_n_with_comments {
    (class, $ast:ident) => {
        find_n_with_comments!(ClassDecl, ClassDecl, visit_class_decl, $ast)
    };
    (func, $ast:ident) => {
        find_n_with_comments!(FnDecl, FnDecl, visit_fn_decl, $ast)
    };
    ($class_name:ident, $type_name: ty, $func_name:ident, $ast:ident) => {{
        use swc_common::comments::Comments;
        use swc_common::Spanned;
        use swc_ecma_visit::VisitWith;
        struct Collector<'a> {
            out: Vec<($type_name, Vec<swc_common::comments::Comment>)>,
            ast: &'a ASTImpl,
            is_export: bool,
        }

        impl<'a> swc_ecma_visit::Visit for Collector<'a> {
            fn $func_name(&mut self, node: &swc_ecma_ast::$class_name) {
                if let Some(leading_comments) = self
                    .ast
                    .comments
                    .get_leading(node.span().lo - swc_common::BytePos(7 * self.is_export as u32))
                {
                    self.out.push((node.clone(), leading_comments.clone()));
                }

                node.visit_children_with(self)
            }
            fn visit_export_decl(&mut self, node: &swc_ecma_ast::ExportDecl) {
                self.is_export = true;
                node.visit_children_with(self);
                self.is_export = false;
            }
        }

        let mut collector = Collector {
            $ast,
            out: Vec::new(),
            is_export: false,
        };

        $ast.module.visit_with(&mut collector);
        if collector.out.is_empty() {
            None
        } else {
            Some::<Vec<($type_name, Vec<Comment>)>>(collector.out)
        }
    }};
}
