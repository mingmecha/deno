use deno_ast::{
  swc::{
    ast::{
      AssignTarget, BlockStmtOrExpr, Callee, Decl, Expr, ForHead, Lit,
      MemberProp, ModuleDecl, ModuleItem, Pat, Program, Prop, PropOrSpread,
      SimpleAssignTarget, Stmt, SuperProp, TsType, VarDeclOrExpr,
    },
    common::Span,
  },
  ParsedSource,
};

// Keep in sync with JS
enum AstNode {
  Invalid,
  //
  Program,

  // Module declarations
  Import,
  ImportDecl,
  ExportDecl,
  ExportNamed,
  ExportDefaultDecl,
  ExportDefaultExpr,
  ExportAll,
  TsImportEquals,
  TsExportAssignment,
  TsNamespaceExport,

  // Decls
  Class,
  Fn,
  Var,
  Using,
  TsInterface,
  TsTypeAlias,
  TsEnum,
  TsModule,

  // Statements
  Block,
  Empty,
  Debugger,
  With,
  Return,
  Labeled,
  Break,
  Continue,
  If,
  Switch,
  SwitchCase,
  Throw,
  Try,
  While,
  DoWhile,
  For,
  ForIn,
  ForOf,
  Decl,
  Expr,

  // Expressions
  This,
  Array,
  Object,
  FnExpr,
  Unary,
  Update,
  Bin,
  Assign,
  Member,
  SuperProp,
  Cond,
  Call,
  New,
  Seq,
  Ident,
  Tpl,
  TaggedTpl,
  Arrow,
  ClassExpr,
  Yield,
  MetaProp,
  Await,
  TsTypeAssertion,
  TsConstAssertion,
  TsNonNull,
  TsAs,
  TsInstantiation,
  TsSatisfies,
  PrivateName,
  OptChain,

  // Literals
  StringLiteral,
  Bool,
  Null,
  Num,
  BigInt,
  Regex,

  // JSX
  JSXMember,
  JSXNamespacedName,
  JSXEmpty,
  JSXElement,
  JSXFragment,
  JSXText,

  // Custom
  EmptyExpr,
  Spread,
  ObjProperty,
  VarDeclarator,
}

impl From<AstNode> for u8 {
  fn from(m: AstNode) -> u8 {
    m as u8
  }
}

enum Flags {
  None,
}

impl From<Flags> for u8 {
  fn from(m: Flags) -> u8 {
    m as u8
  }
}

pub fn serialize_ast_bin(parsed_source: &ParsedSource) -> Vec<u8> {
  let mut result: Vec<u8> = vec![];

  let program = &parsed_source.program();
  match program.as_ref() {
    Program::Module(module) => {
      push_node(
        &mut result,
        AstNode::Program.into(),
        Flags::None.into(),
        module.body.len().into(),
        &module.span,
      );
      for item in &module.body {
        match item {
          ModuleItem::ModuleDecl(module_decl) => {
            serialize_module_decl(&mut result, module_decl)
          }
          ModuleItem::Stmt(stmt) => serialize_stmt(&mut result, stmt),
        }
      }
    }
    Program::Script(script) => {
      push_node(
        &mut result,
        AstNode::Program.into(),
        Flags::None.into(),
        script.body.len().into(),
        &script.span,
      );

      for stmt in &script.body {
        serialize_stmt(&mut result, stmt)
      }
    }
  }

  result
}

fn push_node(
  result: &mut Vec<u8>,
  kind: u8,
  flags: u8,
  count: usize,
  span: &Span,
) {
  result.push(kind);
  result.push(flags);

  if count < 127 {
    result.push(count.try_into().unwrap());
    result.push(0);
    result.push(0);
    result.push(0);
  } else {
    // FIXME
    result.push(0);
    result.push(0);
    result.push(0);
    result.push(0);
  }

  // FIXME: Span
  result.push(0);
  result.push(0);
  result.push(0);
  result.push(0);

  result.push(0);
  result.push(0);
  result.push(0);
  result.push(0);
}

