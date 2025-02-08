pub fn setup_logger() {
	#[cfg(debug_assertions)]
	let log_level = log::LevelFilter::Debug;
	#[cfg(not(debug_assertions))]
	let log_level = log::LevelFilter::Info;

	env_logger::builder()
		.target(env_logger::Target::Stdout)
		.filter_level(log_level)
		.init();
}
