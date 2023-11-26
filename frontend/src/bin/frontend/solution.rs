use std::collections::HashMap;
use std::str::FromStr;

use cubelib::defs::*;
use cubelib::puzzles::cube::CubeAxis;
use cubelib::steps::step::StepConfig;
use leptonic::prelude::*;
use leptos::*;

#[cfg(feature = "backend")]
pub use backend::SolutionComponent;
#[cfg(feature = "wasm_solver")]
pub use wasm_solver::SolutionComponent;

use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig, SelectableAxis};

#[cfg(feature = "backend")]
pub mod backend {
    use std::cell::RefCell;
    use std::str::FromStr;

    use cubelib::algs::Algorithm;
    use cubelib::puzzles::c333::Turn333;
    use cubelib::solver::solution::Solution;
    use cubelib_interface::{SolverRequest, SolverResponse};
    use ehttp::Request;
    use leptonic::prelude::*;
    use leptos::*;
    use leptos_use::watch_debounced;

    use crate::solution::get_step_configs;
    use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig};

    #[derive(Clone)]
    enum SolutionState {
        NotFetched,
        Requested,
        Found(ehttp::Result<Solution<Turn333>>)
    }

    #[component]
    pub fn SolutionComponent() -> impl IntoView {

        let scramble = Signal::derive(move || Algorithm::<Turn333>::from_str(use_context::<RwSignal<String>>().expect("Scramble context required").get().as_str()).ok());
        let eo = use_context::<EOConfig>().expect("EO context required");
        let rzp = use_context::<RZPConfig>().expect("RZP context required");
        let dr = use_context::<DRConfig>().expect("DR context required");
        let htr = use_context::<HTRConfig>().expect("HTR context required");
        let fr = use_context::<FRConfig>().expect("FR context required");
        let fin = use_context::<FinishConfig>().expect("Finish context required");

        let req_signal = Signal::derive(move||{
            if let Some(alg) = scramble.get() {
                let steps = get_step_configs(eo, rzp, dr, htr, fr, fin);
                Some(SolverRequest {
                    steps: steps.clone(),
                    scramble: alg.to_string()
                })
            } else {
                None
            }
        });

        let prev_req = create_rw_signal::<Option<Option<SolverRequest>>>(None);

        let solution_data = create_rw_signal(SolutionState::NotFetched);
        let is_done_data = create_rw_signal(true);
        let req_id = create_rw_signal(0usize);

        let _ = watch_debounced(move || req_signal.get(), move |req, _, _| {
            let req = req.clone();
            //watch_debounced previous is buggy so we do this
            if let Some(prev) = prev_req.get() {
                if prev == req {
                    return;
                }
            }
            prev_req.set(Some(req.clone()));
            req_id.update(|x| *x = *x + 1);
            solution_data.set(SolutionState::Requested);
            if let Some(req) = req {
                is_done_data.set(false);
                fetch_solution(req.clone(), req_id.get(), solution_data, is_done_data, req_id);
            }
        }, 1000f64);

        view! {
            {move ||
                match solution_data.get() {
                    SolutionState::Found(Ok(s)) => view! {
                        <Code>{format!("{}", s)}</Code>
                    },
                    SolutionState::Found(Err(err)) => view! {
                        <Code>{format!("Error fetching request: {}", err)}</Code>
                    },
                    SolutionState::Requested if is_done_data.get() => view! { //Kind of a hack :(
                        <Code>"No solution found"</Code>
                    },
                    SolutionState::Requested => view! {
                        <Code>"Fetching solution..."</Code>
                    },
                    SolutionState::NotFetched => view! {
                        <Code>"Please enter a scramble"</Code>
                    }
                }
            }
            <div class:hidden=move || is_done_data.get()>
                <ProgressBar progress=create_signal(None).0 />
            </div>
        }
    }

    fn fetch_solution(request: SolverRequest, id: usize, solution_callback: RwSignal<SolutionState>, done_callback: RwSignal<bool>, cur_id: RwSignal<usize>) {
        let current_bytes = RefCell::<Vec<u8>>::new(vec![]);

        let body = serde_json::to_vec(&request).unwrap();
        let mut req = Request::post("http://localhost:8049/solve_stream", body);
        req.headers.insert("content-type".to_string(), "application/json".to_string());

        ehttp::streaming::fetch(req, move |res: ehttp::Result<ehttp::streaming::Part>| {
            let part = match res {
                Ok(part) => part,
                Err(err) => {
                    if cur_id.get_untracked() == id {
                        solution_callback.set(SolutionState::Found(Err(err)));
                        done_callback.set(true)
                    }
                    return std::ops::ControlFlow::Break(());
                }
            };

            match part {
                ehttp::streaming::Part::Response(response) => {
                    if response.ok {
                        std::ops::ControlFlow::Continue(())
                    } else {
                        std::ops::ControlFlow::Break(())
                    }
                }
                ehttp::streaming::Part::Chunk(chunk) => {
                    if cur_id.get_untracked() != id {
                        return std::ops::ControlFlow::Break(());
                    }
                    let mut start = 0;
                    for n in 0..(chunk.len()) {
                        if chunk[n] == b'\n' {
                            let mut val = current_bytes.take();
                            val.extend_from_slice(&chunk[start..n]);
                            match serde_json::from_slice::<SolverResponse>(val.as_slice()) {
                                Ok(res) => {
                                    if let Some(sol) = res.solution {
                                        solution_callback.set(SolutionState::Found(Ok(sol)))
                                    }
                                    if res.done {
                                        done_callback.set(true);
                                    }
                                },
                                Err(err) => {
                                    solution_callback.set(SolutionState::Found(Err(err.to_string())));
                                    done_callback.set(true);
                                },
                            }
                            start = n + 1; //Skip the newline
                        }
                    }
                    current_bytes.borrow_mut().extend_from_slice(&chunk[start..]);
                    std::ops::ControlFlow::Continue(())
                }
            }
        });
    }
}