fn serialize_module_decl(result: &mut Vec<u8>, module_decl: &ModuleDecl) {
  match module_decl {
    ModuleDecl::Import(import_decl) => {
      push_node(
        result,
        AstNode::Import.into(),
        Flags::None.into(),
        0,
        &import_decl.span,
      );
    }
    ModuleDecl::ExportDecl(export_decl) => {
      push_node(
        result,
        AstNode::ExportDecl.into(),
        Flags::None.into(),
        0,
        &export_decl.span,
      );
    }
    ModuleDecl::ExportNamed(named_export) => {
      push_node(
        result,
        AstNode::ExportNamed.into(),
        Flags::None.into(),
        0,
        &named_export.span,
      );
    }
    ModuleDecl::ExportDefaultDecl(export_default_decl) => {
      push_node(
        result,
        AstNode::ExportDefaultDecl.into(),
        Flags::None.into(),
        0,
        &export_default_decl.span,
      );
    }
    ModuleDecl::ExportDefaultExpr(export_default_expr) => {
      push_node(
        result,
        AstNode::ExportDefaultExpr.into(),
        Flags::None.into(),
        0,
        &export_default_expr.span,
      );
    }
    ModuleDecl::ExportAll(export_all) => {
      push_node(
        result,
        AstNode::ExportAll.into(),
        Flags::None.into(),
        0,
        &export_all.span,
      );
    }
    ModuleDecl::TsImportEquals(ts_import_equals_decl) => {
      push_node(
        result,
        AstNode::TsImportEquals.into(),
        Flags::None.into(),
        0,
        &ts_import_equals_decl.span,
      );
    }
    ModuleDecl::TsExportAssignment(ts_export_assignment) => {
      push_node(
        result,
        AstNode::TsExportAssignment.into(),
        Flags::None.into(),
        0,
        &ts_export_assignment.span,
      );
    }
    ModuleDecl::TsNamespaceExport(ts_namespace_export_decl) => {
      push_node(
        result,
        AstNode::TsNamespaceExport.into(),
        Flags::None.into(),
        0,
        &ts_namespace_export_decl.span,
      );
    }
  }
}

