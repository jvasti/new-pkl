use crate::{
    parser::{AstPklValue, ExprHash, Identifier, PklExpr, PklResult, PklStatement},
    Pkl,
};
use data_size::Byte;
use duration::duration_from_value_and_unit;
use std::{fs, ops::Range, time::Duration};

#[cfg(feature = "hashbrown_support")]
use hashbrown::Hashmap as HashMap;
#[cfg(not(feature = "hashbrown_support"))]
use std::collections::HashMap;

mod data_size;
mod duration;

/// Represents a value in the PKL format.
///
/// The `PklValue` enum encapsulates various types of values that can be parsed from a PKL string.
/// These include booleans, floats, integers, strings, multiline strings, objects, and class instances.
///
/// # Variants
///
/// * `Bool` - Represents a boolean value.
/// * `Float` - Represents a floating-point number.
/// * `Int` - Represents an integer, which can be decimal, octal, hex, or binary.
/// * `String` - Represents a single-line string.
/// * `MultiLineString` - Represents a multiline string.
/// * `Object` - Represents a nested object, which is a hashmap of key-value pairs.
/// * `ClassInstance` - Represents an instance of a class, which includes the class name and its properties.
#[derive(Debug, PartialEq, Clone)]
pub enum PklValue<'a> {
    /// A boolean value.
    Bool(bool),

    /// A floating-point number.
    Float(f64),

    /// An integer value.
    Int(i64),

    /// A single-line string.
    String(&'a str),

    /// A multiline string.
    MultiLineString(&'a str),

    /// A nested object represented as a hashmap of key-value pairs.
    Object(HashMap<&'a str, PklValue<'a>>),

    /// An instance of a class, including the class name and its properties.
    ClassInstance(&'a str, HashMap<&'a str, PklValue<'a>>),

    /// A duration
    Duration(Duration),

    // A datasize
    DataSize(Byte),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PklTable<'a> {
    pub variables: HashMap<&'a str, PklValue<'a>>,
    imports: Vec<String>,
}

impl<'a> PklTable<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            imports: vec![],
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

    /// Merges another `PklTable` into this table.
    ///
    /// This method takes another `PklTable` and inserts all of its variables into the current table.
    /// If a variable with the same name already exists in the current table, it will be overwritten
    /// with the value from the other table.
    ///
    /// # Arguments
    ///
    /// * `other_table` - The `PklTable` to merge into the current table.
    ///
    /// # Example
    ///
    /// ```
    /// let mut table1 = PklTable::new();
    /// table1.insert("var1", PklValue::Int(1));
    ///
    /// let mut table2 = PklTable::new();
    /// table2.insert("var2", PklValue::Int(2));
    ///
    /// table1.extends(table2);
    ///
    /// assert_eq!(table1.get("var1"), Some(&PklValue::Int(1)));
    /// assert_eq!(table1.get("var2"), Some(&PklValue::Int(2)));
    /// ```
    pub fn extends(&mut self, other_table: PklTable<'a>) {
        for (name, value) in other_table.variables {
            self.insert(name, value);
        }
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

    pub fn import(&mut self, name: &'a str, rng: Range<usize>) -> PklResult<()> {
        match name {
            name if name.starts_with("package://") => {
                return Err(("Package imports not yet supported!".to_owned(), rng))
            }
            name if name.starts_with("pkl:") => {
                return Err((
                    "Pkl official packages imports not yet supported!".to_owned(),
                    rng,
                ))
            }
            name if name.starts_with("https://") => {
                return Err(("Web imports not yet supported!".to_owned(), rng))
            }
            file_name => {
                let file_content = fs::read_to_string(file_name)
                    .map_err(|e| (format!("Error reading {file_name}: {}", e.to_string()), rng))?;

                let mut pkl = Pkl::new();
                pkl.parse(&file_content)?;
                let hash = pkl.table.variables.to_owned();

                println!("{:?}", hash);
            }
        };

        return Ok(());
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
            PklExpr::Identifier(Identifier(id, range)) => self
                .variables
                .get(id)
                .cloned()
                .ok_or_else(|| (format!("unknown variable `{}`", id), range)),
            PklExpr::Value(value) => self.evaluate_value(value),
            PklExpr::MemberExpression(base_expr, indexor, range) => {
                let base = self.evaluate(*base_expr)?;
                let value = indexor.value();

                match base {
                    PklValue::Int(int) => {
                        if let Some(unit) = duration::Unit::from_str(value) {
                            return Ok(PklValue::Duration(duration_from_value_and_unit(
                                int as f64, unit,
                            )));
                        }

                        if let Some(unit) = data_size::Unit::from_str(value) {
                            return Ok(PklValue::DataSize(Byte::from_value_and_unit(
                                int as f64, unit,
                            )));
                        }
                    }
                    PklValue::Float(float) => {
                        if let Some(unit) = duration::Unit::from_str(value) {
                            return Ok(PklValue::Duration(duration_from_value_and_unit(
                                float, unit,
                            )));
                        }

                        if let Some(unit) = data_size::Unit::from_str(value) {
                            return Ok(PklValue::DataSize(Byte::from_value_and_unit(float, unit)));
                        }
                    }
                    PklValue::Object(hashmap) => {}
                    _ => {
                        return Err((
                            format!("Indexing of value '{:?}' not yet supported", base),
                            range,
                        ))
                    }
                };

                todo!()
            }
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

    let mut in_body = false;

    for statement in ast {
        match statement {
            PklStatement::Constant(name, expr, _) => {
                in_body = true;
                table.insert(name, table.evaluate(expr)?);
            }
            PklStatement::Import(value, local_name, rng) => {
                if in_body {
                    return Err((
                        "Import statements must be before document body".to_owned(),
                        rng,
                    ));
                }

                // it does not import for the moment, issue with lifetimes
                table.import(value, rng)?;
            }
        }
    }

    Ok(table)
}
