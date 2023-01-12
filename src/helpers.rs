use std::collections::HashMap;

pub fn make_params(params: Option<&HashMap<&str, String>>) -> String {
	if params.is_none() {
		return String::new();
	}
	let params = params.unwrap();
	let mut res = String::from("?");
	let kvs = params
		.iter()
		.map(|(key, val)| format!("{}={}", key.to_lowercase(), val))
		.collect::<Vec<String>>();

	if kvs.len() == 0 {
		return String::new();
	}

	res.push_str(&kvs.join("&"));
	return res;
}

#[macro_export]
macro_rules! with_param {
	($param:ident, $_type: ty) => {
		pub fn $param(mut self, $param: $_type) -> Self {
			self.params.insert(stringify!($param), $param.to_string());
			self
		}
	};
}