fn serialize_stmt(result: &mut Vec<u8>, stmt: &Stmt) {
  match stmt {
    Stmt::Block(block_stmt) => {
      push_node(
        result,
        AstNode::Block.into(),
        Flags::None.into(),
        block_stmt.stmts.len().into(),
        &block_stmt.span,
      );

      for child in &block_stmt.stmts {
        serialize_stmt(result, child);
      }
    }
    Stmt::Empty(empty_stmt) => {
      push_node(
        result,
        AstNode::Empty.into(),
        Flags::None.into(),
        0,
        &empty_stmt.span,
      );
    }
    Stmt::Debugger(debugger_stmt) => {
      push_node(
        result,
        AstNode::Debugger.into(),
        Flags::None.into(),
        0,
        &debugger_stmt.span,
      );
    }
    Stmt::With(_) => todo!(),
    Stmt::Return(return_stmt) => {
      let count = if return_stmt.arg.is_some() { 1 } else { 0 };
      push_node(
        result,
        AstNode::Return.into(),
        Flags::None.into(),
        count,
        &return_stmt.span,
      );
    }
    Stmt::Labeled(labeled_stmt) => {
      push_node(
        result,
        AstNode::Labeled.into(),
        Flags::None.into(),
        1,
        &labeled_stmt.span,
      );
    }
    Stmt::Break(break_stmt) => {
      let count = if break_stmt.label.is_some() { 1 } else { 0 };
      push_node(
        result,
        AstNode::Break.into(),
        Flags::None.into(),
        count,
        &break_stmt.span,
      );
    }
    Stmt::Continue(continue_stmt) => {
      let count = if continue_stmt.label.is_some() { 1 } else { 0 };
      push_node(
        result,
        AstNode::Continue.into(),
        Flags::None.into(),
        count,
        &continue_stmt.span,
      );
    }
    Stmt::If(if_stmt) => {
      let count = if if_stmt.alt.is_some() { 3 } else { 2 };
      push_node(
        result,
        AstNode::If.into(),
        Flags::None.into(),
        count,
        &if_stmt.span,
      );

      serialize_expr(result, if_stmt.test.as_ref());
      serialize_stmt(result, if_stmt.cons.as_ref());

      if let Some(alt) = &if_stmt.alt {
        serialize_stmt(result, &alt);
      }
    }
    Stmt::Switch(switch_stmt) => {
      push_node(
        result,
        AstNode::Switch.into(),
        Flags::None.into(),
        switch_stmt.cases.len().into(),
        &switch_stmt.span,
      );

      for case in &switch_stmt.cases {
        let count = if case.test.is_some() {
          case.cons.len() + 1
        } else {
          case.cons.len()
        };
        push_node(
          result,
          AstNode::SwitchCase.into(),
          Flags::None.into(),
          count,
          &case.span,
        );
      }
    }
    Stmt::Throw(throw_stmt) => {
      push_node(
        result,
        AstNode::Throw.into(),
        Flags::None.into(),
        1,
        &throw_stmt.span,
      );
    }
    Stmt::Try(try_stmt) => {
      let mut count = 1;
      if try_stmt.finalizer.is_some() {
        count += 1
      }
      if try_stmt.handler.is_some() {
        count += 1
      }

      push_node(
        result,
        AstNode::Try.into(),
        Flags::None.into(),
        count,
        &try_stmt.span,
      );

      serialize_stmt(result, &Stmt::Block(try_stmt.block.clone()));
    }
    Stmt::While(while_stmt) => {
      push_node(
        result,
        AstNode::While.into(),
        Flags::None.into(),
        2,
        &while_stmt.span,
      );

      serialize_expr(result, while_stmt.test.as_ref());
      serialize_stmt(result, while_stmt.body.as_ref());
    }
    Stmt::DoWhile(do_while_stmt) => {
      push_node(
        result,
        AstNode::DoWhile.into(),
        Flags::None.into(),
        2,
        &do_while_stmt.span,
      );

      serialize_expr(result, do_while_stmt.test.as_ref());
      serialize_stmt(result, do_while_stmt.body.as_ref());
    }
    Stmt::For(for_stmt) => {
      push_node(
        result,
        AstNode::For.into(),
        Flags::None.into(),
        4,
        &for_stmt.span,
      );

      if let Some(init) = &for_stmt.init {
        match init {
          VarDeclOrExpr::VarDecl(var_decl) => {
            serialize_stmt(result, &Stmt::Decl(Decl::Var(var_decl.clone())));
          }
          VarDeclOrExpr::Expr(expr) => {
            serialize_expr(result, expr);
          }
        }
      } else {
        push_node(
          result,
          AstNode::EmptyExpr.into(),
          Flags::None.into(),
          0,
          &for_stmt.span,
        );
      }

      if let Some(test_expr) = &for_stmt.test {
        serialize_expr(result, test_expr.as_ref());
      } else {
        push_node(
          result,
          AstNode::EmptyExpr.into(),
          Flags::None.into(),
          0,
          &for_stmt.span,
        );
      }

      if let Some(update_expr) = &for_stmt.update {
        serialize_expr(result, update_expr.as_ref());
      } else {
        push_node(
          result,
          AstNode::EmptyExpr.into(),
          Flags::None.into(),
          0,
          &for_stmt.span,
        );
      }

      serialize_stmt(result, &for_stmt.body.as_ref());
    }
    Stmt::ForIn(for_in_stmt) => {
      push_node(
        result,
        AstNode::ForIn.into(),
        Flags::None.into(),
        3,
        &for_in_stmt.span,
      );

      match &for_in_stmt.left {
        ForHead::VarDecl(var_decl) => {}
        ForHead::UsingDecl(using_decl) => {}
        ForHead::Pat(pat) => {}
      }

      serialize_expr(result, for_in_stmt.right.as_ref());
      serialize_stmt(result, for_in_stmt.body.as_ref());
    }
    Stmt::ForOf(for_of_stmt) => {
      push_node(
        result,
        AstNode::ForOf.into(),
        Flags::None.into(),
        3,
        &for_of_stmt.span,
      );

      match &for_of_stmt.left {
        ForHead::VarDecl(var_decl) => {}
        ForHead::UsingDecl(using_decl) => {}
        ForHead::Pat(pat) => {}
      }

      serialize_expr(result, for_of_stmt.right.as_ref());
      serialize_stmt(result, for_of_stmt.body.as_ref());
    }
    Stmt::Decl(decl) => serialize_decl(result, decl),
    Stmt::Expr(expr_stmt) => {
      push_node(
        result,
        AstNode::Expr.into(),
        Flags::None.into(),
        1,
        &expr_stmt.span,
      );
      serialize_expr(result, expr_stmt.expr.as_ref());
    }
  }
}

