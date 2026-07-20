use keyring::Entry;

const SERVICE: &str = "jarvis";
const ACCOUNT: &str = "cloud_api_key";

fn entry() -> Result<Entry, keyring::Error> {
    Entry::new(SERVICE, ACCOUNT)
}

pub fn get_api_key() -> Result<String, keyring::Error> {
    entry()?.get_password()
}

pub fn set_api_key(key: &str) -> Result<(), keyring::Error> {
    entry()?.set_password(key)
}

pub fn delete_api_key() -> Result<(), keyring::Error> {
    entry()?.delete_credential()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires OS credential store"]
    fn roundtrips_api_key() {
        let test_key = "sk-test-roundtrip";
        let _ = delete_api_key();
        set_api_key(test_key).expect("set");
        assert_eq!(get_api_key().expect("get"), test_key);
        delete_api_key().expect("delete");
    }
}
