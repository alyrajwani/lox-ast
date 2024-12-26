use std::io::{self, Write};
use std::fs::File;

#[derive(Debug)]
struct TreeType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

pub fn generate_ast(output_dir: &String) -> io::Result<()> {
    define_ast(output_dir, &"Expr".to_string(), &vec![
        "Binary     : Box<Expr> left, Token operator, Box<Expr> right".to_string(),
        "Grouping   : Box<Expr> expression".to_string(),
        "Literal    : Object value".to_string(),
        "Unary      : Token operatore Box<Expr> right".to_string()
    ])?;
    
    Ok(())
}

fn define_ast(output_dir: &String, base_name: &String, types: &[String]) -> io::Result<()> {
    let path: String = format!("{}/{}.rs", output_dir, base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();    

    write!(file, "{}", "use crate::error::*;\n")?;
    write!(file, "{}", "use crate::token::*;\n")?;

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
    write!(file, "\npub enum {base_name} {{\n")?;
    for t in &tree_types {
        write!(file, "    {}({}),\n", t.base_class_name, t.class_name)?;
    }
    write!(file, "}}\n\n")?;

    // create Expr structs
    for t in &tree_types {
        write!(file, "pub struct {} {{\n", t.class_name)?;
        for f in &t.fields {
            write!(file, "    {},\n", f)?;
        }
        write!(file, "}}\n\n")?;
    }

    // create ExprVisitor trait
    write!(file, "pub trait ExprVisitor<T> {{\n")?;
    for t in &tree_types {
        write!(file, "    fn visit_{}_{}(&self, expr: &{}) -> Result<T, LoxError>;\n",
            t.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            t.class_name)?;
    }
    write!(file, "}}\n\n")?;

    // create Expr impls
    for t in &tree_types {
        write!(file, "impl {} {{\n", t.class_name)?;
        write!(file, "    fn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> Result<T, LoxError> {{\n", base_name)?;
        write!(file, "        visitor.visitor_{}_{}(self)\n", t.base_class_name.to_lowercase(), base_name.to_lowercase())?;
        write!(file, "    }}\n")?;
        write!(file, "}}\n\n")?;
    }

    Ok(()) 
}
