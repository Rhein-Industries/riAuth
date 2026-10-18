//! Bounded subprocess capture shared by the standalone client integration tests.
use std::{
    io::{self, Read, Write},
    process::{Child, Command, Output},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

fn drain(mut stream: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stream.read_to_end(&mut bytes).map(|_| bytes);
        let _ = sender.send(result);
    });
    receiver
}

fn timeout(child: &mut Child, args: &[&str]) -> ! {
    let _ = child.kill();
    let _ = child.wait();
    // Do not join pipe readers here: a descendant may still own a writer.
    // The test must retain its deadline even in that case.
    panic!("riauthctl timed out for {args:?}");
}

pub fn output(command: &mut Command, input: Option<&str>, args: &[&str]) -> Output {
    let mut child = command.spawn().expect("start riauthctl");
    let deadline = Instant::now() + Duration::from_secs(15);
    // Read both pipes while the process runs. Waiting for exit first can
    // deadlock even a small JSON response on systems with small pipe buffers.
    let stdout = drain(child.stdout.take().expect("piped stdout"));
    let stderr = drain(child.stderr.take().expect("piped stderr"));
    let mut stdin = child.stdin.take().expect("piped stdin");
    if let Some(input) = input {
        let input = input.as_bytes().to_vec();
        thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
    } else {
        drop(stdin);
    }
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait for riauthctl") {
            break status;
        }
        if Instant::now() >= deadline {
            timeout(&mut child, args);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let mut collect = |receiver: Receiver<io::Result<Vec<u8>>>| match receiver
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        Ok(result) => result.expect("capture riauthctl output"),
        Err(_) => timeout(&mut child, args),
    };
    Output {
        status,
        stdout: collect(stdout),
        stderr: collect(stderr),
    }
}
