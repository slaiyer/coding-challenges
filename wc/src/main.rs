use std::env;
use std::error;
use std::fs;
use std::io;

enum Opts {
    All,
    Lines,
    Words,
    Bytes,
    Chars,
}

impl Opts {
    fn from_str(s: &str) -> Opts {
        match s {
            "l" => Opts::Lines,
            "w" => Opts::Words,
            "c" => Opts::Bytes,
            "m" => Opts::Chars,
            _ => Opts::All,
        }
    }
}

// TODO: fix incorrect counts with smaller buffers; likely due to splitting multi-byte characters
const BUF_LEN: usize = 1_024_000;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let opts: Opts;

    let mut reader_buffered: Box<dyn io::BufRead>;
    let stdin = io::stdin().lock();

    match args.len() {
        1 => {
            opts = Opts::All;
            reader_buffered = Box::new(create_buffered_reader(BUF_LEN, stdin));
        }
        2 => {
            if &args[1][..1] != "-" {
                opts = Opts::All;
                let file = &args[1];
                reader_buffered = Box::new(create_buffered_reader(
                    BUF_LEN,
                    fs::File::open(file)?,
                ));
            } else {
                opts = Opts::from_str(&args[1][1..]);
                reader_buffered = Box::new(create_buffered_reader(BUF_LEN, stdin));
            }
        }
        3 => {
            opts = Opts::from_str(&args[1][1..]);
            let file = &args[2];
            reader_buffered = Box::new(create_buffered_reader(
                BUF_LEN,
                fs::File::open(file)?,
            ));
        }
        _ => return Err(From::from("invalid arguments"))
    }

    match process(&mut reader_buffered, opts) {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e)
    }

    if args.len() > 1 && &args[1][..1] != "-" {
        println!(" {}", &args[args.len() - 1])
    } else {
        println!()
    }

    Ok(())
}

fn process(reader: &mut Box<dyn io::BufRead>, opts: Opts) -> Result<(), Box<dyn error::Error>> {
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut chars = 0;

    let mut buf = [0; BUF_LEN];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break
        }

        let slice = std::str::from_utf8(&buf[..n])?;

        match opts {
            Opts::All => {
                bytes += n;
                lines += count_lines(slice);
                words += count_words(slice);
                chars += count_chars(slice);
            }
            Opts::Lines => {
                lines += count_lines(slice)
            }
            Opts::Words => {
                words += count_words(slice)
            }
            Opts::Bytes => {
                bytes += n
            }
            Opts::Chars => {
                chars += count_chars(slice)
            }
        }
    }

    match opts {
        Opts::All => {
            print!("{:>8}{:>8}{:>8}{:>8}", lines, words, bytes, chars)
        }
        Opts::Lines => {
            print!("{:>8}", lines)
        }
        Opts::Words => {
            print!("{:>8}", words)
        }
        Opts::Bytes => {
            print!("{:>8}", bytes)
        }
        Opts::Chars => {
            print!("{:>8}", chars)
        }
    }

    Ok(())
}

fn count_lines(slice: &str) -> usize {
    slice.lines().count()
}

fn count_words(slice: &str) -> usize {
    slice.split_whitespace().count()
}

fn count_chars(slice: &str) -> usize {
    slice.chars().count()
}

fn create_buffered_reader<R: io::Read>(buf_len: usize, input: R) -> impl io::BufRead {
    io::BufReader::with_capacity(buf_len, input)
}
