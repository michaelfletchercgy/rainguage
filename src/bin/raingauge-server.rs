extern crate raingauge;
extern crate libc;

use std::fs::File;
use std::fs::OpenOptions;

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use std::os::unix::io::AsRawFd;

use std::path::Path;

use std::thread::spawn;
use std::thread::sleep;

use std::time::Duration;

use libc::epoll_event;
use libc::epoll_create;
use libc::epoll_ctl;
use libc::epoll_wait;
use libc::close;

use libc::EPOLLPRI;
use libc::EPOLLERR;
use libc::EPOLL_CTL_ADD;

struct Gpio {
    epoll_fd: libc::c_int,
    file: File
}

impl Gpio {
    fn new(port: u32) -> Gpio {
        let path_str = format!("/sys/class/gpio/gpio{}/value", port);
        let path = Path::new(&path_str);

        if !path.exists() {
            let mut exp = OpenOptions::new().write(true).open("/sys/class/gpio/export").unwrap();
            writeln!(exp, "{}", port).unwrap();
            exp.flush().unwrap();
        }

        let edge_str = format!("/sys/class/gpio/gpio{}/edge", port);
        let mut edge = OpenOptions::new().write(true).open(edge_str).unwrap(); 
        write!(edge, "both").unwrap();
        edge.flush().unwrap();

        let file = File::open(path).unwrap();

        unsafe {
            let epollfd = epoll_create(1);

            let mut event = epoll_event {
                u64: 0,
                events: EPOLLPRI as u32 | EPOLLERR as u32
            };

            let epoll_ctl_result = epoll_ctl(epollfd, EPOLL_CTL_ADD, file.as_raw_fd(), 
                &mut event as *mut epoll_event);

            if epoll_ctl_result != 0 {
                panic!("epoll_ctl fail {}", epoll_ctl_result);
            }

            Gpio {
                epoll_fd:epollfd,
                file:file
            }
        }
    }

    fn next_value(&mut self) -> bool {
        let mut events = Vec::<epoll_event>::with_capacity(1);
        unsafe {
            events.set_len(1);
            let mut buffer = vec![0; 1];


            println!("about to wait");
            let num_ready = epoll_wait(self.epoll_fd, events.as_mut_ptr(), 5, -1);

            if num_ready < 0 {
                panic!("epoll_wait fail {}", num_ready);
            }

            self.file.read(&mut buffer).unwrap();
            let result = buffer[0] == b'1';

            self.file.seek(SeekFrom::Start(0)).unwrap();   
            
            result
        }
    }

}

impl Drop for Gpio {
    fn drop(&mut self) {
        unsafe {
            close(self.epoll_fd);
        }
    }
}

struct NopOutput {
    val:bool
}

impl raingauge::Output for NopOutput {
    fn value(&mut self, val:bool) {
        sleep(Duration::from_secs(2));
        self.val = val;        
    }
}


pub fn main() {
    let f = OpenOptions::new().write(true).open("/dev/ttyAMA0").unwrap();

    let mut tipper_gpio = Gpio::new(24);
    let tx_pwr = NopOutput { val:false };
    let rg = raingauge::RainGauge::new(tx_pwr, f);

    let timeout_rg = rg.clone();
    spawn(move|| {
            loop {
                sleep(Duration::from_secs(2));

                timeout_rg.transmit_timeout();
            }
        });

    loop {
        let value = tipper_gpio.next_value();
        rg.tip(value);
    }
    //rg.stop();
}
