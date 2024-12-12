use std::collections::HashMap;

use crate::ast::*;

/// Analyzes the AST to replace `Variable::NotYetKnown` types with the appropriate kind of
/// `Variable`.
///
/// Returns the number of distinct scalar variables.
pub fn analyze(
    known_arrays: &HashMap<String, usize>,
    begin: &mut Option<Action>,
    end: &mut Option<Action>,
    routines: &mut [Routine],
) -> crate::Result<usize> {
    let mut vars = VarsMap::new(known_arrays);

    analyze_action(begin.as_mut(), &mut vars)?;

    for r in routines.iter_mut() {
        analyze_routine(r, &mut vars)?;
    }

    analyze_action(end.as_mut(), &mut vars)?;

    Ok(vars.num_scalars())
}

/// Analyzes an expression `e`, replacing `Variable::NotYetKnown` types with the appropriate kind of
/// variable.
fn analyze_expression(e: &mut Expression, vars: &mut VarsMap) -> crate::Result<()> {
    match e {
        Expression::Attr(_) => {}
        Expression::Atom(_) => {}
        Expression::Var(v) => {
            if let Variable::NotYetKnown(name) = v {
                *e = Expression::Var(vars.new_variable(name));
            }
        }
        Expression::Bin(b) => {
            analyze_expression(&mut b.left, vars)?;
            analyze_expression(&mut b.right, vars)?;
        }
    };

    Ok(())
}

fn analyze_assignment(a: &mut Assignment, vars: &mut VarsMap) -> crate::Result<()> {
    let new_lhs = match &a.lhs {
        Variable::NotYetKnown(name) => vars.new_variable(name),
        _ => a.lhs.clone(),
    };

    match new_lhs {
        // XXX: make this return an Error once I do error handling...
        Variable::Arr(_) => panic!("Cannot assign to an array name."),
        Variable::NotYetKnown(name) => panic!("Failed to resolve variable \"{name}\"."),
        _ => {}
    };

    a.lhs = new_lhs;

    analyze_expression(&mut a.rhs, vars)?;

    Ok(())
}

fn analyze_action(mut action: Option<&mut Action>, vars: &mut VarsMap) -> crate::Result<()> {
    let Some(ref mut action) = action else {
        return Ok(());
    };

    let Some(ref mut statements) = action.statements else {
        return Ok(());
    };

    for st in statements.iter_mut() {
        match st {
            Statement::Assignment(ref mut a) => analyze_assignment(a, vars)?,
            Statement::Print(pr) => {
                for expr in pr.iter_mut() {
                    analyze_expression(expr, vars)?;
                }
            }
        };
    }

    Ok(())
}

fn analyze_routine(routine: &mut Routine, vars: &mut VarsMap) -> crate::Result<()> {
    if let Some(cond) = &mut routine.cond {
        analyze_expression(&mut cond.expr, vars)?
    };

    analyze_action(Some(&mut routine.action), vars)
}

struct VarsMap<'a> {
    /// An immutable reference to the map for variables that are already known to be arrays.
    known_arrays: &'a HashMap<String, usize>,

    /// A mutable map for variables whose type will be discovered to be scalar.
    scalars_map: HashMap<String, usize>,
}

impl<'a> VarsMap<'a> {
    fn new(known_arrays: &'a HashMap<String, usize>) -> Self {
        VarsMap {
            known_arrays,
            scalars_map: HashMap::new(),
        }
    }

    fn new_variable(&mut self, name: &str) -> Variable {
        match self.known_arrays.get(name) {
            Some(id) => Variable::Arr(*id),
            None => self.scalar(name),
        }
    }

    fn scalar(&mut self, name: &str) -> Variable {
        let prev_len = self.scalars_map.len();

        let id = *self
            .scalars_map
            .entry(name.to_string())
            .or_insert_with(|| prev_len);

        Variable::Scalar(Identifier { id })
    }

    fn num_scalars(&self) -> usize {
        self.scalars_map.len()
    }
}
