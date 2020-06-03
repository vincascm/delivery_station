mod config;
pub mod constants;
pub mod executor;
pub mod http;
pub mod notifier;
pub mod trigger;

fn tmp_filename(len: usize) -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}
