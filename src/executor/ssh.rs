use std::{
    borrow::Cow,
    fs::File,
    io::{copy, Read},
    net::TcpStream,
    path::Path,
};

use anyhow::{anyhow, bail, Result};
use ssh2::Session;

use crate::{
    config::{Action, Host, Step},
    executor::StepResult,
};

impl Step {
    pub fn ssh(&self, host: &Host, work_dir: &str) -> Result<StepResult> {
        let stream = TcpStream::connect((host.hostname.as_str(), host.port.unwrap_or(22)))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(stream);
        session.handshake()?;
        let private_key = {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow!("\"HOME\" environment variable is required"))?;
            let home = Path::new(&home);
            let ed25519 = home.join(".ssh/id_ed25519");
            let rsa = home.join(".ssh/id_rsa");
            if ed25519.exists() {
                ed25519
            } else if rsa.exists() {
                rsa
            } else {
                bail!("missing private key");
            }
        };
        session.userauth_pubkey_file(&host.user, None, &private_key, None)?;

        let remote_filename = format!("/tmp/delivery_station_{}", crate::tmp_filename(12));
        let (name, args) = match &self.action {
            Action::Script { name } => {
                let script_name = self.get_script_fullname(work_dir, name.get_name())?;
                let mut file = File::open(script_name)?;
                let metadata = file.metadata()?;
                let mut remote_file =
                    session.scp_send(Path::new(&remote_filename), 0o755, metadata.len(), None)?;
                copy(&mut file, &mut remote_file)?;
                (remote_filename.as_str(), name.get_args())
            }
            Action::Command { command } => (command.get_name(), command.get_args()),
        };
        let remote_cmd = match args {
            None => Cow::Borrowed(name),
            Some(args) => format!("{} \"{}\"", name, args.join("\" \"")).into(),
        };
        let mut channel = session.channel_session()?;
        channel.exec(&remote_cmd)?;

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;
        let mut stderr = String::new();
        let mut stderr_stream = channel.stderr();
        stderr_stream.read_to_string(&mut stderr)?;
        channel.wait_close()?;
        let status = channel.exit_status()?;

        if let Action::Script { .. } = &self.action {
            let mut channel = session.channel_session()?;
            channel.exec(&format!("rm {}", remote_filename))?;
            channel.close()?;
        }

        let result = StepResult::new(
            status,
            Some(stdout.as_bytes().to_vec()),
            Some(stderr.as_bytes().to_vec()),
        );
        Ok(result)
    }
}
