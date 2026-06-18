// klc_semantic::contracts — Contract validation
//
// Verifies that classes and structs properly implement their declared contracts.
// A contract is satisfied when the implementing type provides all methods
// with matching signatures.

use klc_core::ast::{ContractDecl, ClassDecl, ClassMember, Parameter};
use klc_core::diagnostic::{Diagnostic, DiagnosticReporter, ErrorCode};
use klc_core::span::Span;

#[derive(Clone, Debug)]
pub struct ContractMethodSig {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<klc_core::ast::AstType>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ContractInfo {
    pub name: String,
    pub methods: Vec<ContractMethodSig>,
    pub span: Span,
}

pub struct ContractChecker {
    pub contracts: Vec<ContractInfo>,
}

impl ContractChecker {
    pub fn new() -> Self {
        Self { contracts: Vec::new() }
    }

    pub fn register_contract(&mut self, decl: &ContractDecl) {
        let methods = decl.methods.iter().map(|m| ContractMethodSig {
            name: m.name.clone(),
            params: m.params.clone(),
            return_type: m.return_type.clone(),
            span: m.span,
        }).collect();

        self.contracts.push(ContractInfo {
            name: decl.name.clone(),
            methods,
            span: decl.span,
        });
    }

    pub fn find_contract(&self, name: &str) -> Option<&ContractInfo> {
        self.contracts.iter().find(|c| c.name == name)
    }

    pub fn check_class_contracts(&self, class: &ClassDecl, reporter: &mut DiagnosticReporter) {
        for contract_name in &class.contracts {
            if let Some(contract) = self.find_contract(contract_name) {
                for method_sig in &contract.methods {
                    if !self.class_has_method(class, method_sig) {
                        let diag = Diagnostic::error(
                            ErrorCode::E0001,
                            format!(
                                "class '{}' does not implement contract method '{}' from '{}'",
                                class.name, method_sig.name, contract.name
                            ),
                        )
                        .with_span(class.span)
                        .with_suggestion(format!(
                            "Add method '{}' to class '{}'",
                            method_sig.name, class.name
                        ));
                        reporter.report(diag);
                    }
                }
            } else {
                let diag = Diagnostic::error(
                    ErrorCode::E0009,
                    format!("contract '{}' not found", contract_name),
                )
                .with_span(class.span)
                .with_suggestion(format!("Define contract '{}' before using it", contract_name));
                reporter.report(diag);
            }
        }
    }

    fn class_has_method(&self, class: &ClassDecl, sig: &ContractMethodSig) -> bool {
        class.members.iter().any(|member| {
            if let ClassMember::Method(m) = member {
                m.name == sig.name
                    && m.params.len() == sig.params.len()
                    && m.return_type == sig.return_type
            } else {
                false
            }
        })
    }

    pub fn check_all_classes(&self, classes: &[&ClassDecl], reporter: &mut DiagnosticReporter) {
        for class in classes {
            self.check_class_contracts(class, reporter);
        }
    }
}

impl Default for ContractChecker {
    fn default() -> Self {
        Self::new()
    }
}
