use hbb_common::{bail, log, ResultType};
use serde_derive::{Deserialize, Serialize};
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct Machine {
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    mac: String,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct Post {
    #[serde(default)]
    machine: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    next_check_time: u32,
}

const LICENSE_FILE: &'static str = ".license.txt";

pub fn check_lic(email: &str, version: &str) -> bool {
    if email.is_empty() {
        log::error!("Registered email required (-m option). Please visit https://rustdesk.com/server for more infomration.");
        return false;
    }

    let machine = get_lic();
    let path = Path::new(LICENSE_FILE);
    if Path::is_file(&path) {
        let contents = std::fs::read_to_string(&path).unwrap_or("".to_owned());
        if verify(&contents, &machine) {
            return true;
        }
    }

    match check_email(machine, email.to_owned(), version.to_owned()) {
        Ok(v) => {
            return true;
        }
        Err(err) => {
            log::error!("{}", err);
            return false;
        }
    }
}

fn write_lic(lic: &str) {
    if let Ok(mut f) = std::fs::File::create(LICENSE_FILE) {
        f.write_all(lic.as_bytes()).ok();
        f.sync_all().ok();
    }
}

fn check_email(machine: String, email: String, version: String) -> ResultType<u32> {
    log::info!("Checking email with the server ...");
    let resp = minreq::post("http://rustdesk.com/api/check-email")
        .with_body(
            serde_json::to_string(&Post {
                machine: machine.clone(),
                version,
                email,
                ..Default::default()
            })
            .unwrap(),
        )
        .send()?;
    if resp.reason_phrase == "OK" {
        let p: Post = serde_json::from_str(&resp.as_str()?)?;
        if !p.status.is_empty() {
            bail!("{}", p.status);
        }
        if !verify(&p.machine, &machine) {
            bail!("Verification failure");
        }
        write_lic(&p.machine);
        Ok(p.next_check_time)
    } else {
        bail!("Server error: {}", resp.reason_phrase);
    }
}

fn get_lic() -> String {
    let hostname = whoami::hostname();
    let uid = machine_uid::get().unwrap_or("".to_owned());
    let mac = if let Ok(Some(ma)) = mac_address::get_mac_address() {
        base64::encode(ma.bytes())
    } else {
        "".to_owned()
    };
    serde_json::to_string(&Machine { hostname, uid, mac }).unwrap()
}

fn verify(enc_str: &str, msg: &str) -> bool {
    if let Ok(data) = base64::decode(enc_str) {
        let key =
            b"\xf1T\xc0\x1c\xffee\x86,S*\xd9.\x91\xcd\x85\x12:\xec\xa9 \x99:\x8a\xa2S\x1f Yy\x93R";
        cryptoxide::ed25519::verify(msg.as_bytes(), &key[..], &data)
    } else {
        false
    }
}

pub const EMAIL_ARG: &'static str =
    "-m, --email=[EMAIL] 'Sets your email address registered with RustDesk'";