fn serialize_decl(result: &mut Vec<u8>, decl: &Decl) {
  match decl {
    Decl::Class(class_decl) => {
      push_node(
        result,
        AstNode::Class.into(),
        Flags::None.into(),
        0,
        &class_decl.class.span,
      );

      //
    }
    Decl::Fn(fn_decl) => {
      push_node(
        result,
        AstNode::Fn.into(),
        Flags::None.into(),
        0,
        &fn_decl.function.span,
      );

      if let Some(body) = &fn_decl.function.as_ref().body {
        serialize_stmt(result, &Stmt::Block(body.clone()));
      }
    }
    Decl::Var(var_decl) => {
      let count = var_decl.decls.len();
      push_node(
        result,
        AstNode::Var.into(),
        Flags::None.into(),
        count,
        &var_decl.span,
      );

      for decl in &var_decl.decls {
        let count = if decl.init.is_some() { 2 } else { 1 };
        push_node(
          result,
          AstNode::VarDeclarator.into(),
          Flags::None.into(),
          count,
          &decl.span,
        );

        match &decl.name {
          Pat::Ident(binding_ident) => {
            serialize_expr(result, &Expr::Ident(binding_ident.id.clone()));
          }
          Pat::Array(array_pat) => {}
          Pat::Rest(rest_pat) => {}
          Pat::Object(object_pat) => {}
          Pat::Assign(assign_pat) => {}
          Pat::Invalid(invalid) => {}
          Pat::Expr(expr) => {
            serialize_expr(result, expr.as_ref());
          }
        }

        if let Some(init) = &decl.init {
          serialize_expr(result, init.as_ref());
        }
      }
    }
    Decl::Using(using_decl) => {
      let count = using_decl.decls.len();
      push_node(
        result,
        AstNode::Using.into(),
        Flags::None.into(),
        count.into(),
        &using_decl.span,
      );

      for decl in &using_decl.decls {
        // TODO
      }
    }
    Decl::TsInterface(ts_interface_decl) => {
      let mut count = 2 + ts_interface_decl.extends.len();
      if ts_interface_decl.type_params.is_some() {
        count += 1;
      }

      push_node(
        result,
        AstNode::TsInterface.into(),
        Flags::None.into(),
        count,
        &ts_interface_decl.span,
      );
    }
    Decl::TsTypeAlias(ts_type_alias_decl) => {
      push_node(
        result,
        AstNode::TsTypeAlias.into(),
        Flags::None.into(),
        0,
        &ts_type_alias_decl.span,
      );
    }
    Decl::TsEnum(ts_enum_decl) => {
      let count = 1 + ts_enum_decl.members.len();
      push_node(
        result,
        AstNode::TsEnum.into(),
        Flags::None.into(),
        count,
        &ts_enum_decl.span,
      );
    }
    Decl::TsModule(ts_module_decl) => {
      push_node(
        result,
        AstNode::TsModule.into(),
        Flags::None.into(),
        0,
        &ts_module_decl.span,
      );
    }
  }
}

