use leptos::*;

pub fn build_toggle_chain<const L: usize>() -> Vec<(Signal<bool>, Callback<bool>)> {
    let signals: Vec<RwSignal<bool>> = (0..L).into_iter()
        .map(|_| create_rw_signal(true))
        .collect();
    let mut callbacks = vec![];
    for i in 0..signals.len() {
        let signals_0 = signals.clone();
        let signals_1 = signals.clone();
        callbacks.push((Signal::derive(move || signals_0[i].get()), Callback::new(move |state| {
            if state {
                for j in 0..=i {
                    signals_1[j].set(true);
                }
            } else {
                for j in i..signals_1.len() {
                    signals_1[j].set(false);
                }
            }
        })));
    }
    callbacks
}