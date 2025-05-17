use std::env;

use tera::Tera;

use crate::{error::AppError, field::Field};

#[derive(Debug, PartialEq)]
pub enum Template {
    Default,
    Eight,
    Disabled,
}

impl Template {
    pub fn generate_neos_input_string(&self, field: &Field) -> Result<String, AppError> {
        let tera = Tera::new("template/*.tera").expect("Failed to load template");

        let mut context = tera::Context::new();

        context.insert("size", &field.field_size);

        if field.start_cell.is_none() {
            return Err(AppError::StartNotSet);
        }

        if field.end_cell.is_none() {
            return Err(AppError::EndNotSet);
        }

        context.insert("start_x", &(field.start_cell.as_ref().unwrap().x + 1));
        context.insert("start_y", &(field.start_cell.as_ref().unwrap().y + 1));

        context.insert("end_x", &(field.end_cell.as_ref().unwrap().x + 1));
        context.insert("end_y", &(field.end_cell.as_ref().unwrap().y + 1));

        context.insert(
            "disabled_nodes",
            &field
                .filled_cells
                .iter()
                .map(|c| format!("({},{})", c.x + 1, c.y + 1))
                .collect::<Vec<_>>()
                .join(" "),
        );

        let code = tera
            .render(self.name(), &context)
            .map_err(|_| AppError::FailedRenderFile)?;

        let xml_input = format!(
            "
            <MyProblem>
                <category>milp</category>
                <solver>cplex</solver>
                <inputType>AMPL</inputType>
                <priority>long</priority>
                <email>{}</email>
                <model><![CDATA[
                {}
                ]]></model>
                <data></data>
                <commands></commands>
                <comments></comments>
            </MyProblem>
            ",
            env::var("EMAIL").unwrap(),
            code
        );

        Ok(xml_input)
    }

    pub fn name(&self) -> &str {
        match self {
            Template::Default => "path.tera",
            Template::Eight => "path_8.tera",
            Template::Disabled => "path_disabled.tera",
        }
    }
}