fn serialize_expr(result: &mut Vec<u8>, expr: &Expr) {
  match expr {
    Expr::This(this_expr) => {
      push_node(
        result,
        AstNode::This.into(),
        Flags::None.into(),
        0,
        &this_expr.span,
      );
    }
    Expr::Array(array_lit) => {
      let count = array_lit.elems.len();
      push_node(
        result,
        AstNode::Array.into(),
        Flags::None.into(),
        count,
        &array_lit.span,
      );

      for maybe_item in &array_lit.elems {
        if let Some(item) = maybe_item {
          //
        } else {
          //
        }
        // FIXME
      }
    }
    Expr::Object(object_lit) => {
      let count = object_lit.props.len();
      push_node(
        result,
        AstNode::Object.into(),
        Flags::None.into(),
        count,
        &object_lit.span,
      );

      for prop in &object_lit.props {
        match prop {
          PropOrSpread::Spread(spread_element) => {
            push_node(
              result,
              AstNode::Spread.into(),
              Flags::None.into(),
              1,
              &spread_element.dot3_token,
            );
            serialize_expr(result, spread_element.expr.as_ref());
          }
          PropOrSpread::Prop(prop) => match prop.as_ref() {
            Prop::Shorthand(ident) => {
              serialize_expr(result, &Expr::Ident(ident.clone()));
            }
            Prop::KeyValue(key_value_prop) => {
              serialize_expr(result, key_value_prop.value.as_ref())
            }
            Prop::Assign(assign_prop) => {
              push_node(
                result,
                AstNode::Assign.into(),
                Flags::None.into(),
                2,
                &assign_prop.span,
              );
              serialize_expr(result, &Expr::Ident(assign_prop.key.clone()));
              serialize_expr(result, assign_prop.value.as_ref())
            }
            Prop::Getter(getter_prop) => {
              // TODO
              if let Some(stmt) = &getter_prop.body {
                serialize_stmt(result, &Stmt::Block(stmt.clone()));
              }
            }
            Prop::Setter(setter_prop) => {
              // TODO
              if let Some(body) = &setter_prop.body {
                serialize_stmt(result, &Stmt::Block(body.clone()));
              }
            }
            Prop::Method(method_prop) => {
              if let Some(body) = &method_prop.function.body {
                serialize_stmt(result, &Stmt::Block(body.clone()));
              }
            }
          },
        }
      }
    }
    Expr::Fn(fn_expr) => {
      let fn_obj = fn_expr.function.as_ref();
      push_node(
        result,
        AstNode::FnExpr.into(),
        Flags::None.into(),
        0,
        &fn_obj.span,
      );
    }
    Expr::Unary(unary_expr) => {
      push_node(
        result,
        AstNode::Unary.into(),
        Flags::None.into(),
        1,
        &unary_expr.span,
      );
    }
    Expr::Update(update_expr) => {
      push_node(
        result,
        AstNode::Update.into(),
        Flags::None.into(),
        1,
        &update_expr.span,
      );
      serialize_expr(result, update_expr.arg.as_ref());
    }
    Expr::Bin(bin_expr) => {
      push_node(
        result,
        AstNode::Bin.into(),
        Flags::None.into(),
        2,
        &bin_expr.span,
      );
      serialize_expr(result, bin_expr.left.as_ref());
      serialize_expr(result, bin_expr.right.as_ref());
    }
    Expr::Assign(assign_expr) => {
      push_node(
        result,
        AstNode::Assign.into(),
        Flags::None.into(),
        2,
        &assign_expr.span,
      );

      match &assign_expr.left {
        AssignTarget::Simple(simple_assign_target) => {
          match simple_assign_target {
            SimpleAssignTarget::Ident(binding_ident) => {
              serialize_expr(result, &Expr::Ident(binding_ident.id.clone()));
            }
            SimpleAssignTarget::Member(member_expr) => {
              serialize_expr(result, &Expr::Member(member_expr.clone()));
            }
            SimpleAssignTarget::SuperProp(super_prop_expr) => {}
            SimpleAssignTarget::Paren(paren_expr) => {}
            SimpleAssignTarget::OptChain(opt_chain_expr) => {}
            SimpleAssignTarget::TsAs(ts_as_expr) => {}
            SimpleAssignTarget::TsSatisfies(ts_satisfies_expr) => {}
            SimpleAssignTarget::TsNonNull(ts_non_null_expr) => {}
            SimpleAssignTarget::TsTypeAssertion(ts_type_assertion) => {}
            SimpleAssignTarget::TsInstantiation(ts_instantiation) => {}
            SimpleAssignTarget::Invalid(invalid) => {}
          }
        }
        AssignTarget::Pat(assign_target_pat) => {}
      }

      serialize_expr(result, assign_expr.right.as_ref());
    }
    Expr::Member(member_expr) => {
      push_node(
        result,
        AstNode::Member.into(),
        Flags::None.into(),
        2,
        &member_expr.span,
      );
      serialize_expr(result, member_expr.obj.as_ref());

      match &member_expr.prop {
        MemberProp::Ident(ident_name) => {}
        MemberProp::PrivateName(private_name) => {}
        MemberProp::Computed(computed_prop_name) => {
          serialize_expr(result, computed_prop_name.expr.as_ref());
        }
      }
    }
    Expr::SuperProp(super_prop_expr) => {
      push_node(
        result,
        AstNode::SuperProp.into(),
        Flags::None.into(),
        2,
        &super_prop_expr.span,
      );
      // FIXME
      match &super_prop_expr.prop {
        SuperProp::Ident(ident_name) => {}
        SuperProp::Computed(computed_prop_name) => {}
      }
    }
    Expr::Cond(cond_expr) => {
      push_node(
        result,
        AstNode::Cond.into(),
        Flags::None.into(),
        3,
        &cond_expr.span,
      );

      serialize_expr(result, cond_expr.test.as_ref());
      serialize_expr(result, cond_expr.cons.as_ref());
      serialize_expr(result, cond_expr.alt.as_ref());
    }
    Expr::Call(call_expr) => {
      let count = 1 + call_expr.args.len();
      push_node(
        result,
        AstNode::Call.into(),
        Flags::None.into(),
        count,
        &call_expr.span,
      );

      match &call_expr.callee {
        Callee::Super(_) => {}
        Callee::Import(import) => {}
        Callee::Expr(expr) => {
          serialize_expr(result, expr);
        }
      }

      for arg in &call_expr.args {
        if let Some(spread) = &arg.spread {
          push_node(
            result,
            AstNode::Spread.into(),
            Flags::None.into(),
            1,
            spread,
          );
        }

        serialize_expr(result, arg.expr.as_ref());
      }
    }
    Expr::New(new_expr) => {
      let mut count = 1;

      if let Some(args) = &new_expr.args {
        count += args.len()
      }
      if new_expr.type_args.is_some() {
        count += 1;
      }

      push_node(
        result,
        AstNode::New.into(),
        Flags::None.into(),
        count,
        &new_expr.span,
      );
      serialize_expr(result, new_expr.callee.as_ref());

      if let Some(args) = &new_expr.args {
        for arg in args {
          if let Some(spread) = &arg.spread {
            push_node(
              result,
              AstNode::Spread.into(),
              Flags::None.into(),
              1,
              spread,
            );
          }

          serialize_expr(result, arg.expr.as_ref());
        }
      }
    }
    Expr::Seq(seq_expr) => {
      let count = seq_expr.exprs.len();
      push_node(
        result,
        AstNode::Seq.into(),
        Flags::None.into(),
        count,
        &seq_expr.span,
      );

      for expr in &seq_expr.exprs {
        serialize_expr(result, expr);
      }
    }
    Expr::Ident(ident) => {
      push_node(
        result,
        AstNode::Ident.into(),
        Flags::None.into(),
        0,
        &ident.span,
      );
    }
    Expr::Lit(lit) => {
      serialize_lit(result, lit);
    }
    Expr::Tpl(tpl) => {
      push_node(
        result,
        AstNode::Tpl.into(),
        Flags::None.into(),
        0,
        &tpl.span,
      );
    }
    Expr::TaggedTpl(tagged_tpl) => {
      push_node(
        result,
        AstNode::TaggedTpl.into(),
        Flags::None.into(),
        0,
        &tagged_tpl.span,
      );
    }
    Expr::Arrow(arrow_expr) => {
      let mut count = 1 + arrow_expr.params.len();
      if arrow_expr.return_type.is_some() {
        count += 1;
      }
      if arrow_expr.type_params.is_some() {
        count += 1;
      }

      push_node(
        result,
        AstNode::Arrow.into(),
        Flags::None.into(),
        count,
        &arrow_expr.span,
      );

      match arrow_expr.body.as_ref() {
        BlockStmtOrExpr::BlockStmt(block_stmt) => {
          serialize_stmt(result, &Stmt::Block(block_stmt.clone()));
        }
        BlockStmtOrExpr::Expr(expr) => {
          serialize_expr(result, expr.as_ref());
        }
      }
    }
    Expr::Class(class_expr) => {
      // push_node(result, AstNode::Class.into(), &class_expr.span);
    }
    Expr::Yield(yield_expr) => {
      let count = if yield_expr.arg.is_some() { 1 } else { 0 };
      push_node(
        result,
        AstNode::Yield.into(),
        Flags::None.into(),
        count,
        &yield_expr.span,
      );

      if let Some(arg) = &yield_expr.arg {
        serialize_expr(result, arg.as_ref());
      }
    }
    Expr::MetaProp(meta_prop_expr) => {
      push_node(
        result,
        AstNode::MetaProp.into(),
        Flags::None.into(),
        0,
        &meta_prop_expr.span,
      );
    }
    Expr::Await(await_expr) => {
      push_node(
        result,
        AstNode::Await.into(),
        Flags::None.into(),
        1,
        &await_expr.span,
      );

      serialize_expr(result, await_expr.arg.as_ref());
    }
    Expr::Paren(paren_expr) => {
      push_node(
        result,
        AstNode::Seq.into(),
        Flags::None.into(),
        1,
        &paren_expr.span,
      );

      serialize_expr(result, paren_expr.expr.as_ref());
    }
    Expr::JSXMember(jsxmember_expr) => {
      push_node(
        result,
        AstNode::JSXMember.into(),
        Flags::None.into(),
        0,
        &jsxmember_expr.span,
      );
    }
    Expr::JSXNamespacedName(jsxnamespaced_name) => {
      push_node(
        result,
        AstNode::JSXNamespacedName.into(),
        Flags::None.into(),
        0,
        &jsxnamespaced_name.span,
      );
    }
    Expr::JSXEmpty(jsxempty_expr) => {
      push_node(
        result,
        AstNode::JSXEmpty.into(),
        Flags::None.into(),
        0,
        &jsxempty_expr.span,
      );
    }
    Expr::JSXElement(jsxelement) => {
      push_node(
        result,
        AstNode::JSXElement.into(),
        Flags::None.into(),
        0,
        &jsxelement.span,
      );
    }
    Expr::JSXFragment(jsxfragment) => {
      push_node(
        result,
        AstNode::JSXFragment.into(),
        Flags::None.into(),
        0,
        &jsxfragment.span,
      );
    }
    Expr::TsTypeAssertion(ts_type_assertion) => {
      push_node(
        result,
        AstNode::TsTypeAssertion.into(),
        Flags::None.into(),
        0,
        &ts_type_assertion.span,
      );
    }
    Expr::TsConstAssertion(ts_const_assertion) => {
      push_node(
        result,
        AstNode::TsConstAssertion.into(),
        Flags::None.into(),
        1,
        &ts_const_assertion.span,
      );
      serialize_expr(result, ts_const_assertion.expr.as_ref());
    }
    Expr::TsNonNull(ts_non_null_expr) => {
      push_node(
        result,
        AstNode::TsNonNull.into(),
        Flags::None.into(),
        1,
        &ts_non_null_expr.span,
      );
      serialize_expr(result, ts_non_null_expr.expr.as_ref());
    }
    Expr::TsAs(ts_as_expr) => {
      push_node(
        result,
        AstNode::TsAs.into(),
        Flags::None.into(),
        2,
        &ts_as_expr.span,
      );
      serialize_expr(result, ts_as_expr.expr.as_ref());
      serialize_ts_type(result, ts_as_expr.type_ann.as_ref());
    }
    Expr::TsInstantiation(ts_instantiation) => {
      push_node(
        result,
        AstNode::TsInstantiation.into(),
        Flags::None.into(),
        2,
        &ts_instantiation.span,
      );
      serialize_expr(result, ts_instantiation.expr.as_ref());

      // FIXME
    }
    Expr::TsSatisfies(ts_satisfies_expr) => {
      push_node(
        result,
        AstNode::TsSatisfies.into(),
        Flags::None.into(),
        2,
        &ts_satisfies_expr.span,
      );
      serialize_expr(result, ts_satisfies_expr.expr.as_ref());
      serialize_ts_type(result, ts_satisfies_expr.type_ann.as_ref());
    }
    Expr::PrivateName(private_name) => {
      push_node(
        result,
        AstNode::PrivateName.into(),
        Flags::None.into(),
        0,
        &private_name.span,
      );
    }
    Expr::OptChain(opt_chain_expr) => {
      push_node(
        result,
        AstNode::OptChain.into(),
        Flags::None.into(),
        0,
        &opt_chain_expr.span,
      );
    }
    Expr::Invalid(invalid) => {
      // push_node(result, AstNode::Invalid.into(), &invalid.span);
    }
  }
}

