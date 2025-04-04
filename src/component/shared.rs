use crate::ext::AstBuilderExt;
use oxc_allocator::{Allocator, FromIn, IntoIn};
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_ast::AstBuilder;
use oxc_span::SPAN;
use std::convert::Into;
use std::path::{Path, PathBuf};

pub const QWIK_CORE_SOURCE: &str = "@qwik.dev/core";
pub const MARKER_SUFFIX: &str = "$";
pub const QRL: &str = "qrl";
pub const QRL_SUFFIX: &str = "Qrl";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommonImport {
    QwikCore(Vec<ImportId>),
    Import(Import),
}

impl CommonImport {
    pub fn qrl() -> CommonImport {
        CommonImport::QwikCore(vec![QRL.into()])
    }
}

impl<'a> FromIn<'a, CommonImport> for Statement<'a> {
    fn from_in(value: CommonImport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonImport::QwikCore(names) => {
                ast_builder.create_import_statement(names, QWIK_CORE_SOURCE)
            }
            CommonImport::Import(import) => import.into_statement(allocator),
        }
    }
}

impl<'a> FromIn<'a, &CommonImport> for Statement<'a> {
    fn from_in(value: &CommonImport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonImport::QwikCore(names) => {
                let names = names.clone();
                ast_builder.create_import_statement(names, QWIK_CORE_SOURCE)
            }
            CommonImport::Import(import) => import.into_in(allocator),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonExport {
    BuilderIoQwik(String),
}

impl<'a> FromIn<'a, CommonExport> for Statement<'a> {
    fn from_in(value: CommonExport, allocator: &'a Allocator) -> Self {
        let ast_builder = AstBuilder::new(allocator);
        match value {
            CommonExport::BuilderIoQwik(name) => {
                ast_builder.create_export_statement(name.as_str(), QWIK_CORE_SOURCE)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportId {
    Named(String),
    NamedWithAlias(String, String),
    Default(String),
    Namespace(String),
}

impl From<&str> for ImportId {
    fn from(value: &str) -> Self {
        ImportId::Named(value.to_string())
    }
}

impl From<&mut ImportDeclarationSpecifier<'_>> for ImportId {
    fn from(value: &mut ImportDeclarationSpecifier<'_>) -> Self {
        match value {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let imported = specifier.imported.name().to_string();
                let local_name = specifier.local.name.to_string();
                if imported == local_name {
                    ImportId::Named(imported)
                } else {
                    ImportId::NamedWithAlias(imported, local_name)
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                let local_name = specifier.local.name.to_string();
                ImportId::Default(local_name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                let local_name = specifier.local.name.to_string();
                ImportId::Namespace(local_name)
            }
        }
    }
}

impl<'a> FromIn<'a, ImportId> for ImportDeclarationSpecifier<'a> {
    fn from_in(value: ImportId, allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        match value {
            ImportId::Named(name) => {
                let imported = ast.module_export_name_identifier_name(SPAN, &name);
                let local_name = ast.binding_identifier(SPAN, &name);
                ast.import_declaration_specifier_import_specifier(
                    SPAN,
                    imported,
                    local_name,
                    ImportOrExportKind::Value,
                )
            }

            ImportId::NamedWithAlias(name, local_name) => {
                let imported = ast.module_export_name_identifier_name(SPAN, &name);
                let local_name = ast.binding_identifier(SPAN, &local_name);
                ast.import_declaration_specifier_import_specifier(
                    SPAN,
                    imported,
                    local_name,
                    ImportOrExportKind::Value,
                )
            }
            ImportId::Namespace(local_name) => {
                let local_name = ast.binding_identifier(SPAN, &local_name);
                ast.import_declaration_specifier_import_namespace_specifier(SPAN, local_name)
            }
            ImportId::Default(name) => {
                let local_name = ast.binding_identifier(SPAN, &name);
                ast.import_declaration_specifier_import_default_specifier(SPAN, local_name)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Import {
    names: Vec<ImportId>,
    source: PathBuf,
}

impl Import {
    pub fn new<T: Into<PathBuf>>(names: Vec<ImportId>, source: T) -> Self {
        Self {
            names,
            source: source.into(),
        }
    }

    pub fn into_statement<'a>(&self, allocator: &'a Allocator) -> Statement<'a> {
        let ast_builder = AstBuilder::new(allocator);
        ast_builder.create_import_statement(self.names.clone(), self.source.to_string_lossy())
    }
}

impl<'a> FromIn<'a, &Import> for Statement<'a> {
    fn from_in(value: &Import, allocator: &'a Allocator) -> Self {
        value.into_statement(allocator)
    }
}

impl<'a> FromIn<'a, Import> for Statement<'a> {
    fn from_in(value: Import, allocator: &'a Allocator) -> Self {
        value.into_statement(allocator)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reference {
    Variable(String),
}

impl Reference {
    pub fn name(&self) -> String {
        match self {
            Reference::Variable(name) => name.clone(),
        }
    }
    pub fn into_import<T: AsRef<Path>>(&self, source: T) -> Import {
        match self {
            Reference::Variable(name) => Import::new(vec![name.as_str().into()], source.as_ref()),
        }
    }
}

/// Renamed from `EmitMode` in V 1.0.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Prod,
    Lib,
    Dev,
    Test,
}
