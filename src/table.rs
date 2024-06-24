use crate::parser::{AstPklValue, ExprHash, PklExpr, PklResult, PklStatement};
use std::ops::Range;

#[cfg(feature = "hashbrown_support")]
use hashbrown::Hashmap as HashMap;
#[cfg(not(feature = "hashbrown_support"))]
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum PklValue<'a> {
    Bool(bool),
    Float(f64),
    Int(i64),
    String(&'a str),
    MultiLineString(&'a str),
    Object(HashMap<&'a str, PklValue<'a>>),
    ClassInstance(&'a str, HashMap<&'a str, PklValue<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PklTable<'a> {
    pub variables: HashMap<&'a str, PklValue<'a>>,
}

impl<'a> PklTable<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Inserts a variable with the given name and value into the context.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to insert.
    /// * `value` - The value of the variable to insert.
    ///
    /// # Returns
    ///
    /// An `Option` containing the previous value associated with the name, if any.
    pub fn insert(&mut self, name: &'a str, value: PklValue<'a>) -> Option<PklValue<'a>> {
        self.variables.insert(name, value)
    }

    /// Retrieves the value of a variable with the given name from the context.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `PklValue` associated with the name,
    /// or `None` if the variable is not found.
    pub fn get(&self, name: &'a str) -> Option<&PklValue<'a>> {
        self.variables.get(name)
    }

    /// Evaluates an expression in the current context.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to evaluate.
    ///
    /// # Returns
    ///
    /// A `PklResult` containing the evaluated value or an error message with the range.
    pub fn evaluate(&self, expr: PklExpr<'a>) -> PklResult<PklValue<'a>> {
        match expr {
            PklExpr::Identifier(id, range) => self
                .variables
                .get(id)
                .cloned()
                .ok_or_else(|| (format!("unknown variable `{}`", id), range)),
            PklExpr::Value(value) => self.evaluate_value(value),
        }
    }

    /// Evaluates an AST PKL value in the current context.
    ///
    /// # Arguments
    ///
    /// * `value` - The AST PKL value to evaluate.
    ///
    /// # Returns
    ///
    /// A `PklResult` containing the evaluated value or an error message with the range.
    fn evaluate_value(&self, value: AstPklValue<'a>) -> PklResult<PklValue<'a>> {
        let result = match value {
            AstPklValue::Bool(b, _) => PklValue::Bool(b),
            AstPklValue::Float(f, _) => PklValue::Float(f),
            AstPklValue::Int(i, _) => PklValue::Int(i),
            AstPklValue::String(s, _) => PklValue::String(s),
            AstPklValue::MultiLineString(s, _) => PklValue::MultiLineString(s),
            AstPklValue::Object(o) => self.evaluate_object(o)?,
            AstPklValue::ClassInstance(a, b, _) => self.evaluate_class_instance(a, b)?,
            AstPklValue::AmendedObject(a, b, _) => self.evaluate_amended_object(a, b)?,
            AstPklValue::AmendingObject(a, b, rng) => self.evaluate_amending_object(a, b, rng)?,
        };

        Ok(result)
    }

    fn evaluate_object(&self, o: ExprHash<'a>) -> PklResult<PklValue<'a>> {
        let new_hash: Result<HashMap<_, _>, _> =
            o.0.into_iter()
                .map(|(name, expr)| {
                    let evaluated_expr = self.evaluate(expr)?;
                    Ok((name, evaluated_expr))
                })
                .collect();

        new_hash.map(PklValue::Object)
    }

    fn evaluate_class_instance(&self, a: &'a str, b: ExprHash<'a>) -> PklResult<PklValue<'a>> {
        let new_hash: Result<HashMap<_, _>, _> =
            b.0.into_iter()
                .map(|(name, expr)| {
                    let evaluated_expr = self.evaluate(expr)?;
                    Ok((name, evaluated_expr))
                })
                .collect();

        new_hash.map(|h| PklValue::ClassInstance(a, h))
    }

    fn evaluate_amending_object(
        &self,
        a: &'a str,
        b: ExprHash<'a>,
        rng: Range<usize>,
    ) -> PklResult<PklValue<'a>> {
        let other_object = match self.get(a) {
            Some(PklValue::Object(hash)) => hash,
            _ => return Err((format!("Unknown object `{}`", a), rng)),
        };

        let mut new_hash = other_object.clone();
        for (name, expr) in b.0 {
            new_hash.insert(name, self.evaluate(expr)?);
        }

        Ok(PklValue::Object(new_hash))
    }

    fn evaluate_amended_object(
        &self,
        a: Box<AstPklValue<'a>>,
        b: ExprHash<'a>,
    ) -> PklResult<PklValue<'a>> {
        let first_object = match self.evaluate_value(*a)? {
            PklValue::Object(o) => o,
            _ => unreachable!("should not be reached due to the parser work"),
        };

        let mut new_hash = first_object;
        for (name, expr) in b.0 {
            new_hash.insert(name, self.evaluate(expr)?);
        }

        Ok(PklValue::Object(new_hash))
    }
}

pub fn ast_to_table<'a>(ast: Vec<PklStatement<'a>>) -> PklResult<PklTable<'a>> {
    let mut table = PklTable::new();

    for statement in ast {
        match statement {
            PklStatement::Constant(name, expr, _) => {
                table.insert(name, table.evaluate(expr)?);
            }
        }
    }

    Ok(table)
}
