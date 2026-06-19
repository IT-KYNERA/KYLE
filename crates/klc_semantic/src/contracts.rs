use std::collections::HashMap;
use klc_core::ast::*;
use klc_core::diagnostic::{Diagnostic, ErrorCode, DiagnosticReporter};

pub struct ContractChecker {
    contracts: HashMap<String, ContractDecl>,
}

impl ContractChecker {
    pub fn new() -> Self {
        Self { contracts: HashMap::new() }
    }

    pub fn register_contract(&mut self, contract: &ContractDecl) {
        self.contracts.insert(contract.name.clone(), contract.clone());
    }

    pub fn check_all_classes(&self, classes: &[&ClassDecl], reporter: &mut DiagnosticReporter) {
        for class in classes {
            for contract_name in &class.contracts {
                if let Some(contract) = self.contracts.get(contract_name) {
                    self.validate_class_contract(class, contract, reporter);
                } else {
                    reporter.report(
                        Diagnostic::error(ErrorCode::E0009,
                            format!("contract '{}' not found", contract_name))
                            .with_span(class.span)
                    );
                }
            }
        }
    }

    fn validate_class_contract(&self, class: &ClassDecl, contract: &ContractDecl, reporter: &mut DiagnosticReporter) {
        for req_method in &contract.methods {
            let found = class.members.iter().any(|member| {
                if let ClassMember::Method(m) = member {
                    if m.name != req_method.name { return false; }
                    if m.params.len() != req_method.params.len() { return false; }
                    m.params.iter().zip(&req_method.params).all(|(a, b)| {
                        a.type_ == b.type_
                    })
                } else {
                    false
                }
            });
            if !found {
                reporter.report(
                    Diagnostic::error(ErrorCode::E0001,
                        format!("class '{}' does not implement contract method '{}'",
                            class.name, req_method.name))
                        .with_span(class.span)
                );
            }
        }
    }
}