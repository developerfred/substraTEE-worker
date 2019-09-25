extern crate regex;

use log::*;
use codec::{Decode, Encode};
use substrate_api_client::utils::{hexstr_to_vec};
use primitives::ed25519;
use regex::Regex;
use srml_support::traits::Len;

pub fn get_worker_info(api: &substrate_api_client::Api, index: u64) -> Enclave {
	info!("[>] Get worker's URL");
	let result_str = api.get_storage("substraTEERegistry", "EnclaveRegistry", Some((index).encode())).unwrap();
	debug!("Storage hex_str: {}", result_str);

	let enc = hexstr_to_enclave(result_str);
	info!("[+]: W{} Pubkey is {:?}", index, &enc.pubkey);
	info!("[+]: W{} URL is {:?}", index, enc.url);
	enc
}

pub fn get_worker_amount(api: &substrate_api_client::Api) -> u64 {
	let result_str = api.get_storage("substraTEERegistry", "EnclaveCount", None).unwrap();
	info!("get_worker_amount() ret {:?}", result_str);
	let amount = match result_str.as_str() {
		"null" => 0u64,
		_ => hexstr_to_u64(result_str),
	};
	info!("[+]: Amount of Registered Workers {:?}", amount);
	amount
}

pub fn get_latest_state(api: &substrate_api_client::Api) -> Option<[u8; 46]> {
	let result_str = api.get_storage("substraTEERegistry", "LatestIPFSHash", None).unwrap();
	let unhex = hexstr_to_vec(result_str).unwrap();
	info!("State hash vec: {:?}", unhex);
	let mut h : [u8; 46] = [0; 46];

	match unhex.len() {
		1 => {
			info!("No state update happened yet");
			None
		},
		_ => {
			h.clone_from_slice(&unhex);
			Some(h)
		}
	}
}

fn hexstr_to_enclave(hexstr: String) -> Enclave {
	let mut unhex = hexstr_to_vec(hexstr).unwrap();
	let (h, url) = unhex.split_at_mut(32 as usize);
	let mut raw: [u8; 32] = Default::default();
	raw.copy_from_slice(&h);
	let key = ed25519::Public::from_raw(raw);

	let url_str = std::str::from_utf8(&url[1..]).unwrap();
	let re = Regex::new("[0-9]{1,3}[.][0-9]{1,3}[.][0-9]{1,3}[.][0-9]{1,3}[:][0-9]{4}").unwrap();
	let m = re.find(url_str).unwrap();
	Enclave {
		pubkey: key,
		// Fixme: There are some bytes left that contain metadata about the linkable map.
		// This may be the reason I was not able to do automated deserialization.
		url: url_str[m.start()..m.end()].to_string(),
	}
}

pub fn hexstr_to_u64(hexstr: String) -> u64 {
	let unhex = hexstr_to_vec(hexstr).unwrap();
	let mut gh: [u8; 8] = Default::default();
	gh.copy_from_slice(&unhex);

	u64::from_le_bytes(gh)
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Enclave {
	pub pubkey: ed25519::Public,
	// utf8 encoded url
	pub url: String
}

#[cfg(test)]
mod tests {
	use substrate_api_client::Api;

	use super::*;

	#[test]
	// test requires one registered enclave in substratee_registry
	fn get_worker_enclave_should_work() {
		let mut api: substrate_api_client::Api = Api::new(format!("ws://127.0.0.1:9991"));
		api.init();
		get_worker_amount(&api);
		let enc = get_worker_info(&api, 0);

	}

	#[test]
	fn regex_works() {
		let url = "1192.168.10.21:9111askdfhkajsd";
		let re = Regex::new("[0-9]{1,3}.[0-9]{1,3}.[0-9]{1,3}.[0-9]{1,3}:[0-9]{4}").unwrap();

		println!("Regex {}", re.as_str());
		let m = re.find(url).unwrap();

		assert_eq!("192.168.10.21:9111", &url[m.start()..m.end()])
	}
}