#[cfg(feature = "wasm_solver")]
pub mod wasm_solver {
    use std::rc::Rc;
    use std::str::FromStr;

    use cubelib::algs::Algorithm;
    use cubelib::puzzles::c333::Cube333;
    use cubelib::puzzles::puzzle::ApplyAlgorithm;
    use gloo_worker::Spawnable;
    use leptonic::prelude::*;
    use leptos::*;
    use leptos_use::watch_debounced;

    use frontend::worker::{FMCSolver, WorkerResponse};

    use crate::solution::get_step_configs;
    use crate::step::{DRConfig, EOConfig, FinishConfig, FRConfig, HTRConfig, RZPConfig};

    #[component]
    pub fn SolutionComponent() -> impl IntoView {

        let scramble = Signal::derive(move || Algorithm::from_str(use_context::<RwSignal<String>>().expect("Scramble context required").get().as_str()).ok());
        let eo = use_context::<EOConfig>().expect("EO context required");
        let rzp = use_context::<RZPConfig>().expect("RZP context required");
        let dr = use_context::<DRConfig>().expect("DR context required");
        let htr = use_context::<HTRConfig>().expect("HTR context required");
        let fr = use_context::<FRConfig>().expect("FR context required");
        let fin = use_context::<FinishConfig>().expect("Finish context required");

        let solver_request = Signal::derive(move || {
            let mut cube = Cube333::default();
            cube.apply_alg(&scramble.get().unwrap_or(Algorithm::new()));
            (cube, get_step_configs(eo, rzp, dr, htr, fr, fin))
        });

        let bridge = FMCSolver::spawner()
            .spawn("./cube/worker.js");

        let bridge = create_rw_signal(Rc::new(bridge));
        let bridge_data = create_rw_signal::<Option<WorkerResponse>>(None);

        watch_debounced(move || solver_request.get(), move |conf, _, _| {
            //TODO check if conf is still valid for the current solution and just skip generating a new one if it is.
            bridge_data.set(None);
            let bridge_handle = FMCSolver::spawner()
                .callback(move |data| {
                    bridge_data.set(Some(data));
                })
                .spawn("./cube/worker.js");
            bridge_handle.send(conf.clone());
            bridge.set(Rc::new(bridge_handle));
        }, 500.0);

        view! {
            {move ||
                match bridge_data.get() {
                    Some(WorkerResponse::Solved(s)) => view! {
                        <Code>{format!("{s}")}</Code>
                    },
                    Some(WorkerResponse::NoSolution) => view! {
                        <Code>{format!("No solution")}</Code>
                    },
                    Some(WorkerResponse::InvalidStepConfig) => view! {
                        <Code>{format!("Invalid step config (this is a bug)")}</Code>
                    },
                    Some(WorkerResponse::UnknownError) => view! {
                        <Code>{format!("Unknown error")}</Code>
                    },
                    None => view! {
                        <Code>"Calculating solution..."</Code>
                    }
                }
            }
        }
    }
}

