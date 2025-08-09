mod param_pink;

use tera::Tera;

use crate::{
    error::AppError,
    field::{cell::CellType, Field},
    neos::solver::Solver,
    template::param_pink::PinkPairParam,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Template {
    Default,
    Eight,
    Disabled,
    Multiple,
    MultipleSections,
    MultipleSeparated,
    CornerCutting,
    TurnCost(u32),
    Pink,
}

impl Template {
    pub fn variants() -> &'static [Template] {
        use Template::*;

        &[
            Default,
            Eight,
            Disabled,
            Multiple,
            MultipleSections,
            MultipleSeparated,
            CornerCutting,
            TurnCost(0),
            Pink,
        ]
    }

    pub fn generate_neos_input_string(
        &self,
        field: &Field,
        solver: &Solver,
        email: &str,
    ) -> Result<String, AppError> {
        let tera = Tera::new("template/*.tera").expect("Failed to load template");

        let mut context = tera::Context::new();

        context.insert("width", &field.width);
        context.insert("height", &field.height);

        if field.start_cell.is_none() {
            return Err(AppError::StartNotSet);
        }
        context.insert("start_x", &(field.start_cell.as_ref().unwrap().x));
        context.insert("start_y", &(field.start_cell.as_ref().unwrap().y));

        if field.end_cell.is_none() {
            return Err(AppError::EndNotSet);
        }
        context.insert("end_x", &(field.end_cell.as_ref().unwrap().x));
        context.insert("end_y", &(field.end_cell.as_ref().unwrap().y));

        let pink_pair_range: Vec<usize> = (1..=(field.pink_pair_map.len() / 2)).collect();
        context.insert("pink_pair_range", &pink_pair_range);
        let pink_pairs = field.unique_pink_pairs();
        context.insert("pink_pairs", &PinkPairParam::new(pink_pairs));

        if let Template::TurnCost(turn_cost) = self {
            context.insert("turn_cost", turn_cost);
        }

        context.insert(
            "disabled_nodes",
            &field
                .filled_cells
                .iter()
                .filter(|(_, cell_type)| cell_type != &&CellType::Pink)
                .map(|(cell, _)| format!("({},{})", cell.x, cell.y))
                .collect::<Vec<_>>()
                .join(" "),
        );

        let ampl_code = tera
            .render(&format!("{}.tera", self.name()), &context)
            .map_err(|_| AppError::FailedRenderFile)?;

        let xml_input = format!(
            "
            <MyProblem>
                <category>milp</category>
                <solver>{}</solver>
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
            solver.name(),
            email,
            ampl_code
        );

        Ok(xml_input)
    }

    pub fn name(&self) -> &str {
        match self {
            Template::Default => "path",
            Template::Eight => "path_8",
            Template::Disabled => "path_disabled",
            Template::Multiple => "path_multiple",
            Template::MultipleSections => "path_multiple_sections",
            Template::MultipleSeparated => "path_multiple_separated",
            Template::CornerCutting => "path_corner_cutting",
            Template::TurnCost(_) => "path_turn_cost",
            Template::Pink => "path_pink",
        }
    }
}
