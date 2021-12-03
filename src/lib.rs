mod config;
pub mod constants;
pub mod executor;
pub mod http;
pub mod notifier;
pub mod trigger;

fn tmp_filename(len: usize) -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    let mut rng = thread_rng();
    let s: Vec<_> = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect();
    String::from_utf8_lossy(&s).to_string()
}
