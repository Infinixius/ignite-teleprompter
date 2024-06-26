// Logger macro
#[macro_export]
macro_rules! log {
	($($arg:tt)*) => {
		println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), format!($($arg)*))
	}
}