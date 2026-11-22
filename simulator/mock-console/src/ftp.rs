//! Simulated FTP service (in-memory filesystem). Honours the scenario's behaviour so clients can
//! be tested against auth failure, refused writes, full storage and dropped transfers.

use crate::{scenario, Behavior};
use ps4studio_core::capability::FtpFact;
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Default)]
pub struct SimFs {
    pub files: Mutex<BTreeMap<String, Vec<u8>>>,
    pub dirs: Mutex<BTreeSet<String>>,
}

impl SimFs {
    fn seeded() -> Self {
        let fs = SimFs::default();
        for d in ["/", "/data", "/data/pkg", "/data/homebrew", "/user", "/system"] {
            fs.dirs.lock().unwrap().insert(d.into());
        }
        fs.files.lock().unwrap().insert("/data/homebrew/readme.txt".into(), b"simulated file".to_vec());
        fs.files.lock().unwrap().insert("/system/sys.dat".into(), vec![0u8; 16]);
        fs
    }

    fn children(&self, dir: &str) -> Vec<String> {
        let prefix = if dir == "/" { "/".to_string() } else { format!("{dir}/") };
        let direct = |p: &str| p.strip_prefix(&prefix).is_some_and(|r| !r.is_empty() && !r.contains('/'));
        let mut out = Vec::new();
        for d in self.dirs.lock().unwrap().iter().filter(|p| direct(p)) {
            out.push(format!("drwxr-xr-x 2 sim sim 0 Jan  1 00:00 {}", &d[prefix.len()..]));
        }
        for (f, c) in self.files.lock().unwrap().iter().filter(|(p, _)| direct(p)) {
            out.push(format!("-rw-r--r-- 1 sim sim {} Jan  1 00:00 {}", c.len(), &f[prefix.len()..]));
        }
        out
    }
}

pub async fn serve_ftp(scenario_id: &str, bind: SocketAddr) -> std::io::Result<(SocketAddr, tokio::task::JoinHandle<()>, Arc<SimFs>)> {
    let sc = scenario(scenario_id).ok_or_else(|| std::io::Error::other("unknown scenario"))?;
    let l = TcpListener::bind(bind).await?;
    let addr = l.local_addr()?;
    let fs = Arc::new(SimFs::seeded());
    let fs2 = fs.clone();
    let h = tokio::spawn(async move {
        loop {
            let Ok((s, _)) = l.accept().await else { return };
            let (fs, behavior, auth_fail) = (fs2.clone(), sc.behavior.clone(), sc.facts.ftp == FtpFact::AuthFailed);
            tokio::spawn(async move {
                let _ = session(s, fs, behavior, auth_fail).await;
            });
        }
    });
    Ok((addr, h, fs))
}