fn get_step_configs(eo: EOConfig, rzp: RZPConfig, dr: DRConfig, htr: HTRConfig, fr: FRConfig, fin: FinishConfig) -> Vec<StepConfig> {
    let mut steps_config = vec![];
    if eo.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::EO,
            substeps: Some(variants_to_string(eo.variants.get())),
            min: Some(eo.min.get()),
            max: Some(eo.max.get()),
            absolute_min: None,
            absolute_max: None,
            step_limit: None,
            quality: 10000,
            niss: Some(eo.niss.get()),
            params: Default::default(),
        });
    }
    if dr.enabled.0.get() {
        if dr.triggers.get().len() > 0 {
            steps_config.push(StepConfig {
                kind: StepKind::RZP,
                substeps: None,
                min: Some(rzp.min.get()),
                max: Some(rzp.max.get()),
                absolute_min: None,
                absolute_max: None,
                step_limit: None,
                quality: 10000,
                niss: Some(rzp.niss.get()),
                params: Default::default(),
            });
            let mut triggers = HashMap::new();
            triggers.insert("triggers".to_string(), dr.triggers.get().join(","));
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: Some(variants_to_string(dr.variants.get())),
                min: Some(dr.min.get()),
                max: Some(dr.max.get()),
                absolute_min: None,
                absolute_max: None,
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.get()),
                params: triggers,
            });
        } else {
            steps_config.push(StepConfig {
                kind: StepKind::DR,
                substeps: Some(variants_to_string(dr.variants.get())),
                min: Some(dr.min.get()),
                max: Some(dr.max.get()),
                absolute_min: None,
                absolute_max: None,
                step_limit: None,
                quality: 10000,
                niss: Some(dr.niss.get()),
                params: Default::default(),
            });
        }
    }
    if htr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::HTR,
            substeps: Some(variants_to_string(htr.variants.get())),
            min: Some(htr.min.get()),
            max: Some(htr.max.get()),
            absolute_min: None,
            absolute_max: None,
            step_limit: None,
            quality: 10000,
            niss: Some(htr.niss.get()),
            params: Default::default(),
        });
    }
    if fr.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: if fin.leave_slice.get() {
                StepKind::FRLS
            } else {
                StepKind::FR
            },
            substeps: Some(variants_to_string(fr.variants.get())),
            min: Some(fr.min.get()),
            max: Some(fr.max.get()),
            absolute_min: None,
            absolute_max: None,
            step_limit: None,
            quality: 10000,
            niss: Some(fr.niss.get()),
            params: Default::default(),
        });
    }
    if fin.enabled.0.get() {
        steps_config.push(StepConfig {
            kind: StepKind::FIN,
            substeps: Some(vec!["ud".to_string(), "fb".to_string(), "lr".to_string()]),
            min: Some(fin.min.get()),
            max: Some(fin.max.get()),
            absolute_min: None,
            absolute_max: None,
            step_limit: None,
            quality: 10000,
            niss: Some(NissSwitchType::Never),
            params: Default::default(),
        });
    }
    steps_config
}

fn variants_to_string(variants: Vec<CubeAxis>) -> Vec<String> {
    variants.into_iter()
        .map(|a| Into::<SelectableAxis>::into(a).to_string())
        .collect()
}