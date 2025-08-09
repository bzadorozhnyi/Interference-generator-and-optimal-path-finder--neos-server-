#[derive(Clone, Copy, PartialEq)]
pub enum Solver {
    Cbc,
    Copt,
    Cplex,
    FicoXpress,
    Highs,
    Minto,
    Mosek,
    Raposa,
}

impl Solver {
    pub fn variants() -> &'static [Solver] {
        use Solver::*;

        &[Cbc, Copt, Cplex, FicoXpress, Highs, Minto, Mosek, Raposa]
    }

    pub fn name(&self) -> &str {
        match self {
            Solver::Cbc => "cbc",
            Solver::Copt => "copt",
            Solver::Cplex => "cplex",
            Solver::FicoXpress => "fico-xpress",
            Solver::Highs => "highs",
            Solver::Minto => "minto",
            Solver::Mosek => "mosek",
            Solver::Raposa => "raposa",
        }
    }
}
