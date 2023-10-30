use std::vec;

use cubelib::cubie::CubieCube;
use cubelib::defs::*;
use cubelib::solution::Solution;
use cubelib::steps::step::{DefaultStepOptions, Step, StepConfig};
use cubelib::tables::PruningTables;
use itertools::Itertools;
use log::{debug, info, warn};

pub fn gen_tables(steps: &Vec<StepConfig>, mut tables: &mut PruningTables) {
    let previous = vec![None].into_iter().chain(steps.iter().map(|x|Some(x.kind))).collect_vec();
    let steps = steps.into_iter().zip(previous.into_iter()).collect_vec();

    for (conf, pre) in steps.iter() {
        match (pre.clone(), conf.kind.clone()) {
            (_, StepKind::EO) => tables.gen_eo(),
            (_, StepKind::DR) => tables.gen_dr(),
            (_, StepKind::HTR) => tables.gen_htr(),
            (_, StepKind::FR) => tables.gen_fr(),
            (_, StepKind::FRLS) => tables.gen_fr_leave_slice(),
            (Some(StepKind::FR), StepKind::FIN) | (Some(StepKind::FRLS), StepKind::FIN) => tables.gen_fr_finish(),
            _ => ()
        }
    }
}

pub fn build_steps(steps: Vec<StepConfig>, tables: &PruningTables) -> Result<Vec<(Step<CubieCube>, DefaultStepOptions)>, String> {
    let previous = vec![None].into_iter().chain(steps.iter().map(|x|Some(x.kind))).collect_vec();
    let steps = steps.into_iter().zip(previous.into_iter()).collect_vec();

    let steps = steps.into_iter()
        .flat_map(|(config, previous)| match (previous, config.kind) {
            #[cfg(feature = "eo")]
            (None, StepKind::EO) => vec![eo::eo_config::from_step_config::<CubieCube>(tables.eo().expect("EO table required"), config)].into_iter(),
            #[cfg(feature = "dr")]
            (Some(StepKind::EO), StepKind::RZP)   => vec![dr::rzp_config::from_step_config::<CubieCube>(config)].into_iter(),
            #[cfg(feature = "dr")]
            (Some(StepKind::EO), StepKind::DR) => {
                let dr_table = tables.dr().expect("DR table required");
                if config.params.contains_key("triggers") {
                    info!("Found explicitly defined DR triggers without RZP. Adding RZP step with default settings.");
                    let mut rzp_config = StepConfig::new(StepKind::RZP);
                    rzp_config.quality = config.quality;
                    vec![dr::rzp_config::from_step_config(rzp_config), dr::dr_trigger_config::from_step_config(dr_table, config)].into_iter()
                } else {
                    vec![dr::dr_config::from_step_config(dr_table, config)].into_iter()
                }
            }
            #[cfg(feature = "dr")]
            (Some(StepKind::RZP), StepKind::DR) => {
                let dr_table = tables.dr().expect("DR table required");
                if !config.params.contains_key("triggers") {
                    warn!("RZP without defining triggers is pointless and slower. Consider deleting the RZP step or adding explicit DR triggers.");
                    vec![dr::dr_config::from_step_config::<CubieCube>(dr_table, config)].into_iter()
                } else {
                    vec![dr::dr_trigger_config::from_step_config(dr_table, config)].into_iter()
                }
            }
            #[cfg(feature = "htr")]
            (Some(StepKind::DR), StepKind::HTR)   => vec![htr::htr_config::from_step_config::<CubieCube>(tables.htr().expect("HTR table required"), config)].into_iter(),
            #[cfg(feature = "fr")]
            (Some(StepKind::HTR), StepKind::FR)   => vec![fr::fr_config::from_step_config::<CubieCube>(tables.fr().expect("FR table required"), config)].into_iter(),
            #[cfg(feature = "fr")]
            (Some(StepKind::HTR), StepKind::FRLS)  => vec![fr::fr_config::from_step_config_no_slice::<CubieCube>(tables.fr_leave_slice().expect("FRLeaveSlice table required"), config)].into_iter(),
            #[cfg(feature = "finish")]
            (Some(StepKind::FR), StepKind::FIN)   => vec![finish::finish_config::from_step_config_fr::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config)].into_iter(),
            #[cfg(feature = "finish")]
            (Some(StepKind::FRLS), StepKind::FIN)   => vec![finish::finish_config::from_step_config_fr_leave_slice::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config)].into_iter(),
            (None, x) => vec![Err(format!("{:?} is not supported as a first step", x))].into_iter(),
            (Some(x), y) => vec![Err(format!("Unsupported step order {:?} > {:?}", x, y))].into_iter(),
        })
        .collect();
    steps
}

pub fn solve_steps<'a>(cube: CubieCube, steps: &'a Vec<(Step<'a, CubieCube>, DefaultStepOptions)>) -> impl Iterator<Item = Solution> + 'a {
    let first_step: Box<dyn Iterator<Item = Solution>> = Box::new(vec![Solution::new()].into_iter());

    let solutions: Box<dyn Iterator<Item=Solution>> = steps.iter()
        .fold(first_step, |acc, (step, search_opts)|{
            debug!("Step {} with options {:?}", step.kind(), search_opts);
            let next = step::next_step(acc, step, search_opts.clone(), cube.clone())
                .zip(0..)
                .take_while(|(sol, count)| search_opts.step_limit.map(|limit| limit > *count).unwrap_or(true))
                .map(|(sol, _)|sol);
            Box::new(next)
        });

    solutions
}

pub struct SolutionIterator<'a> {
    steps: Vec<(Step<'a, CubieCube>, DefaultStepOptions)>,
    solutions: Box<dyn Iterator<Item=Solution>>,
}

impl Iterator for SolutionIterator<'_> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.solutions.next()
    }
}