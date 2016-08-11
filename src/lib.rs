
extern crate sqlite;

use sqlite::Connection;
use sqlite::State;

use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;

pub trait Output {
    fn value(&mut self, new_value:bool);
}

#[derive(Clone)]
pub struct RainGauge {
    tx: Sender<Msg>
}

enum Msg {
    Tipper(bool),
    Timeout,
    Stop
}

impl RainGauge {
    pub fn new<P:'static+Output+Send,R: 'static+Write+Send>(mut transmit_power:P, mut radio_tx:R) -> RainGauge {
        let (tx,rx) = channel();
        
        thread::spawn(move|| {
            let mut tips = 0;
            let mut last_value:Option<bool> = None;

            let conn = Connection::open("data.sqlite").unwrap();

            let mut stmt = conn.prepare("
                    SELECT 1 
                    FROM   sqlite_master 
                    WHERE  type='table' and name='measurement'
                ").unwrap();

            let mut has_table = false;
            while let State::Row = stmt.next().unwrap() {
                has_table = true;
            }

            if !has_table {
                conn.execute("CREATE TABLE measurement (
                              id              INTEGER PRIMARY KEY,
                              time_created    TEXT NOT NULL,
                              value           I
                              )").unwrap();
            }

            let mut sum_stmt = conn.prepare("SELECT SUM(value) FROM measurement").unwrap();
            while let State::Row = sum_stmt.next().unwrap() {
                tips = sum_stmt.read::<i64>(0).unwrap();
            }


            loop {
                let mut xmit = false;

                match rx.recv().unwrap() {
                    Msg::Tipper(x) => {
                        if last_value.is_some() {
                            let last_val = last_value.unwrap();

                            if x != last_val && x {
                                tips = tips + 1;

                                let dur = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
                                let ts = dur.as_secs();
                                let sql = format!("INSERT INTO measurement (time_created, value) VALUES ({}, 1)", ts);
                                println!("sql={}", sql);
                                conn.execute(sql).unwrap();
                                xmit = true;
                            }
                        }

                        last_value = Some(x);
                    },
                    Msg::Stop => {
                        break;
                    },
                    Msg::Timeout => {
                        xmit = true;
                    }
                };

                if xmit {
                    write!(radio_tx, "TIPS={}", tips).unwrap();
                    transmit_power.value(false);
                }
            }
        });

        RainGauge {
            tx: tx
        }
    }

    pub fn tip(&self, value:bool) {
        self.tx.send(Msg::Tipper(value)).unwrap();
    }

    pub fn transmit_timeout(&self) {
        self.tx.send(Msg::Timeout).unwrap();
    }

    pub fn stop(&self) {
        self.tx.send(Msg::Stop).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use Output;
    use RainGauge;
    use std::sync::mpsc::Sender;
    use std::sync::mpsc::Receiver;
    use std::sync::mpsc::channel;
    use std::io::Error;

    struct TestOutput {
        val:bool
    }

    impl Output for TestOutput {
        fn value(&mut self, val:bool) {
            self.val = val;
        }
    }

    struct TestWriter {
        tx: Sender<u8>
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            
            for x in buf {
                self.tx.send(x.clone()).unwrap();
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    struct TestSink {
        rx: Receiver<u8>
    }

    impl TestSink {
        fn read_all(&mut self) -> Vec<u8> {
            let mut res = Vec::new();

            loop {
                match self.rx.recv() {
                    Ok(val) => {
                        res.push(val);
                    },
                    Err(_) => { break; }    
                }
            }

            res
        }
    }

    fn test_channel() -> (TestWriter, TestSink) {
        let (tx, rx) = channel();
        
        let writer = TestWriter {
            tx: tx
        };

        let sink = TestSink {
            rx: rx
        };

        (writer, sink)
    }


    #[test]
    fn it_works() {
        let tx_pwr = TestOutput { val:false };
        let (writer, mut sink) = test_channel();
        let rg = RainGauge::new(tx_pwr, writer);
        rg.tip(false);
        rg.tip(true);        
        rg.tip(true);                
        rg.stop();

        use std::str;

        let res = sink.read_all();
        println!("{}", str::from_utf8(&res).unwrap());
    }
}
