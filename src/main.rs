use std::{
    env,
    io::{self, Write},
    net::{IpAddr, TcpStream},
    process,
    str::FromStr,
    sync::mpsc::{channel, Sender},
    thread,
};

const MAX: u16 = 65535;

struct Arguments {
    #[allow(dead_code)]
    flag: String,
    addr: IpAddr,
    threads: u16,
}

enum ErrorType {
    NotEnough,
    TooMany,
    Help,
    NotValidIp,
    NotValidThreads,
    InvalidSyntax,
}

fn get_error_message(err_type: ErrorType) -> &'static str {
    match err_type {
        ErrorType::NotEnough => "Not enough arguments",
        ErrorType::TooMany => "Too many arguments",
        ErrorType::Help => "Help doc",
        ErrorType::NotValidIp => "Not a valid ipv4 or ipv6 address",
        ErrorType::NotValidThreads => "Not a valid number",
        ErrorType::InvalidSyntax => "Invalid syntax",
    }
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err(get_error_message(ErrorType::NotEnough));
        } else if args.len() > 4 {
            return Err(get_error_message(ErrorType::TooMany));
        }

        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                addr: ipaddr,
                threads: 4,
            });
        } else {
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("--help") {
                if args.len() == 2 {
                    println!("Usage: -j to select how many threads you want\r\n       -h or --help to show this help message.");
                    return Err(get_error_message(ErrorType::Help));
                } else {
                    return Err(get_error_message(ErrorType::TooMany));
                }
            } else if flag.contains("-j") {
                let addr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err(get_error_message(ErrorType::NotValidIp)),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(n) => n,
                    Err(_) => return Err(get_error_message(ErrorType::NotValidThreads)),
                };

                return Ok(Arguments {
                    flag,
                    addr,
                    threads,
                });
            } else {
                return Err(get_error_message(ErrorType::InvalidSyntax));
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }

        if (MAX - port) <= threads {
            break;
        }
        port += threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(1);
        }
    });

    let threads = arguments.threads;
    let (tx, rx) = channel();
    for i in 0..threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, arguments.addr, threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{v} is open");
    }
}