fn serialize_lit(result: &mut Vec<u8>, lit: &Lit) {
  match lit {
    Lit::Str(lit_str) => push_node(
      result,
      AstNode::StringLiteral.into(),
      Flags::None.into(),
      0,
      &lit_str.span,
    ),
    Lit::Bool(lit_bool) => push_node(
      result,
      AstNode::Bool.into(),
      Flags::None.into(),
      0,
      &lit_bool.span,
    ),
    Lit::Null(null) => push_node(
      result,
      AstNode::Null.into(),
      Flags::None.into(),
      0,
      &null.span,
    ),
    Lit::Num(number) => push_node(
      result,
      AstNode::Num.into(),
      Flags::None.into(),
      0,
      &number.span,
    ),
    Lit::BigInt(big_int) => push_node(
      result,
      AstNode::BigInt.into(),
      Flags::None.into(),
      0,
      &big_int.span,
    ),
    Lit::Regex(regex) => push_node(
      result,
      AstNode::Regex.into(),
      Flags::None.into(),
      0,
      &regex.span,
    ),
    Lit::JSXText(jsxtext) => push_node(
      result,
      AstNode::JSXText.into(),
      Flags::None.into(),
      0,
      &jsxtext.span,
    ),
  }
}

fn serialize_ts_type(result: &mut Vec<u8>, ts_type: &TsType) {
  match ts_type {
    TsType::TsKeywordType(ts_keyword_type) => {}
    TsType::TsThisType(ts_this_type) => {}
    TsType::TsFnOrConstructorType(ts_fn_or_constructor_type) => {}
    TsType::TsTypeRef(ts_type_ref) => {}
    TsType::TsTypeQuery(ts_type_query) => {}
    TsType::TsTypeLit(ts_type_lit) => {}
    TsType::TsArrayType(ts_array_type) => {}
    TsType::TsTupleType(ts_tuple_type) => {}
    TsType::TsOptionalType(ts_optional_type) => {}
    TsType::TsRestType(ts_rest_type) => {}
    TsType::TsUnionOrIntersectionType(ts_union_or_intersection_type) => {}
    TsType::TsConditionalType(ts_conditional_type) => {}
    TsType::TsInferType(ts_infer_type) => {}
    TsType::TsParenthesizedType(ts_parenthesized_type) => {}
    TsType::TsTypeOperator(ts_type_operator) => {}
    TsType::TsIndexedAccessType(ts_indexed_access_type) => {}
    TsType::TsMappedType(ts_mapped_type) => {}
    TsType::TsLitType(ts_lit_type) => {}
    TsType::TsTypePredicate(ts_type_predicate) => {}
    TsType::TsImportType(ts_import_type) => {}
  }
}