mod reserve_transfer;

#[ctor::ctor]
fn init() {
	env_logger::builder().format_timestamp(None).init();
}