async fn session(s: TcpStream, fs: Arc<SimFs>, behavior: Behavior, auth_fail: bool) -> std::io::Result<()> {
    let local_ip = s.local_addr()?.ip();
    let (rd, mut wr) = s.into_split();
    let mut rd = BufReader::new(rd);
    wr.write_all(b"220 [SIMULATED] FTP service ready\r\n").await?;
    let mut pasv: Option<TcpListener> = None;
    let mut rnfr: Option<String> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if rd.read_line(&mut line).await? == 0 {
            return Ok(());
        }
        let t = line.trim_end();
        let (cmd, arg) = t.split_once(' ').unwrap_or((t, ""));
        let writes_refused = behavior == Behavior::UnauthorizedWrite;
        let reply: String = match cmd.to_ascii_uppercase().as_str() {
            "USER" => "331 password please".into(),
            "PASS" if auth_fail => "530 login incorrect".into(),
            "PASS" => "230 logged in".into(),
            "SYST" => "215 UNIX Type: L8 [SIMULATED]".into(),
            "TYPE" => "200 ok".into(),
            "PWD" => "257 \"/\"".into(),
            "QUIT" => {
                wr.write_all(b"221 bye\r\n").await?;
                return Ok(());
            }
            "PASV" => {
                let l = TcpListener::bind((local_ip, 0)).await?;
                let p = l.local_addr()?.port();
                pasv = Some(l);
                let ip = local_ip.to_string().replace('.', ",");
                format!("227 Entering Passive Mode ({ip},{},{})", p / 256, p % 256)
            }
            "LIST" | "RETR" | "STOR" => {
                let Some(l) = pasv.take() else {
                    wr.write_all(b"425 use PASV first\r\n").await?;
                    continue;
                };
                let up = cmd.to_ascii_uppercase();
                if up == "STOR" && writes_refused {
                    wr.write_all(b"550 permission denied\r\n").await?;
                    continue;
                }
                if up == "RETR" && !fs.files.lock().unwrap().contains_key(arg) {
                    wr.write_all(b"550 no such file\r\n").await?;
                    continue;
                }
                wr.write_all(b"150 opening data connection\r\n").await?;
                let (mut data, _) = l.accept().await?;
                match up.as_str() {
                    "LIST" => {
                        let dir = if arg.is_empty() { "/" } else { arg };
                        for l in fs.children(dir) {
                            data.write_all(format!("{l}\r\n").as_bytes()).await?;
                        }
                    }
                    "RETR" => {
                        let content = fs.files.lock().unwrap().get(arg).cloned().unwrap_or_default();
                        if let Behavior::TransferInterrupts { after_bytes } = behavior {
                            data.write_all(&content[..content.len().min(after_bytes as usize)]).await?;
                            return Ok(()); // drop both connections
                        }
                        data.write_all(&content).await?;
                    }
                    _ => {
                        let mut buf = Vec::new();
                        let mut chunk = [0u8; 8192];
                        loop {
                            let n = data.read(&mut chunk).await?;
                            if n == 0 {
                                break;
                            }
                            buf.extend_from_slice(&chunk[..n]);
                            if let Behavior::TransferInterrupts { after_bytes } = behavior {
                                if buf.len() as u64 >= after_bytes {
                                    return Ok(());
                                }
                            }
                        }
                        if let Behavior::StorageLimit { free_bytes } = behavior {
                            if buf.len() as u64 > free_bytes {
                                drop(data);
                                wr.write_all(b"552 insufficient storage\r\n").await?;
                                continue;
                            }
                        }
                        fs.files.lock().unwrap().insert(arg.to_owned(), buf);
                    }
                }
                data.shutdown().await?;
                "226 transfer complete".into()
            }
            "SIZE" => match fs.files.lock().unwrap().get(arg) {
                Some(c) => format!("213 {}", c.len()),
                None => "550 no such file".into(),
            },
            "MKD" | "RMD" | "DELE" | "RNFR" | "RNTO" if writes_refused => "550 permission denied".into(),
            "MKD" => {
                fs.dirs.lock().unwrap().insert(arg.to_owned());
                format!("257 \"{arg}\" created")
            }
            "RMD" => {
                if fs.dirs.lock().unwrap().remove(arg) { "250 removed".into() } else { "550 no such directory".into() }
            }
            "DELE" => {
                if fs.files.lock().unwrap().remove(arg).is_some() { "250 deleted".into() } else { "550 no such file".into() }
            }
            "RNFR" => {
                rnfr = Some(arg.to_owned());
                "350 ready for RNTO".into()
            }
            "RNTO" => match rnfr.take() {
                Some(from) => {
                    let mut f = fs.files.lock().unwrap();
                    match f.remove(&from) {
                        Some(c) => {
                            f.insert(arg.to_owned(), c);
                            "250 renamed".into()
                        }
                        None => "550 no such file".into(),
                    }
                }
                None => "503 RNFR first".into(),
            },
            _ => "502 not implemented".into(),
        };
        wr.write_all(format!("{reply}\r\n").as_bytes()).await?;
    }
}
