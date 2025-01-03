use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct TreeType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

pub fn generate_ast(output_dir: &String) -> io::Result<()> {
    define_ast(
        output_dir,
        &"Expr".to_string(),
        &["error", "token"],
        &[
            "Assign     : Token name, Box<Expr> value",
            "Binary     : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping   : Box<Expr> expression",
            "Literal    : Option<Object> value",
            "Unary      : Token operator, Box<Expr> right",
            "Variable   : Token name",
        ],
    )?;

    define_ast(
        output_dir,
        &"Stmt".to_string(),
        &["error", "token", "expr"],
        &[
            "Expression : Expr expression",
            "Print      : Expr expression",
            "Var        : Token name, Option<Expr> initializer",
        ],
    )?;

    Ok(())
}

fn define_ast(
    output_dir: &String,
    base_name: &String,
    imports: &[&str],
    types: &[&str],
) -> io::Result<()> {
    let path: String = format!("{}/{}.rs", output_dir, base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();

    for i in imports {
        writeln!(file, "use crate::{}::*;", i)?;
    }

    for expr_type in types {
        let (base_class_name, args) = expr_type.split_once(":").unwrap();
        let class_name = format!("{}{}", base_class_name.trim(), base_name);
        let arg_split = args.split(",");
        let mut fields = Vec::new();
        for arg in arg_split {
            let (ttype, name) = arg.trim().split_once(" ").unwrap();
            fields.push(format!("{}: {}", name, ttype));
        }
        tree_types.push(TreeType {
            base_class_name: base_class_name.to_string().trim().to_string(),
            class_name,
            fields,
        });
    }

    // create Expr enum
    writeln!(file, "\npub enum {base_name} {{")?;
    for t in &tree_types {
        writeln!(file, "    {}({}),", t.base_class_name, t.class_name)?;
    }
    writeln!(file, "}}\n")?;

    // create impl Expr
    writeln!(file, "impl {} {{", base_name)?;
    writeln!(file, "    pub fn accept<T>(&self, {}_visitor: &dyn {base_name}Visitor<T>) -> Result<T, LoxError> {{", base_name.to_lowercase())?;
    writeln!(file, "        match self {{")?;
    for t in &tree_types {
        writeln!(
            file,
            "            {}::{}(v) => v.accept({}_visitor),",
            base_name,
            t.base_class_name,
            base_name.to_lowercase()
        )?;
    }
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}\n")?;

    // create Expr structs
    for t in &tree_types {
        writeln!(file, "pub struct {} {{", t.class_name)?;
        for f in &t.fields {
            writeln!(file, "    pub {},", f)?;
        }
        writeln!(file, "}}\n")?;
    }

    // create ExprVisitor trait
    writeln!(file, "pub trait {}Visitor<T> {{", base_name)?;
    for t in &tree_types {
        writeln!(
            file,
            "    fn visit_{}_{}(&self, expr: &{}) -> Result<T, LoxError>;",
            t.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            t.class_name
        )?;
    }
    writeln!(file, "}}\n")?;

    // create Expr impls
    for t in &tree_types {
        writeln!(file, "impl {} {{", t.class_name)?;
        writeln!(
            file,
            "    pub fn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> Result<T, LoxError> {{",
            base_name
        )?;
        writeln!(
            file,
            "        visitor.visit_{}_{}(self)",
            t.base_class_name.to_lowercase(),
            base_name.to_lowercase()
        )?;
        writeln!(file, "    }}")?;
        writeln!(file, "}}\n")?;
    }

    Ok(())
}
