// use std::fs::File;
// use std::os::fd::AsRawFd;
// use std::os::unix::net::UnixStream;
// use std::io::{Read, Write};

// use nix::sys::socket::{ControlMessage, MsgFlags};

// fn main() {
//     let socket_path = "/tmp/hypr_opengl.sock";
//     let mut stream = UnixStream::connect(socket_path).expect("failed to connect");
//     println!("Connected to receiver");
//     stream.write_all(b"hello from sender").unwrap();
//     stream.flush().unwrap();
//     println!("Sent");
//     let mut buf = [0u8; 1024];
//     let n = stream.read(&mut buf).unwrap();
//     println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
//     let stream_fd = stream.as_raw_fd();
//     let file = File::open("some_spooky_file.txt").unwrap();
//     let file_fd = file.as_raw_fd();
//     let iov = [std::io::IoSlice::new(&[0u8])];
//     nix::sys::socket::sendmsg::<()>(stream_fd, &iov, &[ControlMessage::ScmRights(&[file_fd])], MsgFlags::empty(), None).unwrap();

//     let mut buf = [0u8; 1024];
//     let n = stream.read(&mut buf).unwrap();
//     println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
// }
// 🛰️
